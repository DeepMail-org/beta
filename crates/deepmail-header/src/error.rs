//! Error types for deepmail-header.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum HeaderError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("DNS resolution error: {0}")]
    Dns(String),

    #[error("NATS error: {0}")]
    Nats(String),

    #[error("malformed NATS envelope: {0}")]
    MalformedEnvelope(String),

    #[error("email not found in parser DB: {0}")]
    EmailNotFound(uuid::Uuid),

    #[error("internal error: {0}")]
    Internal(String),
}

impl HeaderError {
    /// True if this error is transient and the message should be NAK'd.
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            HeaderError::Database(_)
                | HeaderError::Dns(_)
                | HeaderError::Nats(_)
                | HeaderError::Internal(_)
        )
    }
}

impl From<HeaderError> for tonic::Status {
    fn from(e: HeaderError) -> Self {
        match e {
            HeaderError::Database(ref inner) => {
                tracing::error!(error = %inner, "database error in header service");
                tonic::Status::internal("internal server error")
            }
            HeaderError::EmailNotFound(id) => {
                tonic::Status::not_found(format!("email {id} not found"))
            }
            HeaderError::Dns(ref msg) => {
                tracing::warn!(error = %msg, "DNS error in header service");
                tonic::Status::unavailable("DNS resolver unavailable")
            }
            HeaderError::MalformedEnvelope(ref msg) => {
                tonic::Status::invalid_argument(format!("bad envelope: {msg}"))
            }
            HeaderError::Nats(ref msg) | HeaderError::Internal(ref msg) => {
                tracing::error!(error = %msg, "internal error in header service");
                tonic::Status::internal("internal server error")
            }
        }
    }
}
