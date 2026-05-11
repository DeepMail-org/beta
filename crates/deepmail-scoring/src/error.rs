use thiserror::Error;

#[derive(Debug, Error)]
pub enum ScoringError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),

    #[error("NATS error: {0}")]
    Nats(String),

    #[error("payload parse error: {0}")]
    PayloadParse(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl ScoringError {
    pub fn is_transient(&self) -> bool {
        matches!(self, ScoringError::Db(_) | ScoringError::Nats(_))
    }
}

impl From<ScoringError> for tonic::Status {
    fn from(e: ScoringError) -> Self {
        match &e {
            ScoringError::NotFound(_) => tonic::Status::not_found(e.to_string()),
            ScoringError::InvalidArgument(_) => tonic::Status::invalid_argument(e.to_string()),
            ScoringError::PayloadParse(_) => tonic::Status::invalid_argument(e.to_string()),
            _ => tonic::Status::internal(e.to_string()),
        }
    }
}
