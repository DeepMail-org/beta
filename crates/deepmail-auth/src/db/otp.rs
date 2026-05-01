//! Database access layer for the `otp_codes` table.

use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AuthError;

/// Insert a new OTP code row. Marks any previous active codes
/// for the same `(user_id, otp_type)` pair as used before inserting.
pub async fn insert_otp(
    pool: &PgPool,
    user_id: Uuid,
    code_hash: &str,
    otp_type: &str,
    expiry_seconds: u64,
    ip_address: Option<&str>,
    user_agent: Option<&str>,
) -> Result<Uuid, AuthError> {
    let expires_at = Utc::now() + Duration::seconds(expiry_seconds as i64);

    sqlx::query!(
        r#"
        UPDATE otp_codes
        SET used = true,
            used_at = now()
        WHERE user_id = $1
          AND otp_type = $2
          AND used = false
        "#,
        user_id,
        otp_type,
    )
    .execute(pool)
    .await
    .map_err(AuthError::Database)?;

    let ip: Option<std::net::IpAddr> = ip_address.and_then(|s| s.parse().ok());

    let row = sqlx::query!(
        r#"
        INSERT INTO otp_codes
          (user_id, code_hash, otp_type, expires_at, ip_address, user_agent)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id
        "#,
        user_id,
        code_hash,
        otp_type,
        expires_at,
        ip as Option<std::net::IpAddr>,
        user_agent,
    )
    .fetch_one(pool)
    .await
    .map_err(AuthError::Database)?;

    Ok(row.id)
}

/// One row of the `otp_codes` table.
pub struct OtpRow {
    pub id: Uuid,
    pub code_hash: String,
    pub expires_at: DateTime<Utc>,
    pub attempts: i32,
    pub max_attempts: i32,
}

/// Fetch the most recent unused, unexpired OTP for a user and type.
pub async fn get_active_otp(
    pool: &PgPool,
    user_id: Uuid,
    otp_type: &str,
) -> Result<Option<OtpRow>, AuthError> {
    let row = sqlx::query!(
        r#"
        SELECT id, code_hash, expires_at, attempts, max_attempts
        FROM otp_codes
        WHERE user_id = $1
          AND otp_type = $2
          AND used = false
          AND expires_at > now()
        ORDER BY created_at DESC
        LIMIT 1
        "#,
        user_id,
        otp_type,
    )
    .fetch_optional(pool)
    .await
    .map_err(AuthError::Database)?;

    Ok(row.map(|r| OtpRow {
        id: r.id,
        code_hash: r.code_hash,
        expires_at: r.expires_at,
        attempts: r.attempts,
        max_attempts: r.max_attempts,
    }))
}

/// Increment the attempt counter for an OTP code.
pub async fn increment_otp_attempts(
    pool: &PgPool,
    otp_id: Uuid,
) -> Result<(), AuthError> {
    sqlx::query!(
        r#"
        UPDATE otp_codes
        SET attempts = attempts + 1
        WHERE id = $1
        "#,
        otp_id,
    )
    .execute(pool)
    .await
    .map_err(AuthError::Database)?;
    Ok(())
}

/// Mark an OTP as used.
pub async fn mark_otp_used(pool: &PgPool, otp_id: Uuid) -> Result<(), AuthError> {
    sqlx::query!(
        r#"
        UPDATE otp_codes
        SET used = true,
            used_at = now()
        WHERE id = $1
        "#,
        otp_id,
    )
    .execute(pool)
    .await
    .map_err(AuthError::Database)?;
    Ok(())
}
