//! Error types for deepmail-ingest.

use axum::{http::StatusCode, response::IntoResponse, Json};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IngestError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("validation failed at step '{step}': {reason}")]
    ValidationFailed { step: String, reason: String },

    #[error("file extension '{0}' is not allowed — only .eml and .msg")]
    DisallowedExtension(String),

    #[error("file size {size_bytes} bytes exceeds limit of {limit_bytes} bytes")]
    FileTooLarge { size_bytes: u64, limit_bytes: u64 },

    #[error("magic bytes do not match declared file type")]
    MagicBytesMismatch,

    #[error("MIME type '{0}' is not allowed")]
    DisallowedMimeType(String),

    #[error("path traversal detected in filename")]
    PathTraversal,

    #[error("S3 upload error: {0}")]
    S3Upload(String),

    #[error("NATS publish error: {0}")]
    NatsPublish(String),

    #[error("missing required header: {0}")]
    MissingHeader(String),

    #[error("invalid UUID in header '{header}': {value}")]
    InvalidHeaderUuid { header: String, value: String },

    #[error("multipart error: {0}")]
    Multipart(String),

    #[error("internal error: {0}")]
    Internal(String),
}

/// HTTP error response body.
#[derive(serde::Serialize)]
struct ErrorBody {
    error: String,
    code: String,
}

impl IntoResponse for IngestError {
    fn into_response(self) -> axum::response::Response {
        let (status, code) = match &self {
            IngestError::DisallowedExtension(_)
            | IngestError::FileTooLarge { .. }
            | IngestError::MagicBytesMismatch
            | IngestError::DisallowedMimeType(_)
            | IngestError::PathTraversal
            | IngestError::ValidationFailed { .. }
            | IngestError::MissingHeader(_)
            | IngestError::InvalidHeaderUuid { .. }
            | IngestError::Multipart(_) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR"),

            IngestError::S3Upload(_)
            | IngestError::NatsPublish(_)
            | IngestError::Database(_)
            | IngestError::Internal(_) => {
                tracing::error!(error = %self, "internal ingest error");
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR")
            }
        };

        (
            status,
            Json(ErrorBody {
                error: self.to_string(),
                code: code.to_string(),
            }),
        )
            .into_response()
    }
}
