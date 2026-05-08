/// Error types for deepmail-homograph.

use tonic::Status;

#[derive(Debug, thiserror::Error)]
pub enum HomographError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),

    #[error("internal error: {0}")]
    Internal(String),

    #[error("not found: {0}")]
    NotFound(String),
}

impl HomographError {
    /// Whether this error is transient and can be retried.
    pub fn is_transient(&self) -> bool {
        matches!(self, HomographError::Db(_))
    }
}

impl From<HomographError> for Status {
    fn from(e: HomographError) -> Status {
        match e {
            HomographError::NotFound(msg) => Status::not_found(msg),
            HomographError::Db(e) => Status::internal(format!("db: {e}")),
            HomographError::Internal(msg) => Status::internal(msg),
        }
    }
}
