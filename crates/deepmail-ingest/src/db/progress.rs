//! CRUD for the job_progress table.

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::IngestError;

/// All pipeline stages that get a progress row on ingest.
pub const ALL_STAGES: &[&str] = &[
    "parse",
    "header",
    "dkim",
    "geo",
    "ip",
    "intel",
    "ioc",
    "body",
    "homograph",
    "sandbox_url",
    "sandbox_file",
    "sandbox_dynamic",
    "hashdb",
    "scoring",
    "ml",
    "graph",
    "report",
];

/// Insert one job_progress row with status='pending'.
pub async fn insert_progress_row(
    pool: &PgPool,
    email_id: Uuid,
    tenant_id: Uuid,
    stage: &str,
) -> Result<(), IngestError> {
    sqlx::query!(
        r#"
        INSERT INTO job_progress (email_id, tenant_id, stage)
        VALUES ($1, $2, $3)
        "#,
        email_id,
        tenant_id,
        stage,
    )
    .execute(pool)
    .await
    .map_err(IngestError::Database)?;
    Ok(())
}

/// Insert progress rows for ALL pipeline stages in a single transaction.
pub async fn insert_all_progress_rows(
    pool: &PgPool,
    email_id: Uuid,
    tenant_id: Uuid,
) -> Result<(), IngestError> {
    let mut tx = pool.begin().await.map_err(IngestError::Database)?;

    for stage in ALL_STAGES {
        sqlx::query!(
            r#"
            INSERT INTO job_progress (email_id, tenant_id, stage)
            VALUES ($1, $2, $3)
            "#,
            email_id,
            tenant_id,
            stage,
        )
        .execute(&mut *tx)
        .await
        .map_err(IngestError::Database)?;
    }

    tx.commit().await.map_err(IngestError::Database)?;
    Ok(())
}
