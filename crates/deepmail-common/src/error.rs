use tonic::Status;

/// Unified error type for all DeepMail services.
///
/// Each variant maps to a specific `tonic::Status` code for gRPC responses.
/// Services should use this as their internal error type and let the
/// `From<DeepMailError> for Status` conversion handle gRPC mapping.
#[derive(Debug, thiserror::Error)]
pub enum DeepMailError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("NATS error: {0}")]
    Nats(String),

    #[error("configuration error: {0}")]
    Config(String),

    #[error("authentication failed: {0}")]
    Auth(String),

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("resource not found: {0}")]
    NotFound(String),

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("internal error: {0}")]
    Internal(String),

    #[error("external API error: {0}")]
    ExternalApi(String),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("service unavailable: {0}")]
    Unavailable(String),
}

impl From<DeepMailError> for Status {
    fn from(err: DeepMailError) -> Self {
        match &err {
            DeepMailError::Database(_) => Status::internal(err.to_string()),
            DeepMailError::Nats(_) => Status::internal(err.to_string()),
            DeepMailError::Config(_) => Status::internal(err.to_string()),
            DeepMailError::Auth(_) => Status::unauthenticated(err.to_string()),
            DeepMailError::PermissionDenied(_) => Status::permission_denied(err.to_string()),
            DeepMailError::NotFound(_) => Status::not_found(err.to_string()),
            DeepMailError::InvalidInput(_) => Status::invalid_argument(err.to_string()),
            DeepMailError::Internal(_) => Status::internal(err.to_string()),
            DeepMailError::ExternalApi(_) => Status::unavailable(err.to_string()),
            DeepMailError::Serialization(_) => Status::internal(err.to_string()),
            DeepMailError::Unavailable(_) => Status::unavailable(err.to_string()),
        }
    }
}

impl From<serde_json::Error> for DeepMailError {
    fn from(err: serde_json::Error) -> Self {
        DeepMailError::Serialization(err.to_string())
    }
}

impl From<async_nats::Error> for DeepMailError {
    fn from(err: async_nats::Error) -> Self {
        DeepMailError::Nats(err.to_string())
    }
}

impl From<config::ConfigError> for DeepMailError {
    fn from(err: config::ConfigError) -> Self {
        DeepMailError::Config(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn database_error_maps_to_internal() {
        let err = DeepMailError::Internal("test".into());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::Internal);
    }

    #[test]
    fn auth_error_maps_to_unauthenticated() {
        let err = DeepMailError::Auth("bad token".into());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::Unauthenticated);
    }

    #[test]
    fn not_found_maps_correctly() {
        let err = DeepMailError::NotFound("user 123".into());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::NotFound);
    }

    #[test]
    fn invalid_input_maps_to_invalid_argument() {
        let err = DeepMailError::InvalidInput("bad email".into());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::InvalidArgument);
    }
}
