//! Cross-service job_progress updates (writes to deepmail_ingest DB).
//!
//! These queries use `sqlx::query()` (runtime-checked) instead of
//! `sqlx::query!()` because the ingest database schema is not available
//! at compile time under DATABASE_URL (which points at deepmail_parser).

use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::ParserError;

/// Mark the "parse" stage as running in the ingest DB.
pub async fn mark_parse_running(
    ingest_pool: &PgPool,
    email_id: Uuid,
    worker_id: &str,
) -> Result<(), ParserError> {
    let now = Utc::now();
    sqlx::query(
        r#"
        UPDATE job_progress
        SET status     = 'running',
            worker_id  = $2,
            started_at = $3
        WHERE email_id = $1
          AND stage    = 'parse'
          AND status   = 'pending'
        "#,
    )
    .bind(email_id)
    .bind(worker_id)
    .bind(now)
    .execute(ingest_pool)
    .await?;

    Ok(())
}

/// Mark the "parse" stage as completed in the ingest DB.
pub async fn mark_parse_completed(
    ingest_pool: &PgPool,
    email_id: Uuid,
    duration_ms: i32,
) -> Result<(), ParserError> {
    let now = Utc::now();
    sqlx::query(
        r#"
        UPDATE job_progress
        SET status       = 'completed',
            completed_at = $2,
            duration_ms  = $3
        WHERE email_id   = $1
          AND stage      = 'parse'
        "#,
    )
    .bind(email_id)
    .bind(now)
    .bind(duration_ms)
    .execute(ingest_pool)
    .await?;

    Ok(())
}

/// Mark the "parse" stage as failed in the ingest DB.
pub async fn mark_parse_failed(
    ingest_pool: &PgPool,
    email_id: Uuid,
    error_message: &str,
) -> Result<(), ParserError> {
    let now = Utc::now();
    sqlx::query(
        r#"
        UPDATE job_progress
        SET status        = 'failed',
            completed_at  = $2,
            error_message = $3
        WHERE email_id    = $1
          AND stage       = 'parse'
        "#,
    )
    .bind(email_id)
    .bind(now)
    .bind(error_message)
    .execute(ingest_pool)
    .await?;

    Ok(())
}
