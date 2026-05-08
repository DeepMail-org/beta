use thiserror::Error;

#[derive(Error, Debug)]
pub enum IpError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("NATS error: {0}")]
    Nats(String),

    #[error("malformed NATS envelope: {0}")]
    MalformedEnvelope(String),

    #[error("email not found: {0}")]
    EmailNotFound(uuid::Uuid),

    #[error("Redis error: {0}")]
    Redis(String),

    #[error("HTTP request error: {0}")]
    Http(String),

    #[error("parse error: {0}")]
    Parse(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl IpError {
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            IpError::Database(_)
                | IpError::Nats(_)
                | IpError::Redis(_)
                | IpError::Http(_)
        )
    }
}

impl From<redis::RedisError> for IpError {
    fn from(e: redis::RedisError) -> Self {
        IpError::Redis(e.to_string())
    }
}

impl From<reqwest::Error> for IpError {
    fn from(e: reqwest::Error) -> Self {
        IpError::Http(e.to_string())
    }
}

impl From<IpError> for tonic::Status {
    fn from(e: IpError) -> Self {
        match e {
            IpError::Database(ref inner) => {
                tracing::error!(error = %inner, "database error in ip service");
                tonic::Status::internal("internal server error")
            }
            IpError::EmailNotFound(id) => {
                tonic::Status::not_found(format!("email {id} not found"))
            }
            IpError::Nats(ref msg) | IpError::Redis(ref msg) => {
                tracing::warn!(error = %msg, "transient error in ip service");
                tonic::Status::unavailable("service temporarily unavailable")
            }
            IpError::MalformedEnvelope(ref msg) => {
                tonic::Status::invalid_argument(format!("bad envelope: {msg}"))
            }
            IpError::Http(ref msg) => {
                tracing::warn!(error = %msg, "http error in ip service");
                tonic::Status::internal("enrichment error")
            }
            IpError::Parse(ref msg) => {
                tracing::warn!(error = %msg, "parse error in ip service");
                tonic::Status::internal("parse error")
            }
            IpError::Internal(ref msg) => {
                tracing::error!(error = %msg, "internal error in ip service");
                tonic::Status::internal("internal server error")
            }
        }
    }
}
