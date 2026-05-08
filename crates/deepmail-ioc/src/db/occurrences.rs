/// Email-IOC occurrence database operations.

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::IocError;

/// Insert an occurrence (idempotent — ON CONFLICT DO NOTHING).
pub async fn insert_occurrence(
    pool: &PgPool,
    email_id: Uuid,
    tenant_id: Uuid,
    ioc_node_id: Uuid,
    extraction_source: &str,
    raw_value: &str,
) -> Result<(), IocError> {
    sqlx::query(
        r#"INSERT INTO email_ioc_occurrences
             (email_id, tenant_id, ioc_node_id, extraction_source, raw_value)
           VALUES ($1, $2, $3, $4, $5)
           ON CONFLICT (email_id, ioc_node_id) DO NOTHING"#,
    )
    .bind(email_id)
    .bind(tenant_id)
    .bind(ioc_node_id)
    .bind(extraction_source)
    .bind(raw_value)
    .execute(pool)
    .await?;

    Ok(())
}

/// Check if IOCs have already been extracted for this email (idempotency).
pub async fn has_occurrences(pool: &PgPool, email_id: Uuid) -> Result<bool, IocError> {
    let exists: (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM email_ioc_occurrences WHERE email_id = $1 LIMIT 1)",
    )
    .bind(email_id)
    .fetch_one(pool)
    .await?;

    Ok(exists.0)
}
