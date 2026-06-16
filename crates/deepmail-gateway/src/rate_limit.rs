use chrono::Utc;
use redis::AsyncCommands;
use uuid::Uuid;

use crate::error::GatewayError;
use deepmail_common::zig_ratelimiter::{check_tenant_api, check_api};

/// Returns Ok(true) if allowed, Ok(false) if rate limited.
/// Uses a two-layer approach:
/// 1. Fast local Zig rate limiter (in-process, ~1-5μs)
/// 2. Distributed Redis rate limiter (authoritative, ~1-5ms)
pub async fn check_rate_limit(
    redis: &mut redis::aio::ConnectionManager,
    tenant_id: Uuid,
    client_ip: &str,
    limit_per_minute: u32,
) -> Result<bool, GatewayError> {
    // Layer 1: Fast local Zig rate limiter (per-tenant + per-IP)
    let tenant_api_key = format!("{}:{}", tenant_id, client_ip);
    if !check_tenant_api(&tenant_id.to_string(), &tenant_api_key) {
        tracing::warn!(
            tenant_id = %tenant_id,
            ip = %client_ip,
            "local rate limit exceeded (Zig)"
        );
        return Ok(false);
    }

    // Layer 2: Distributed Redis check (per-tenant)
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

/// Check general API rate limit (no tenant context)
pub fn check_general_api(key: &str) -> bool {
    deepmail_common::zig_ratelimiter::check_api(key)
}