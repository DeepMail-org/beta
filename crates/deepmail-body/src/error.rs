/// Error types for deepmail-body.

use tonic::Status;

#[derive(Debug, thiserror::Error)]
pub enum BodyError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),

    #[error("internal error: {0}")]
    Internal(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("ML service unavailable: {0}")]
    MlUnavailable(String),
}

impl BodyError {
    pub fn is_transient(&self) -> bool {
        matches!(self, BodyError::Db(_) | BodyError::MlUnavailable(_))
    }
}

impl From<BodyError> for Status {
    fn from(e: BodyError) -> Status {
        match e {
            BodyError::NotFound(msg) => Status::not_found(msg),
            BodyError::Db(e) => Status::internal(format!("db: {e}")),
            BodyError::Internal(msg) => Status::internal(msg),
            BodyError::MlUnavailable(msg) => Status::unavailable(msg),
        }
    }
}
