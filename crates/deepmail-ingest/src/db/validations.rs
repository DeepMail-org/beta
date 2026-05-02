//! CRUD for the file_validations table.

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::IngestError;

/// Insert one validation step result.
pub async fn insert_validation_step(
    pool: &PgPool,
    email_id: Uuid,
    step: &str,
    passed: bool,
    detail: Option<&str>,
) -> Result<(), IngestError> {
    sqlx::query!(
        r#"
        INSERT INTO file_validations (email_id, step, passed, detail)
        VALUES ($1, $2, $3, $4)
        "#,
        email_id,
        step,
        passed,
        detail,
    )
    .execute(pool)
    .await
    .map_err(IngestError::Database)?;
    Ok(())
}
