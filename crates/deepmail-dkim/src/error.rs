use thiserror::Error;

#[derive(Error, Debug)]
pub enum DkimError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("DNS resolution error: {0}")]
    Dns(String),

    #[error("NATS error: {0}")]
    Nats(String),

    #[error("malformed NATS envelope: {0}")]
    MalformedEnvelope(String),

    #[error("email not found: {0}")]
    EmailNotFound(uuid::Uuid),

    #[error("DKIM parse error: {0}")]
    Parse(String),

    #[error("crypto verification error: {0}")]
    Crypto(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl DkimError {
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            DkimError::Database(_) | DkimError::Dns(_) | DkimError::Nats(_)
        )
    }
}

impl From<DkimError> for tonic::Status {
    fn from(e: DkimError) -> Self {
        match e {
            DkimError::Database(ref inner) => {
                tracing::error!(error = %inner, "database error in dkim service");
                tonic::Status::internal("internal server error")
            }
            DkimError::EmailNotFound(id) => {
                tonic::Status::not_found(format!("email {id} not found"))
            }
            DkimError::Dns(ref msg) => {
                tracing::warn!(error = %msg, "DNS error in dkim service");
                tonic::Status::unavailable("DNS resolver unavailable")
            }
            DkimError::MalformedEnvelope(ref msg) => {
                tonic::Status::invalid_argument(format!("bad envelope: {msg}"))
            }
            DkimError::Parse(ref msg) => {
                tonic::Status::invalid_argument(format!("DKIM parse error: {msg}"))
            }
            DkimError::Crypto(ref msg) => {
                tracing::warn!(error = %msg, "crypto error in dkim service");
                tonic::Status::internal("verification error")
            }
            DkimError::Nats(ref msg) | DkimError::Internal(ref msg) => {
                tracing::error!(error = %msg, "internal error in dkim service");
                tonic::Status::internal("internal server error")
            }
        }
    }
}
