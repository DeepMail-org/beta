//! Database access layer for the `refresh_tokens` table.

use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AuthError;

/// Insert a new refresh token row.
pub async fn insert_refresh_token(
    pool: &PgPool,
    user_id: Uuid,
    token_hash: &str,
    expiry_days: u64,
    ip_address: Option<&str>,
    device_info: Option<&str>,
) -> Result<Uuid, AuthError> {
    let expires_at = Utc::now() + Duration::days(expiry_days as i64);
    let ip: Option<std::net::IpAddr> = ip_address.and_then(|s| s.parse().ok());

    let row = sqlx::query!(
        r#"
        INSERT INTO refresh_tokens
          (user_id, token_hash, expires_at, ip_address, device_info)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
        user_id,
        token_hash,
        expires_at,
        ip as Option<std::net::IpAddr>,
        device_info,
    )
    .fetch_one(pool)
    .await
    .map_err(AuthError::Database)?;

    Ok(row.id)
}

/// Fetch a non-revoked, unexpired refresh token by its hash.
///
/// Returns `(token_id, user_id)` on hit.
pub async fn get_refresh_token(
    pool: &PgPool,
    token_hash: &str,
) -> Result<Option<(Uuid, Uuid)>, AuthError> {
    let row = sqlx::query!(
        r#"
        SELECT id, user_id
        FROM refresh_tokens
        WHERE token_hash = $1
          AND revoked = false
          AND expires_at > now()
        "#,
        token_hash,
    )
    .fetch_optional(pool)
    .await
    .map_err(AuthError::Database)?;

    Ok(row.map(|r| (r.id, r.user_id)))
}

/// Revoke a refresh token by ID.
pub async fn revoke_refresh_token(
    pool: &PgPool,
    token_id: Uuid,
    reason: &str,
) -> Result<(), AuthError> {
    sqlx::query!(
        r#"
        UPDATE refresh_tokens
        SET revoked = true,
            revoked_at = now(),
            revoke_reason = $2
        WHERE id = $1
        "#,
        token_id,
        reason,
    )
    .execute(pool)
    .await
    .map_err(AuthError::Database)?;
    Ok(())
}
