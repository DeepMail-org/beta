/// Redis + DB cache layer with per-provider TTLs.

use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use sqlx::PgPool;

use crate::error::IntelError;

/// Redis key format: deepmail:intel:{provider}:{ioc_type}:{ioc_value}
fn redis_key(provider: &str, ioc_type: &str, ioc_value: &str) -> String {
    format!("deepmail:intel:{provider}:{ioc_type}:{ioc_value}")
}

pub async fn get_cached_redis(
    redis: &mut ConnectionManager,
    provider: &str,
    ioc_type: &str,
    ioc_value: &str,
) -> Result<Option<serde_json::Value>, IntelError> {
    let key = redis_key(provider, ioc_type, ioc_value);
    let cached: Option<String> = redis.get(&key).await?;
    match cached {
        Some(json) => {
            let val: serde_json::Value =
                serde_json::from_str(&json).map_err(|e| IntelError::Parse(e.to_string()))?;
            Ok(Some(val))
        }
        None => Ok(None),
    }
}

pub async fn set_cached_redis(
    redis: &mut ConnectionManager,
    provider: &str,
    ioc_type: &str,
    ioc_value: &str,
    value: &serde_json::Value,
    ttl_secs: u64,
) -> Result<(), IntelError> {
    let key = redis_key(provider, ioc_type, ioc_value);
    let json = serde_json::to_string(value).map_err(|e| IntelError::Parse(e.to_string()))?;
    let _: () = redis.set_ex(&key, &json, ttl_secs).await?;
    Ok(())
}

pub async fn get_cached_db(
    pool: &PgPool,
    ioc_type: &str,
    ioc_value: &str,
    provider: &str,
) -> Result<Option<CachedResult>, IntelError> {
    let row = sqlx::query_as::<_, CachedResult>(
        r#"SELECT id, ioc_type, ioc_value, provider, result_json,
                  vt_score, abuse_score, pulse_count, fetched_at, expires_at
           FROM enrichment_cache
           WHERE ioc_type = $1 AND ioc_value = $2 AND provider = $3
             AND expires_at > now()"#,
    )
    .bind(ioc_type)
    .bind(ioc_value)
    .bind(provider)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn set_cached_db(
    pool: &PgPool,
    ioc_type: &str,
    ioc_value: &str,
    provider: &str,
    result_json: &serde_json::Value,
    vt_score: Option<f32>,
    abuse_score: Option<i32>,
    pulse_count: Option<i32>,
    ttl_secs: i64,
) -> Result<(), IntelError> {
    let expires_at = chrono::Utc::now() + chrono::Duration::seconds(ttl_secs);

    sqlx::query(
        r#"INSERT INTO enrichment_cache
               (ioc_type, ioc_value, provider, result_json,
                vt_score, abuse_score, pulse_count, expires_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
           ON CONFLICT (ioc_type, ioc_value, provider)
           DO UPDATE SET
               result_json = EXCLUDED.result_json,
               vt_score    = EXCLUDED.vt_score,
               abuse_score = EXCLUDED.abuse_score,
               pulse_count = EXCLUDED.pulse_count,
               fetched_at  = now(),
               expires_at  = EXCLUDED.expires_at"#,
    )
    .bind(ioc_type)
    .bind(ioc_value)
    .bind(provider)
    .bind(result_json)
    .bind(vt_score)
    .bind(abuse_score)
    .bind(pulse_count)
    .bind(expires_at)
    .execute(pool)
    .await?;

    Ok(())
}

/// Delete expired cache entries. Called by background task every hour.
pub async fn delete_expired(pool: &PgPool) -> Result<u64, IntelError> {
    let result = sqlx::query("DELETE FROM enrichment_cache WHERE expires_at < now()")
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

/// Background task: cleanup expired cache entries on interval.
pub async fn cache_cleanup_loop(pool: PgPool, interval_hours: u64) {
    let interval = std::time::Duration::from_secs(interval_hours * 3600);
    loop {
        tokio::time::sleep(interval).await;
        match delete_expired(&pool).await {
            Ok(count) => {
                if count > 0 {
                    tracing::info!(deleted = count, "cache cleanup completed");
                }
            }
            Err(e) => tracing::warn!(error = %e, "cache cleanup failed"),
        }
    }
}

/// Provider TTL lookup in seconds.
pub fn provider_ttl(provider: &str, ioc_type: &str) -> u64 {
    match provider {
        "virustotal" => match ioc_type {
            "hash" => 86400,
            _ => 3600,
        },
        "abuseipdb" => 3600,
        "greynoise" => 3600,
        "ipinfo" => 21600,
        "shodan" => 86400,
        "otx" => 3600,
        _ => 3600,
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CachedResult {
    pub id: uuid::Uuid,
    pub ioc_type: String,
    pub ioc_value: String,
    pub provider: String,
    pub result_json: serde_json::Value,
    pub vt_score: Option<f32>,
    pub abuse_score: Option<i32>,
    pub pulse_count: Option<i32>,
    pub fetched_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}
