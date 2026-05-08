/// Error types for deepmail-ioc.

use tonic::Status;

#[derive(Debug, thiserror::Error)]
pub enum IocError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),

    #[error("parse error: {0}")]
    Parse(String),

    #[error("grpc error: {0}")]
    Grpc(String),

    #[error("internal error: {0}")]
    Internal(String),

    #[error("not found: {0}")]
    NotFound(String),
}

impl IocError {
    /// Whether this error is transient and can be retried.
    pub fn is_transient(&self) -> bool {
        matches!(self, IocError::Db(_) | IocError::Grpc(_))
    }
}

impl From<IocError> for Status {
    fn from(e: IocError) -> Status {
        match e {
            IocError::NotFound(msg) => Status::not_found(msg),
            IocError::Parse(msg) => Status::invalid_argument(msg),
            IocError::Db(e) => Status::internal(format!("db: {e}")),
            IocError::Grpc(msg) => Status::unavailable(msg),
            IocError::Internal(msg) => Status::internal(msg),
        }
    }
}
