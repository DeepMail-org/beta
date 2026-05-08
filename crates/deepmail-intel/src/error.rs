use thiserror::Error;

#[derive(Error, Debug)]
pub enum IntelError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("NATS error: {0}")]
    Nats(String),

    #[error("malformed envelope: {0}")]
    MalformedEnvelope(String),

    #[error("Redis error: {0}")]
    Redis(String),

    #[error("HTTP error: {0}")]
    Http(String),

    #[error("parse error: {0}")]
    Parse(String),

    #[error("circuit open for provider: {0}")]
    CircuitOpen(String),

    #[error("rate limited: {0}")]
    RateLimited(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl IntelError {
    pub fn is_transient(&self) -> bool {
        matches!(
            self,
            IntelError::Database(_)
                | IntelError::Nats(_)
                | IntelError::Redis(_)
                | IntelError::Http(_)
                | IntelError::RateLimited(_)
        )
    }
}

impl From<redis::RedisError> for IntelError {
    fn from(e: redis::RedisError) -> Self {
        IntelError::Redis(e.to_string())
    }
}

impl From<reqwest::Error> for IntelError {
    fn from(e: reqwest::Error) -> Self {
        IntelError::Http(e.to_string())
    }
}

impl From<IntelError> for tonic::Status {
    fn from(e: IntelError) -> Self {
        match e {
            IntelError::Database(ref inner) => {
                tracing::error!(error = %inner, "database error in intel service");
                tonic::Status::internal("internal server error")
            }
            IntelError::Nats(ref msg) | IntelError::Redis(ref msg) => {
                tracing::warn!(error = %msg, "transient error in intel service");
                tonic::Status::unavailable("service temporarily unavailable")
            }
            IntelError::MalformedEnvelope(ref msg) => {
                tonic::Status::invalid_argument(format!("bad envelope: {msg}"))
            }
            IntelError::Http(ref msg) => {
                tracing::warn!(error = %msg, "http error in intel service");
                tonic::Status::internal("enrichment error")
            }
            IntelError::Parse(ref msg) => {
                tracing::warn!(error = %msg, "parse error in intel service");
                tonic::Status::internal("parse error")
            }
            IntelError::CircuitOpen(ref provider) => {
                tonic::Status::unavailable(format!("provider {provider} circuit open"))
            }
            IntelError::RateLimited(ref msg) => {
                tonic::Status::resource_exhausted(format!("rate limited: {msg}"))
            }
            IntelError::Internal(ref msg) => {
                tracing::error!(error = %msg, "internal error in intel service");
                tonic::Status::internal("internal server error")
            }
        }
    }
}
