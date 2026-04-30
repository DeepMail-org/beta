//! Shared error type used across every DeepMail microservice.
//!
//! Services should bubble these up via the `?` operator. Conversion to
//! `tonic::Status` is provided so handlers can `?` straight to a gRPC
//! response without scattering match arms in every endpoint.

use thiserror::Error;

/// Unified error type for DeepMail services.
#[derive(Error, Debug)]
pub enum DeepMailError {
    /// Database (PostgreSQL / sqlx) failure: connection, query, or migration.
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    /// NATS (core or JetStream) transport / publish / subscribe failure.
    #[error("nats error: {0}")]
    Nats(String),

    /// JetStream-specific publish or stream-management failure.
    #[error("jetstream error: {0}")]
    JetStream(String),

    /// Redis connection or command failure.
    #[error("redis error: {0}")]
    Redis(#[from] redis::RedisError),

    /// Input failed validation (malformed UUID, bad email, missing field, etc).
    #[error("validation error: {0}")]
    Validation(String),

    /// Requested resource does not exist (email, tenant, user, report, …).
    #[error("not found: {0}")]
    NotFound(String),

    /// Caller is not authenticated, or token is invalid / expired / revoked.
    #[error("unauthorized: {0}")]
    Unauthorized(String),

    /// Caller is authenticated but lacks permission for this operation.
    #[error("forbidden: {0}")]
    Forbidden(String),

    /// Caller has exceeded a per-tenant or per-user rate limit / quota.
    #[error("rate limited: {0}")]
    RateLimited(String),

    /// Configuration loading or schema validation failure.
    #[error("config error: {0}")]
    Config(#[from] config::ConfigError),

    /// JSON or proto-level (de)serialization failure.
    #[error("serialization error: {0}")]
    Serialization(String),

    /// Failure invoking an upstream HTTP threat-intel / sandbox / billing API.
    #[error("external provider '{provider}' error: {message}")]
    ExternalProvider {
        /// Name of the upstream service (e.g. "VirusTotal", "AbuseIPDB").
        provider: String,
        /// Human-readable failure description.
        message: String,
    },

    /// gRPC call to a sibling DeepMail service failed at the transport layer.
    #[error("grpc transport error: {0}")]
    GrpcTransport(String),

    /// gRPC call to a sibling DeepMail service returned a non-OK status.
    #[error("grpc status: {0}")]
    GrpcStatus(#[from] tonic::Status),

    /// Background task panicked or was cancelled.
    #[error("task join error: {0}")]
    TaskJoin(#[from] tokio::task::JoinError),

    /// Catch-all for unexpected internal failures.
    #[error("internal error: {0}")]
    Internal(String),
}

impl From<serde_json::Error> for DeepMailError {
    fn from(err: serde_json::Error) -> Self {
        DeepMailError::Serialization(err.to_string())
    }
}

impl From<async_nats::ConnectError> for DeepMailError {
    fn from(err: async_nats::ConnectError) -> Self {
        DeepMailError::Nats(err.to_string())
    }
}

impl From<async_nats::jetstream::context::PublishError> for DeepMailError {
    fn from(err: async_nats::jetstream::context::PublishError) -> Self {
        DeepMailError::JetStream(err.to_string())
    }
}

impl From<async_nats::SubscribeError> for DeepMailError {
    fn from(err: async_nats::SubscribeError) -> Self {
        DeepMailError::Nats(err.to_string())
    }
}

impl From<async_nats::PublishError> for DeepMailError {
    fn from(err: async_nats::PublishError) -> Self {
        DeepMailError::Nats(err.to_string())
    }
}

impl From<tonic::transport::Error> for DeepMailError {
    fn from(err: tonic::transport::Error) -> Self {
        DeepMailError::GrpcTransport(err.to_string())
    }
}

impl From<uuid::Error> for DeepMailError {
    fn from(err: uuid::Error) -> Self {
        DeepMailError::Validation(format!("invalid uuid: {err}"))
    }
}

impl From<DeepMailError> for tonic::Status {
    fn from(err: DeepMailError) -> Self {
        use tonic::Code;
        let (code, msg) = match &err {
            DeepMailError::Validation(m) => (Code::InvalidArgument, m.clone()),
            DeepMailError::NotFound(m) => (Code::NotFound, m.clone()),
            DeepMailError::Unauthorized(m) => (Code::Unauthenticated, m.clone()),
            DeepMailError::Forbidden(m) => (Code::PermissionDenied, m.clone()),
            DeepMailError::RateLimited(m) => (Code::ResourceExhausted, m.clone()),
            DeepMailError::Config(_) => (Code::FailedPrecondition, err.to_string()),
            DeepMailError::Serialization(m) => (Code::InvalidArgument, m.clone()),
            DeepMailError::ExternalProvider { .. } => (Code::Unavailable, err.to_string()),
            DeepMailError::GrpcTransport(m) => (Code::Unavailable, m.clone()),
            DeepMailError::GrpcStatus(s) => return s.clone(),
            DeepMailError::Database(_)
            | DeepMailError::Nats(_)
            | DeepMailError::JetStream(_)
            | DeepMailError::Redis(_)
            | DeepMailError::TaskJoin(_)
            | DeepMailError::Internal(_) => (Code::Internal, err.to_string()),
        };
        tonic::Status::new(code, msg)
    }
}

/// Convenience alias: `Result<T, DeepMailError>`.
pub type Result<T> = std::result::Result<T, DeepMailError>;
