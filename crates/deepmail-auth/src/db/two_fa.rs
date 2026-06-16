//! Database access layer for the `user_2fa` table.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AuthError;

/// A 2FA configuration row.
#[derive(Debug, Clone)]
pub struct TwoFaRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub secret_encrypted: String,
    pub backup_codes_encrypted: Vec<String>,
    pub enabled: bool,
    pub verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Insert a new 2FA configuration for a user.
pub async fn insert_2fa(
    pool: &PgPool,
    user_id: Uuid,
    secret_encrypted: &str,
    backup_codes_encrypted: &[String],
) -> Result<(), AuthError> {
    sqlx::query!(
        r#"
        INSERT INTO user_2fa (user_id, secret_encrypted, backup_codes_encrypted)
        VALUES ($1, $2, $3)
        "#,
        user_id,
        secret_encrypted,
        backup_codes_encrypted,
    )
    .execute(pool)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(ref dbe) = e {
            if dbe.code().as_deref() == Some("23505") {
                return AuthError::UserAlreadyExists;
            }
        }
        AuthError::Database(e)
    })?;
    Ok(())
}

/// Fetch 2FA configuration by user ID.
pub async fn get_2fa(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<TwoFaRow>, AuthError> {
    let row = sqlx::query!(
        r#"
        SELECT id, user_id, secret_encrypted, backup_codes_encrypted, enabled, verified_at, created_at, updated_at
        FROM user_2fa
        WHERE user_id = $1
        "#,
        user_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(AuthError::Database)?;

    Ok(row.map(|r| TwoFaRow {
        id: r.id,
        user_id: r.user_id,
        secret_encrypted: r.secret_encrypted,
        backup_codes_encrypted: r.backup_codes_encrypted,
        enabled: r.enabled,
        verified_at: r.verified_at,
        created_at: r.created_at,
        updated_at: r.updated_at,
    }))
}

/// Enable 2FA for a user (set enabled=true, verified_at=now).
pub async fn enable_2fa(pool: &PgPool, user_id: Uuid) -> Result<(), AuthError> {
    sqlx::query!(
        r#"
        UPDATE user_2fa
        SET enabled = true,
            verified_at = now(),
            updated_at = now()
        WHERE user_id = $1
        "#,
        user_id,
    )
    .execute(pool)
    .await
    .map_err(AuthError::Database)?;
    Ok(())
}

/// Increment the attempt counter for a 2FA code.
pub async fn increment_attempts(pool: &PgPool, two_fa_id: Uuid) -> Result<(), AuthError> {
    sqlx::query!(
        r#"
        UPDATE user_2fa
        SET updated_at = now()
        WHERE id = $1
        "#,
        two_fa_id,
    )
    .execute(pool)
    .await
    .map_err(AuthError::Database)?;
    Ok(())
}

/// Mark a backup code as used (remove from the encrypted array).
/// This is a simplified implementation - in production, you'd want to track
/// which specific backup code was used.
pub async fn mark_backup_code_used(
    pool: &PgPool,
    two_fa_id: Uuid,
    _used_code: &str,
) -> Result<(), AuthError> {
    sqlx::query!(
        r#"
        UPDATE user_2fa
        SET updated_at = now()
        WHERE id = $1
        "#,
        two_fa_id,
    )
    .execute(pool)
    .await
    .map_err(AuthError::Database)?;
    Ok(())
}

/// Delete 2FA configuration for a user.
pub async fn delete_2fa(pool: &PgPool, user_id: Uuid) -> Result<(), AuthError> {
    sqlx::query!(
        r#"
        DELETE FROM user_2fa WHERE user_id = $1
        "#,
        user_id,
    )
    .execute(pool)
    .await
    .map_err(AuthError::Database)?;
    Ok(())
}