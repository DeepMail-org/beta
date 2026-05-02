//! CRUD for the tenants table.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::TenantError;

#[derive(Debug, Clone)]
pub struct TenantRow {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub owner_user_id: Uuid,
    pub status: String,
    pub razorpay_customer_id: Option<String>,
    pub billing_email: String,
    pub country: String,
    pub timezone: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create a new tenant. Also inserts the owner as a member with role 'owner'.
/// Also inserts default tenant_settings row.
/// All three inserts run in a single transaction.
pub async fn create_tenant(
    pool: &PgPool,
    name: &str,
    slug: &str,
    owner_user_id: Uuid,
    billing_email: &str,
    country: &str,
) -> Result<TenantRow, TenantError> {
    // Validate slug: lowercase alphanumeric + hyphens only
    if !slug.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        return Err(TenantError::Validation(
            "slug must be lowercase alphanumeric with hyphens only".into(),
        ));
    }

    let mut tx = pool.begin().await.map_err(TenantError::Database)?;

    let row = sqlx::query!(
        r#"
        INSERT INTO tenants (name, slug, owner_user_id, billing_email, country)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, name, slug, owner_user_id, status,
                  razorpay_customer_id,
                  billing_email AS "billing_email!: String",
                  country, timezone,
                  created_at, updated_at
        "#,
        name,
        slug,
        owner_user_id,
        billing_email,
        country,
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(ref dbe) = e {
            if dbe.code().as_deref() == Some("23505") {
                return TenantError::TenantAlreadyExists(slug.to_string());
            }
        }
        TenantError::Database(e)
    })?;

    // Insert owner as member
    sqlx::query!(
        r#"
        INSERT INTO tenant_members (tenant_id, user_id, role)
        VALUES ($1, $2, 'owner')
        "#,
        row.id,
        owner_user_id,
    )
    .execute(&mut *tx)
    .await
    .map_err(TenantError::Database)?;

    // Insert default settings
    sqlx::query!(
        r#"
        INSERT INTO tenant_settings (tenant_id)
        VALUES ($1)
        "#,
        row.id,
    )
    .execute(&mut *tx)
    .await
    .map_err(TenantError::Database)?;

    tx.commit().await.map_err(TenantError::Database)?;

    Ok(TenantRow {
        id: row.id,
        name: row.name,
        slug: row.slug,
        owner_user_id: row.owner_user_id,
        status: row.status,
        razorpay_customer_id: row.razorpay_customer_id,
        billing_email: row.billing_email,
        country: row.country,
        timezone: row.timezone,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

/// Fetch a tenant by UUID.
pub async fn get_tenant_by_id(
    pool: &PgPool,
    tenant_id: Uuid,
) -> Result<Option<TenantRow>, TenantError> {
    let row = sqlx::query!(
        r#"
        SELECT id, name, slug, owner_user_id, status,
               razorpay_customer_id,
               billing_email AS "billing_email!: String",
               country, timezone, created_at, updated_at
        FROM tenants
        WHERE id = $1
          AND deleted_at IS NULL
        "#,
        tenant_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(TenantError::Database)?;

    Ok(row.map(|r| TenantRow {
        id: r.id,
        name: r.name,
        slug: r.slug,
        owner_user_id: r.owner_user_id,
        status: r.status,
        razorpay_customer_id: r.razorpay_customer_id,
        billing_email: r.billing_email,
        country: r.country,
        timezone: r.timezone,
        created_at: r.created_at,
        updated_at: r.updated_at,
    }))
}

/// Fetch a tenant by slug.
pub async fn get_tenant_by_slug(
    pool: &PgPool,
    slug: &str,
) -> Result<Option<TenantRow>, TenantError> {
    let row = sqlx::query!(
        r#"
        SELECT id, name, slug, owner_user_id, status,
               razorpay_customer_id,
               billing_email AS "billing_email!: String",
               country, timezone, created_at, updated_at
        FROM tenants
        WHERE slug = $1
          AND deleted_at IS NULL
        "#,
        slug,
    )
    .fetch_optional(pool)
    .await
    .map_err(TenantError::Database)?;

    Ok(row.map(|r| TenantRow {
        id: r.id,
        name: r.name,
        slug: r.slug,
        owner_user_id: r.owner_user_id,
        status: r.status,
        razorpay_customer_id: r.razorpay_customer_id,
        billing_email: r.billing_email,
        country: r.country,
        timezone: r.timezone,
        created_at: r.created_at,
        updated_at: r.updated_at,
    }))
}

/// Update tenant status.
pub async fn update_tenant_status(
    pool: &PgPool,
    tenant_id: Uuid,
    status: &str,
) -> Result<(), TenantError> {
    sqlx::query!(
        r#"
        UPDATE tenants
        SET status = $2, updated_at = now()
        WHERE id = $1
          AND deleted_at IS NULL
        "#,
        tenant_id,
        status,
    )
    .execute(pool)
    .await
    .map_err(TenantError::Database)?;
    Ok(())
}

/// Soft-delete a tenant.
pub async fn delete_tenant(pool: &PgPool, tenant_id: Uuid) -> Result<(), TenantError> {
    sqlx::query!(
        r#"
        UPDATE tenants
        SET deleted_at = now(), updated_at = now(), status = 'cancelled'
        WHERE id = $1
          AND deleted_at IS NULL
        "#,
        tenant_id,
    )
    .execute(pool)
    .await
    .map_err(TenantError::Database)?;
    Ok(())
}
