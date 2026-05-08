/// DB cache operations (enrichment_cache table).

use sqlx::PgPool;
use crate::error::IntelError;

pub async fn upsert_cache(
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
    crate::cache::set_cached_db(
        pool, ioc_type, ioc_value, provider, result_json,
        vt_score, abuse_score, pulse_count, ttl_secs,
    )
    .await
}

pub async fn get_cache(
    pool: &PgPool,
    ioc_type: &str,
    ioc_value: &str,
    provider: &str,
) -> Result<Option<crate::cache::CachedResult>, IntelError> {
    crate::cache::get_cached_db(pool, ioc_type, ioc_value, provider).await
}

pub async fn delete_expired(pool: &PgPool) -> Result<u64, IntelError> {
    crate::cache::delete_expired(pool).await
}
