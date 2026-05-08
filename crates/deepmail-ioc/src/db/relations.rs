/// IOC relation database operations.

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::IocError;
use crate::relations::IocRelation;

/// Bulk insert relations (idempotent — ON CONFLICT DO NOTHING).
pub async fn bulk_insert_relations(
    pool: &PgPool,
    tenant_id: Uuid,
    email_id: Uuid,
    relations: &[IocRelation],
) -> Result<(), IocError> {
    for rel in relations {
        sqlx::query(
            r#"INSERT INTO ioc_relations
                 (tenant_id, source_ioc_id, target_ioc_id, relation_type, email_id, confidence)
               VALUES ($1, $2, $3, $4, $5, $6)
               ON CONFLICT (source_ioc_id, target_ioc_id, relation_type, email_id) DO NOTHING"#,
        )
        .bind(tenant_id)
        .bind(rel.source_ioc_id)
        .bind(rel.target_ioc_id)
        .bind(&rel.relation_type)
        .bind(email_id)
        .bind(rel.confidence)
        .execute(pool)
        .await?;
    }

    Ok(())
}
