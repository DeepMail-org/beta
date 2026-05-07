use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::error::DkimError;

pub struct CachedKey {
    pub selector: String,
    pub domain: String,
    pub public_key_pem: Option<String>,
    pub algorithm: Option<String>,
    pub fetched_at: DateTime<Utc>,
    pub ttl_seconds: i32,
}

impl CachedKey {
    pub fn is_expired(&self) -> bool {
        let now = Utc::now();
        let age = (now - self.fetched_at).num_seconds();
        age > self.ttl_seconds as i64
    }
}

pub async fn get_cached_key(
    pool: &PgPool,
    selector: &str,
    domain: &str,
) -> Result<Option<CachedKey>, DkimError> {
    let row = sqlx::query_as!(
        CachedKey,
        r#"SELECT selector, domain, public_key_pem, algorithm, fetched_at, ttl_seconds
           FROM dkim_key_cache
           WHERE selector = $1 AND domain = $2"#,
        selector,
        domain,
    )
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn upsert_key(
    pool: &PgPool,
    selector: &str,
    domain: &str,
    public_key_pem: Option<&str>,
    algorithm: Option<&str>,
    ttl_seconds: i32,
) -> Result<(), DkimError> {
    sqlx::query!(
        r#"INSERT INTO dkim_key_cache (selector, domain, public_key_pem, algorithm, ttl_seconds, fetched_at)
           VALUES ($1, $2, $3, $4, $5, now())
           ON CONFLICT (selector, domain) DO UPDATE SET
             public_key_pem = EXCLUDED.public_key_pem,
             algorithm      = EXCLUDED.algorithm,
             ttl_seconds    = EXCLUDED.ttl_seconds,
             fetched_at     = now()"#,
        selector,
        domain,
        public_key_pem,
        algorithm,
        ttl_seconds,
    )
    .execute(pool)
    .await?;

    Ok(())
}
