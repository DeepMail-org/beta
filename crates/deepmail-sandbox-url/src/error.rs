/// Error types for deepmail-sandbox-url.

#[derive(Debug, thiserror::Error)]
pub enum SandboxUrlError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),

    #[error("NATS error: {0}")]
    Nats(String),

    #[error("Docker error: {0}")]
    Docker(String),

    #[error("container timeout after {0}s")]
    Timeout(u64),

    #[error("no results from container")]
    NoResults,

    #[error("S3 upload error: {0}")]
    S3(String),

    #[error("invalid payload: {0}")]
    InvalidPayload(String),

    #[error("QR decode error: {0}")]
    QrDecode(String),

    #[error("HTTP fetch error: {0}")]
    HttpFetch(String),

    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

impl From<SandboxUrlError> for tonic::Status {
    fn from(e: SandboxUrlError) -> Self {
        match e {
            SandboxUrlError::Db(_) => tonic::Status::internal(e.to_string()),
            SandboxUrlError::InvalidPayload(msg) => tonic::Status::invalid_argument(msg),
            SandboxUrlError::NoResults => tonic::Status::not_found(e.to_string()),
            _ => tonic::Status::internal(e.to_string()),
        }
    }
}
