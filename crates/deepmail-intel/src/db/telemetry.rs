/// DB operations for provider_telemetry table.

use sqlx::PgPool;

use crate::error::IntelError;

pub async fn insert_telemetry_window(
    pool: &PgPool,
    provider: &str,
    window_start: chrono::DateTime<chrono::Utc>,
    window_end: chrono::DateTime<chrono::Utc>,
    request_count: i32,
    success_count: i32,
    failure_count: i32,
    cache_hit_count: i32,
    total_latency_ms: i64,
    quota_remaining: Option<i32>,
) -> Result<(), IntelError> {
    sqlx::query(
        r#"INSERT INTO provider_telemetry
               (provider, window_start, window_end,
                request_count, success_count, failure_count,
                cache_hit_count, total_latency_ms, quota_remaining)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#,
    )
    .bind(provider)
    .bind(window_start)
    .bind(window_end)
    .bind(request_count)
    .bind(success_count)
    .bind(failure_count)
    .bind(cache_hit_count)
    .bind(total_latency_ms)
    .bind(quota_remaining)
    .execute(pool)
    .await?;

    Ok(())
}
