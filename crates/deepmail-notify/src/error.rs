use thiserror::Error;

#[derive(Debug, Error)]
pub enum NotifyError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),

    #[error("NATS error: {0}")]
    Nats(String),

    #[error("SMTP error: {0}")]
    SmtpError(String),

    #[error("SMTP not configured")]
    SmtpNotConfigured,

    #[error("webhook delivery failed: {0}")]
    WebhookFailed(String),

    #[error("auth error: {0}")]
    AuthError(String),

    #[error("payload parse error: {0}")]
    PayloadParse(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl NotifyError {
    pub fn is_transient(&self) -> bool {
        matches!(
            self,
            NotifyError::Db(_)
                | NotifyError::Nats(_)
                | NotifyError::SmtpError(_)
                | NotifyError::WebhookFailed(_)
                | NotifyError::AuthError(_)
        )
    }
}

impl From<NotifyError> for tonic::Status {
    fn from(e: NotifyError) -> Self {
        match &e {
            NotifyError::NotFound(_) => tonic::Status::not_found(e.to_string()),
            NotifyError::InvalidArgument(_) => tonic::Status::invalid_argument(e.to_string()),
            NotifyError::PayloadParse(_) => tonic::Status::invalid_argument(e.to_string()),
            NotifyError::SmtpNotConfigured => {
                tonic::Status::failed_precondition(e.to_string())
            }
            _ => tonic::Status::internal(e.to_string()),
        }
    }
}
