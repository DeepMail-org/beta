//! Database access layer for `login_attempts` and `ip_lockouts`.

use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AuthError;

/// Check whether an IP address is currently locked out.
///
/// Returns `Some(locked_until)` if locked, `None` otherwise.
pub async fn check_lockout(
    pool: &PgPool,
    ip_address: &str,
) -> Result<Option<DateTime<Utc>>, AuthError> {
    let ip: std::net::IpAddr = ip_address
        .parse()
        .map_err(|_| AuthError::Validation("invalid IP address".into()))?;

    let row = sqlx::query!(
        r#"
        SELECT locked_until
        FROM ip_lockouts
        WHERE ip_address = $1
          AND locked_until > now()
        "#,
        ip as std::net::IpAddr,
    )
    .fetch_optional(pool)
    .await
    .map_err(AuthError::Database)?;

    Ok(row.map(|r| r.locked_until))
}

/// Record a login attempt and apply IP lockout if the per-hour failure
/// threshold has been crossed.
///
/// Returns true if a lockout was applied on this call.
#[allow(clippy::too_many_arguments)]
pub async fn record_login_attempt(
    pool: &PgPool,
    email: Option<&str>,
    user_id: Option<Uuid>,
    ip_address: &str,
    user_agent: Option<&str>,
    success: bool,
    failure_reason: Option<&str>,
    otp_stage: bool,
    lockout_max_attempts: i32,
    lockout_duration_minutes: u64,
) -> Result<bool, AuthError> {
    let ip: std::net::IpAddr = ip_address
        .parse()
        .map_err(|_| AuthError::Validation("invalid IP address".into()))?;

    sqlx::query!(
        r#"
        INSERT INTO login_attempts
          (email, user_id, ip_address, user_agent,
           success, failure_reason, otp_stage)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
        email,
        user_id,
        ip as std::net::IpAddr,
        user_agent,
        success,
        failure_reason,
        otp_stage,
    )
    .execute(pool)
    .await
    .map_err(AuthError::Database)?;

    if success {
        return Ok(false);
    }

    let recent_failures = sqlx::query!(
        r#"
        SELECT COUNT(*) as cnt
        FROM login_attempts
        WHERE ip_address = $1
          AND success = false
          AND created_at > now() - INTERVAL '1 hour'
        "#,
        ip as std::net::IpAddr,
    )
    .fetch_one(pool)
    .await
    .map_err(AuthError::Database)?;

    let count = recent_failures.cnt.unwrap_or(0);

    if count >= lockout_max_attempts as i64 {
        let locked_until =
            Utc::now() + Duration::minutes(lockout_duration_minutes as i64);

        sqlx::query!(
            r#"
            INSERT INTO ip_lockouts
              (ip_address, locked_until, attempt_count, lockout_count)
            VALUES ($1, $2, $3, 1)
            ON CONFLICT (ip_address) DO UPDATE
              SET locked_until   = EXCLUDED.locked_until,
                  attempt_count  = ip_lockouts.attempt_count + 1,
                  lockout_count  = ip_lockouts.lockout_count + 1,
                  updated_at     = now()
            "#,
            ip as std::net::IpAddr,
            locked_until,
            count as i32,
        )
        .execute(pool)
        .await
        .map_err(AuthError::Database)?;

        return Ok(true);
    }

    Ok(false)
}
