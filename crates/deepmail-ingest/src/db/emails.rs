//! CRUD for the emails table.

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::IngestError;

/// Insert a new email record after successful validation and S3 upload.
/// Returns the new email UUID.
pub async fn insert_email(
    pool: &PgPool,
    tenant_id: Uuid,
    uploaded_by: Uuid,
    original_filename: &str,
    quarantine_name: &str,
    s3_bucket: &str,
    s3_key: &str,
    sha256_hash: &str,
    md5_hash: &str,
    file_size_bytes: i64,
    file_extension: &str,
    mime_type: &str,
    magic_bytes_valid: bool,
    nats_message_id: Option<&str>,
) -> Result<Uuid, IngestError> {
    let row = sqlx::query!(
        r#"
        INSERT INTO emails (
          tenant_id, uploaded_by, original_filename, quarantine_name,
          s3_bucket, s3_key, sha256_hash, md5_hash,
          file_size_bytes, file_extension, mime_type,
          magic_bytes_valid, nats_message_id
        )
        VALUES (
          $1, $2, $3, $4,
          $5, $6, $7, $8,
          $9, $10, $11,
          $12, $13
        )
        RETURNING id
        "#,
        tenant_id,
        uploaded_by,
        original_filename,
        quarantine_name,
        s3_bucket,
        s3_key,
        sha256_hash,
        md5_hash,
        file_size_bytes,
        file_extension,
        mime_type,
        magic_bytes_valid,
        nats_message_id,
    )
    .fetch_one(pool)
    .await
    .map_err(IngestError::Database)?;

    Ok(row.id)
}

/// Update the nats_message_id on an email after NATS publish.
pub async fn set_nats_message_id(
    pool: &PgPool,
    email_id: Uuid,
    nats_message_id: &str,
) -> Result<(), IngestError> {
    sqlx::query!(
        r#"
        UPDATE emails
        SET nats_message_id = $2
        WHERE id = $1
        "#,
        email_id,
        nats_message_id,
    )
    .execute(pool)
    .await
    .map_err(IngestError::Database)?;
    Ok(())
}

/// Mark an email as rejected with a reason.
pub async fn reject_email(
    pool: &PgPool,
    email_id: Uuid,
    reason: &str,
) -> Result<(), IngestError> {
    sqlx::query!(
        r#"
        UPDATE emails
        SET status = 'rejected', rejection_reason = $2
        WHERE id = $1
        "#,
        email_id,
        reason,
    )
    .execute(pool)
    .await
    .map_err(IngestError::Database)?;
    Ok(())
}
