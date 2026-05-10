/// Database operations for dynamic analysis jobs.

use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool, Row};
use uuid::Uuid;

use crate::error::DynamicError;

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct DynamicJobRow {
    pub id: Uuid,
    pub email_id: Uuid,
    pub tenant_id: Uuid,
    pub attachment_id: Uuid,
    pub s3_key: String,
    pub filename: String,
    pub sha256_hash: Option<String>,
    pub cape_task_id: Option<i32>,
    pub status: String,
    pub attempt_count: i32,
    pub cape_unavailable: bool,
    pub submitted_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create a new job. ON CONFLICT attachment_id DO NOTHING.
/// Returns the job ID (fetched if conflict).
pub async fn create_job(
    pool: &PgPool,
    email_id: Uuid,
    tenant_id: Uuid,
    attachment_id: Uuid,
    s3_key: &str,
    filename: &str,
    sha256_hash: Option<&str>,
) -> Result<Uuid, DynamicError> {
    let result = sqlx::query(
        "INSERT INTO dynamic_analysis_jobs
            (email_id, tenant_id, attachment_id, s3_key, filename, sha256_hash)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (attachment_id) DO NOTHING
         RETURNING id",
    )
    .bind(email_id)
    .bind(tenant_id)
    .bind(attachment_id)
    .bind(s3_key)
    .bind(filename)
    .bind(sha256_hash)
    .fetch_optional(pool)
    .await?;

    match result {
        Some(row) => Ok(row.get("id")),
        None => {
            // Conflict: fetch existing
            let row = sqlx::query("SELECT id FROM dynamic_analysis_jobs WHERE attachment_id = $1")
                .bind(attachment_id)
                .fetch_one(pool)
                .await?;
            Ok(row.get("id"))
        }
    }
}

/// Update job status.
pub async fn update_status(
    pool: &PgPool,
    job_id: Uuid,
    status: &str,
    error: Option<&str>,
) -> Result<(), DynamicError> {
    sqlx::query(
        "UPDATE dynamic_analysis_jobs
         SET status = $2, error_message = $3, updated_at = now(),
             completed_at = CASE WHEN $2 IN ('completed','failed','timeout') THEN now() ELSE completed_at END
         WHERE id = $1",
    )
    .bind(job_id)
    .bind(status)
    .bind(error)
    .execute(pool)
    .await?;
    Ok(())
}

/// Update CAPE task ID and status to submitted.
pub async fn update_cape_task_id(
    pool: &PgPool,
    job_id: Uuid,
    task_id: u64,
) -> Result<(), DynamicError> {
    sqlx::query(
        "UPDATE dynamic_analysis_jobs
         SET cape_task_id = $2, status = 'submitted', submitted_at = now(), updated_at = now()
         WHERE id = $1",
    )
    .bind(job_id)
    .bind(task_id as i32)
    .execute(pool)
    .await?;
    Ok(())
}

/// Get job by ID.
pub async fn get_job(pool: &PgPool, job_id: Uuid) -> Result<Option<DynamicJobRow>, DynamicError> {
    let row = sqlx::query_as::<_, DynamicJobRow>(
        "SELECT * FROM dynamic_analysis_jobs WHERE id = $1",
    )
    .bind(job_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// Get job by attachment_id.
pub async fn get_by_attachment_id(
    pool: &PgPool,
    attachment_id: Uuid,
) -> Result<Option<DynamicJobRow>, DynamicError> {
    let row = sqlx::query_as::<_, DynamicJobRow>(
        "SELECT * FROM dynamic_analysis_jobs WHERE attachment_id = $1",
    )
    .bind(attachment_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// Mark job as cape_unavailable.
pub async fn mark_cape_unavailable(
    pool: &PgPool,
    job_id: Uuid,
) -> Result<(), DynamicError> {
    sqlx::query(
        "UPDATE dynamic_analysis_jobs SET cape_unavailable = true, updated_at = now() WHERE id = $1",
    )
    .bind(job_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Increment attempt count.
pub async fn increment_attempt(pool: &PgPool, job_id: Uuid) -> Result<(), DynamicError> {
    sqlx::query(
        "UPDATE dynamic_analysis_jobs SET attempt_count = attempt_count + 1, updated_at = now() WHERE id = $1",
    )
    .bind(job_id)
    .execute(pool)
    .await?;
    Ok(())
}
