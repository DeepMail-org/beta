/// Database operations for url_sandbox_jobs.

use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool, Row};
use uuid::Uuid;

/// Row from url_sandbox_jobs table.
#[derive(Debug, Clone, FromRow)]
pub struct UrlSandboxJobRow {
    pub id: Uuid,
    pub email_id: Uuid,
    pub tenant_id: Uuid,
    pub url: String,
    pub url_type: String,
    pub status: String,
    pub attempt_count: i32,
    pub container_id: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create a new sandbox job. Returns the new job ID.
pub async fn create_job(
    pool: &PgPool,
    email_id: Uuid,
    tenant_id: Uuid,
    url: &str,
    url_type: &str,
) -> Result<Uuid, sqlx::Error> {
    let row = sqlx::query(
        "INSERT INTO url_sandbox_jobs (email_id, tenant_id, url, url_type)
         VALUES ($1, $2, $3, $4)
         RETURNING id"
    )
    .bind(email_id)
    .bind(tenant_id)
    .bind(url)
    .bind(url_type)
    .fetch_one(pool)
    .await?;

    Ok(row.get("id"))
}

/// Update job status and optionally set error message.
pub async fn update_status(
    pool: &PgPool,
    job_id: Uuid,
    status: &str,
    error: Option<&str>,
) -> Result<(), sqlx::Error> {
    let completed = if status == "completed" || status == "failed" || status == "timeout" {
        Some(Utc::now())
    } else {
        None
    };

    sqlx::query(
        "UPDATE url_sandbox_jobs
         SET status = $2, error_message = $3, completed_at = $4, updated_at = now()
         WHERE id = $1"
    )
    .bind(job_id)
    .bind(status)
    .bind(error)
    .bind(completed)
    .execute(pool)
    .await?;

    Ok(())
}

/// Mark a job as started: update status, started_at, container_id, increment attempt.
pub async fn update_started(
    pool: &PgPool,
    job_id: Uuid,
    container_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE url_sandbox_jobs
         SET status = 'running',
             started_at = now(),
             container_id = $2,
             attempt_count = attempt_count + 1,
             updated_at = now()
         WHERE id = $1"
    )
    .bind(job_id)
    .bind(container_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get a job by ID.
pub async fn get_job(
    pool: &PgPool,
    job_id: Uuid,
) -> Result<Option<UrlSandboxJobRow>, sqlx::Error> {
    let row = sqlx::query_as::<_, UrlSandboxJobRow>(
        "SELECT id, email_id, tenant_id, url, url_type, status,
                attempt_count, container_id, started_at, completed_at,
                error_message, created_at, updated_at
         FROM url_sandbox_jobs
         WHERE id = $1"
    )
    .bind(job_id)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

/// List pending jobs, ordered by creation time.
pub async fn list_pending(
    pool: &PgPool,
    limit: i64,
) -> Result<Vec<UrlSandboxJobRow>, sqlx::Error> {
    let rows = sqlx::query_as::<_, UrlSandboxJobRow>(
        "SELECT id, email_id, tenant_id, url, url_type, status,
                attempt_count, container_id, started_at, completed_at,
                error_message, created_at, updated_at
         FROM url_sandbox_jobs
         WHERE status = 'pending'
         ORDER BY created_at ASC
         LIMIT $1"
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// Check if a URL has already been processed for a given email.
pub async fn find_existing(
    pool: &PgPool,
    email_id: Uuid,
    url: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let row: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM url_sandbox_jobs
         WHERE email_id = $1 AND url = $2
         AND status IN ('completed', 'running')
         LIMIT 1"
    )
    .bind(email_id)
    .bind(url)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| r.0))
}
