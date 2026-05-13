use thiserror::Error;

#[derive(Debug, Error)]
pub enum BillingError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),

    #[error("NATS error: {0}")]
    Nats(String),

    #[error("Razorpay error: {0}")]
    RazorpayError(String),

    #[error("Razorpay not configured")]
    NotConfigured,

    #[error("payload parse error: {0}")]
    PayloadParse(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl BillingError {
    pub fn is_transient(&self) -> bool {
        matches!(
            self,
            BillingError::Db(_) | BillingError::Nats(_) | BillingError::RazorpayError(_)
        )
    }
}

impl From<BillingError> for tonic::Status {
    fn from(e: BillingError) -> Self {
        match &e {
            BillingError::NotFound(_) => tonic::Status::not_found(e.to_string()),
            BillingError::InvalidArgument(_) => tonic::Status::invalid_argument(e.to_string()),
            BillingError::PayloadParse(_) => tonic::Status::invalid_argument(e.to_string()),
            BillingError::NotConfigured => tonic::Status::failed_precondition(e.to_string()),
            _ => tonic::Status::internal(e.to_string()),
        }
    }
}
