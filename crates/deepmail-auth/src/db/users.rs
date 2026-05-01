//! Database access layer for the `users` table.
//!
//! All functions use `sqlx::query!` for compile-time SQL verification.
//! No string-interpolated SQL anywhere in this file.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AuthError;

/// A user row returned from the database.
#[derive(Debug, Clone)]
pub struct UserRow {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub is_active: bool,
    pub email_verified: bool,
    pub is_flagged: bool,
    pub flagged_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

/// Insert a new user row. Returns the new user's UUID.
///
/// Returns `AuthError::UserAlreadyExists` if email or username conflicts.
pub async fn insert_user(
    pool: &PgPool,
    username: &str,
    email: &str,
    password_hash: &str,
) -> Result<Uuid, AuthError> {
    let row = sqlx::query!(
        r#"
        INSERT INTO users (username, email, password_hash)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
        username,
        email,
        password_hash,
    )
    .fetch_one(pool)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(ref dbe) = e {
            if dbe.code().as_deref() == Some("23505") {
                return AuthError::UserAlreadyExists;
            }
        }
        AuthError::Database(e)
    })?;
    Ok(row.id)
}

/// Fetch a user by email (case-insensitive via CITEXT).
pub async fn get_user_by_email(
    pool: &PgPool,
    email: &str,
) -> Result<Option<UserRow>, AuthError> {
    let row = sqlx::query!(
        r#"
        SELECT id,
               username,
               email   AS "email!: String",
               password_hash,
               is_active,
               email_verified,
               is_flagged,
               flagged_reason,
               created_at,
               updated_at,
               last_login_at
        FROM users
        WHERE email = $1
          AND deleted_at IS NULL
        "#,
        email,
    )
    .fetch_optional(pool)
    .await
    .map_err(AuthError::Database)?;

    Ok(row.map(|r| UserRow {
        id: r.id,
        username: r.username,
        email: r.email,
        password_hash: r.password_hash,
        is_active: r.is_active,
        email_verified: r.email_verified,
        is_flagged: r.is_flagged,
        flagged_reason: r.flagged_reason,
        created_at: r.created_at,
        updated_at: r.updated_at,
        last_login_at: r.last_login_at,
    }))
}

/// Fetch a user by UUID.
pub async fn get_user_by_id(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<UserRow>, AuthError> {
    let row = sqlx::query!(
        r#"
        SELECT id,
               username,
               email   AS "email!: String",
               password_hash,
               is_active,
               email_verified,
               is_flagged,
               flagged_reason,
               created_at,
               updated_at,
               last_login_at
        FROM users
        WHERE id = $1
          AND deleted_at IS NULL
        "#,
        user_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(AuthError::Database)?;

    Ok(row.map(|r| UserRow {
        id: r.id,
        username: r.username,
        email: r.email,
        password_hash: r.password_hash,
        is_active: r.is_active,
        email_verified: r.email_verified,
        is_flagged: r.is_flagged,
        flagged_reason: r.flagged_reason,
        created_at: r.created_at,
        updated_at: r.updated_at,
        last_login_at: r.last_login_at,
    }))
}

/// Activate a user's account (`is_active=true`, `email_verified=true`).
pub async fn activate_user(pool: &PgPool, user_id: Uuid) -> Result<(), AuthError> {
    sqlx::query!(
        r#"
        UPDATE users
        SET is_active = true,
            email_verified = true,
            updated_at = now()
        WHERE id = $1
        "#,
        user_id,
    )
    .execute(pool)
    .await
    .map_err(AuthError::Database)?;
    Ok(())
}

/// Record the `last_login_at` timestamp for a user.
pub async fn record_login(pool: &PgPool, user_id: Uuid) -> Result<(), AuthError> {
    sqlx::query!(
        r#"
        UPDATE users
        SET last_login_at = now(),
            updated_at = now()
        WHERE id = $1
        "#,
        user_id,
    )
    .execute(pool)
    .await
    .map_err(AuthError::Database)?;
    Ok(())
}
