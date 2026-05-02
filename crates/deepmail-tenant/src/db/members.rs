//! CRUD for tenant_members table.

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::TenantError;

#[derive(Debug, Clone)]
pub struct MemberRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
}

/// Add a user to a tenant with a given role.
pub async fn add_member(
    pool: &PgPool,
    tenant_id: Uuid,
    user_id: Uuid,
    role: &str,
    invited_by: Option<Uuid>,
) -> Result<Uuid, TenantError> {
    let row = sqlx::query!(
        r#"
        INSERT INTO tenant_members (tenant_id, user_id, role, invited_by)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
        tenant_id,
        user_id,
        role,
        invited_by,
    )
    .fetch_one(pool)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(ref dbe) = e {
            if dbe.code().as_deref() == Some("23505") {
                return TenantError::AlreadyMember;
            }
        }
        TenantError::Database(e)
    })?;
    Ok(row.id)
}

/// Get a member's role in a tenant. Returns None if not a member.
pub async fn get_member_role(
    pool: &PgPool,
    tenant_id: Uuid,
    user_id: Uuid,
) -> Result<Option<String>, TenantError> {
    let row = sqlx::query!(
        r#"
        SELECT role
        FROM tenant_members
        WHERE tenant_id = $1 AND user_id = $2
        "#,
        tenant_id,
        user_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(TenantError::Database)?;

    Ok(row.map(|r| r.role))
}

/// List all members of a tenant.
pub async fn list_members(
    pool: &PgPool,
    tenant_id: Uuid,
) -> Result<Vec<MemberRow>, TenantError> {
    let rows = sqlx::query!(
        r#"
        SELECT id, tenant_id, user_id, role
        FROM tenant_members
        WHERE tenant_id = $1
        ORDER BY joined_at ASC
        "#,
        tenant_id,
    )
    .fetch_all(pool)
    .await
    .map_err(TenantError::Database)?;

    Ok(rows
        .into_iter()
        .map(|r| MemberRow {
            id: r.id,
            tenant_id: r.tenant_id,
            user_id: r.user_id,
            role: r.role,
        })
        .collect())
}

/// Remove a member from a tenant (cannot remove the owner).
pub async fn remove_member(
    pool: &PgPool,
    tenant_id: Uuid,
    user_id: Uuid,
) -> Result<(), TenantError> {
    let result = sqlx::query!(
        r#"
        DELETE FROM tenant_members
        WHERE tenant_id = $1
          AND user_id = $2
          AND role != 'owner'
        "#,
        tenant_id,
        user_id,
    )
    .execute(pool)
    .await
    .map_err(TenantError::Database)?;

    if result.rows_affected() == 0 {
        return Err(TenantError::MemberNotFound);
    }
    Ok(())
}
