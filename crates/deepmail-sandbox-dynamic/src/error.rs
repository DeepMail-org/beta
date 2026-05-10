/// Error types for dynamic sandbox.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DynamicError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),

    #[error("CAPEv2 unavailable")]
    CapeUnavailable,

    #[error("CAPEv2 API error: HTTP {0}: {1}")]
    CapeApiError(u16, String),

    #[error("CAPEv2 task not found: {0}")]
    TaskNotFound(u64),

    #[error("CAPEv2 timeout waiting for task {0}")]
    CapeTimeout(u64),

    #[error("S3 error: {0}")]
    S3(String),

    #[error("NATS error: {0}")]
    Nats(String),

    #[error("payload parse error: {0}")]
    PayloadParse(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("internal error: {0}")]
    Internal(String),
}

impl DynamicError {
    /// Returns true if this is a transient error worth retrying.
    pub fn is_transient(&self) -> bool {
        matches!(
            self,
            DynamicError::CapeUnavailable
                | DynamicError::Db(_)
                | DynamicError::S3(_)
                | DynamicError::Nats(_)
        )
    }
}

impl From<DynamicError> for tonic::Status {
    fn from(e: DynamicError) -> Self {
        match &e {
            DynamicError::TaskNotFound(_) => tonic::Status::not_found(e.to_string()),
            DynamicError::PayloadParse(_) => tonic::Status::invalid_argument(e.to_string()),
            DynamicError::CapeUnavailable => tonic::Status::unavailable(e.to_string()),
            _ => tonic::Status::internal(e.to_string()),
        }
    }
}
