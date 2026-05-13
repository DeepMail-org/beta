use chrono::Utc;
use redis::AsyncCommands;
use uuid::Uuid;

use crate::error::GatewayError;

/// Returns Ok(true) if allowed, Ok(false) if rate limited.
/// Uses atomic pipeline: INCR + EXPIRE in one round-trip to prevent
/// the race where a crash between INCR and EXPIRE leaves a key without TTL.
pub async fn check_rate_limit(
    redis: &mut redis::aio::ConnectionManager,
    tenant_id: Uuid,
    limit_per_minute: u32,
) -> Result<bool, GatewayError> {
    let bucket = Utc::now().timestamp() / 60;
    let key = format!("deepmail:ratelimit:{}:{}", tenant_id, bucket);

    // Atomic: INCR always sets TTL via pipeline regardless of count
    let (count,): (i64,) = redis::pipe()
        .atomic()
        .incr(&key, 1i64)
        .expire(&key, 60)
        .ignore()
        .query_async(redis)
        .await?;

    Ok(count <= limit_per_minute as i64)
}
