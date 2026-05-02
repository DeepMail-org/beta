//! HTTP handler for email file upload.
//!
//! Endpoint: POST /api/v1/upload
//!
//! Required headers (set by gateway after JWT validation):
//!   X-DeepMail-User-Id:   <user UUID>
//!   X-DeepMail-Tenant-Id: <tenant UUID>
//!
//! Request body: multipart/form-data with field name "file"
//!
//! Response 202 Accepted:
//!   { "email_id": "<uuid>", "status": "queued" }
//!
//! Response 400 Bad Request:
//!   { "error": "<reason>", "code": "VALIDATION_ERROR" }

use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::{Multipart, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use deepmail_common::nats::{publish_envelope, NatsEnvelope};

use crate::{
    config::Config,
    db,
    error::IngestError,
    hash::compute_hashes,
    s3::upload_quarantine,
    validate::validate_upload,
};

/// Shared application state injected into every handler.
pub struct AppState {
    pub pool: Arc<sqlx::PgPool>,
    pub config: Arc<Config>,
    pub s3_client: Arc<aws_sdk_s3::Client>,
    pub nats_js: Arc<async_nats::jetstream::Context>,
}

/// Upload response body.
#[derive(serde::Serialize)]
struct UploadResponse {
    email_id: String,
    status: String,
}

/// Extract a required UUID from a request header.
fn extract_uuid_header(
    headers: &HeaderMap,
    header_name: &'static str,
) -> Result<Uuid, IngestError> {
    let raw = headers
        .get(header_name)
        .ok_or_else(|| IngestError::MissingHeader(header_name.to_string()))?
        .to_str()
        .map_err(|_| IngestError::MissingHeader(header_name.to_string()))?;

    Uuid::parse_str(raw).map_err(|_| IngestError::InvalidHeaderUuid {
        header: header_name.to_string(),
        value: raw.to_string(),
    })
}

/// POST /api/v1/upload
pub async fn upload_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, IngestError> {
    // Extract gateway-injected identity headers
    let user_id = extract_uuid_header(&headers, "X-DeepMail-User-Id")?;
    let tenant_id = extract_uuid_header(&headers, "X-DeepMail-Tenant-Id")?;

    // Extract file field from multipart
    let mut original_filename = String::new();
    let mut content_type = String::from("application/octet-stream");
    let mut file_bytes: Option<Bytes> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| IngestError::Multipart(e.to_string()))?
    {
        if field.name() == Some("file") {
            original_filename = field
                .file_name()
                .unwrap_or("upload")
                .to_string();
            if let Some(ct) = field.content_type() {
                content_type = ct.to_string();
            }
            file_bytes = Some(
                field
                    .bytes()
                    .await
                    .map_err(|e| IngestError::Multipart(e.to_string()))?,
            );
            break;
        }
    }

    let file_bytes = file_bytes
        .ok_or_else(|| IngestError::Multipart("no 'file' field in multipart body".into()))?;

    let max_bytes = state.config.max_upload_size_mb * 1024 * 1024;

    // ── 5-step validation ────────────────────────────────────────
    let validation_result = validate_upload(
        &original_filename,
        &content_type,
        &file_bytes,
        max_bytes,
    );

    match validation_result {
        Err((ingest_err, steps)) => {
            // Log all partial validation steps without an email_id.
            // We need to create a rejected email row first so we
            // have an email_id to attach validations to.
            // For rejected files we create a minimal email row with
            // status='rejected' immediately, then log the steps.
            let rejection_reason = ingest_err.to_string();

            // Insert a rejected email row (no S3 upload — file was rejected)
            // We still need the quarantine_name to be unique. Use a UUID.
            let reject_uuid = Uuid::new_v4().to_string();
            let ext = if original_filename.to_lowercase().ends_with(".eml") {
                ".eml"
            } else {
                ".msg"
            };
            let quarantine_name = format!("{reject_uuid}{ext}");

            let email_id = sqlx::query!(
                r#"
                INSERT INTO emails (
                  tenant_id, uploaded_by, original_filename,
                  quarantine_name, s3_bucket, s3_key,
                  sha256_hash, md5_hash, file_size_bytes,
                  file_extension, mime_type, magic_bytes_valid,
                  status, rejection_reason
                )
                VALUES (
                  $1, $2, $3,
                  $4, $5, $6,
                  $7, $8, $9,
                  $10, $11, $12,
                  'rejected', $13
                )
                RETURNING id
                "#,
                tenant_id,
                user_id,
                &original_filename,
                &quarantine_name,
                &state.config.s3_bucket,
                "",          // no S3 key for rejected files
                "",          // no SHA-256 for rejected files
                "",          // no MD5 for rejected files
                file_bytes.len() as i64,
                ext,
                &content_type,
                false,       // magic_bytes_valid = false for rejected
                &rejection_reason,
            )
            .fetch_one(state.pool.as_ref())
            .await
            .map_err(IngestError::Database)?
            .id;

            // Log validation steps
            for step in &steps {
                let _ = db::validations::insert_validation_step(
                    &state.pool,
                    email_id,
                    step.step,
                    step.passed,
                    step.detail.as_deref(),
                )
                .await;
            }

            return Err(ingest_err);
        }

        Ok(output) => {
            // ── Compute hashes ────────────────────────────────────
            let (sha256, md5) = compute_hashes(&file_bytes).await;

            let quarantine_name =
                format!("{}{}", output.quarantine_uuid, output.extension);

            // ── S3 upload ─────────────────────────────────────────
            let s3_key = upload_quarantine(
                &state.s3_client,
                &state.config.s3_bucket,
                tenant_id,
                &quarantine_name,
                &original_filename,
                user_id,
                &content_type,
                file_bytes.to_vec(),
            )
            .await?;

            // ── Insert email record ───────────────────────────────
            let email_id = db::emails::insert_email(
                &state.pool,
                tenant_id,
                user_id,
                &original_filename,
                &quarantine_name,
                &state.config.s3_bucket,
                &s3_key,
                &sha256,
                &md5,
                file_bytes.len() as i64,
                &output.extension,
                &content_type,
                true, // magic_bytes_valid = true (passed step 3)
                None, // nats_message_id set after publish
            )
            .await?;

            // ── Log all validation steps ──────────────────────────
            for step in &output.steps {
                let _ = db::validations::insert_validation_step(
                    &state.pool,
                    email_id,
                    step.step,
                    step.passed,
                    step.detail.as_deref(),
                )
                .await;
            }

            // ── Insert job_progress rows ──────────────────────────
            db::progress::insert_all_progress_rows(
                &state.pool,
                email_id,
                tenant_id,
            )
            .await?;

            // ── Publish to NATS ───────────────────────────────────
            let envelope = NatsEnvelope::new(
                email_id,
                tenant_id,
                user_id,
                uuid::Uuid::new_v4().to_string(),
                serde_json::json!({
                    "email_id": email_id,
                    "s3_bucket": state.config.s3_bucket,
                    "s3_key": s3_key,
                    "sha256": sha256,
                    "extension": output.extension,
                }),
            );

            let ack = publish_envelope(
                &state.nats_js,
                deepmail_common::nats::subjects::JOBS_INGEST,
                &envelope,
            )
            .await
            .map_err(|e| IngestError::NatsPublish(e.to_string()))?;

            // Update nats_message_id after successful publish
            let nats_msg_id = ack.sequence.to_string();
            let _ = db::emails::set_nats_message_id(
                &state.pool,
                email_id,
                &nats_msg_id,
            )
            .await;

            tracing::info!(
                email_id = %email_id,
                tenant_id = %tenant_id,
                sha256 = %sha256,
                s3_key = %s3_key,
                "email uploaded and queued successfully"
            );

            Ok((
                StatusCode::ACCEPTED,
                Json(UploadResponse {
                    email_id: email_id.to_string(),
                    status: "queued".to_string(),
                }),
            ))
        }
    }
}
