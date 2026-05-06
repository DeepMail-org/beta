//! Error types for deepmail-hashdb.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum HashDbError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("ssdeep error: {0}")]
    Ssdeep(String),

    #[error("invalid SHA-256 hash format: {0}")]
    InvalidHash(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl From<HashDbError> for tonic::Status {
    fn from(e: HashDbError) -> Self {
        match e {
            HashDbError::Database(ref inner) => {
                tracing::error!(error = %inner, "database error in hashdb");
                tonic::Status::internal("internal server error")
            }
            HashDbError::Redis(ref inner) => {
                tracing::warn!(error = %inner, "Redis error in hashdb — degraded mode");
                // Redis errors are non-fatal: we fall through to PostgreSQL
                tonic::Status::internal("internal server error")
            }
            HashDbError::Ssdeep(ref msg) => {
                tracing::warn!(error = %msg, "ssdeep error in hashdb");
                tonic::Status::internal("internal server error")
            }
            HashDbError::InvalidHash(ref msg) => {
                tonic::Status::invalid_argument(
                    format!("invalid hash format: {msg}")
                )
            }
            HashDbError::Internal(ref msg) => {
                tracing::error!(error = %msg, "internal error in hashdb");
                tonic::Status::internal("internal server error")
            }
        }
    }
}
