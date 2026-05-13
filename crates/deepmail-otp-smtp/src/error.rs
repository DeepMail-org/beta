use tonic::Status;

#[derive(Debug, thiserror::Error)]
pub enum OtpSmtpError {
    #[error("smtp address parse error: {0}")]
    Address(String),
    #[error("smtp transport error: {0}")]
    Transport(String),
    #[error("smtp auth configuration missing")]
    MissingAuth,
}

impl From<OtpSmtpError> for Status {
    fn from(value: OtpSmtpError) -> Self {
        match value {
            OtpSmtpError::Address(m) => Status::invalid_argument(m),
            OtpSmtpError::Transport(m) => Status::unavailable(m),
            OtpSmtpError::MissingAuth => Status::failed_precondition("smtp auth configuration missing"),
        }
    }
}
