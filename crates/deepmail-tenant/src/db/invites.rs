//! CRUD for tenant_invites table.

use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::TenantError;

pub struct InviteRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub email: String,
    pub role: String,
    pub expires_at: DateTime<Utc>,
    pub accepted: bool,
}

/// Create an invite. token_hash is SHA-256 of the raw invite token.
pub async fn create_invite(
    pool: &PgPool,
    tenant_id: Uuid,
    email: &str,
    role: &str,
    token_hash: &str,
    invited_by: Uuid,
    ttl_hours: u64,
) -> Result<Uuid, TenantError> {
    let expires_at = Utc::now() + Duration::hours(ttl_hours as i64);

    let row = sqlx::query!(
        r#"
        INSERT INTO tenant_invites
          (tenant_id, email, role, token_hash, invited_by, expires_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id
        "#,
        tenant_id,
        email,
        role,
        token_hash,
        invited_by,
        expires_at,
    )
    .fetch_one(pool)
    .await
    .map_err(TenantError::Database)?;
    Ok(row.id)
}

/// Fetch an active invite by its token hash.
pub async fn get_invite_by_token_hash(
    pool: &PgPool,
    token_hash: &str,
) -> Result<Option<InviteRow>, TenantError> {
    let row = sqlx::query!(
        r#"
        SELECT id, tenant_id,
               email AS "email!: String",
               role, expires_at, accepted
        FROM tenant_invites
        WHERE token_hash = $1
        "#,
        token_hash,
    )
    .fetch_optional(pool)
    .await
    .map_err(TenantError::Database)?;

    Ok(row.map(|r| InviteRow {
        id: r.id,
        tenant_id: r.tenant_id,
        email: r.email,
        role: r.role,
        expires_at: r.expires_at,
        accepted: r.accepted,
    }))
}

/// Mark an invite as accepted.
pub async fn accept_invite(
    pool: &PgPool,
    invite_id: Uuid,
) -> Result<(), TenantError> {
    sqlx::query!(
        r#"
        UPDATE tenant_invites
        SET accepted = true, accepted_at = now()
        WHERE id = $1
        "#,
        invite_id,
    )
    .execute(pool)
    .await
    .map_err(TenantError::Database)?;
    Ok(())
}
