use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReportError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),

    #[error("S3 upload error: {0}")]
    S3Upload(String),

    #[error("S3 download error: {0}")]
    S3Download(String),

    #[error("render error: {0}")]
    RenderError(String),

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

impl ReportError {
    pub fn is_transient(&self) -> bool {
        matches!(
            self,
            ReportError::Db(_)
                | ReportError::S3Upload(_)
                | ReportError::S3Download(_)
                | ReportError::Nats(_)
        )
    }
}

impl From<ReportError> for tonic::Status {
    fn from(e: ReportError) -> Self {
        match &e {
            ReportError::NotFound(_) => tonic::Status::not_found(e.to_string()),
            ReportError::InvalidArgument(_) => tonic::Status::invalid_argument(e.to_string()),
            ReportError::PayloadParse(_) => tonic::Status::invalid_argument(e.to_string()),
            _ => tonic::Status::internal(e.to_string()),
        }
    }
}
