use thiserror::Error;

#[derive(Error, Debug)]
pub enum GeoError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("DNS error: {0}")]
    Dns(String),

    #[error("NATS error: {0}")]
    Nats(String),

    #[error("malformed NATS envelope: {0}")]
    MalformedEnvelope(String),

    #[error("email not found: {0}")]
    EmailNotFound(uuid::Uuid),

    #[error("Redis error: {0}")]
    Redis(String),

    #[error("MaxMind DB error: {0}")]
    Mmdb(String),

    #[error("HTTP request error: {0}")]
    Http(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl GeoError {
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            GeoError::Database(_)
                | GeoError::Dns(_)
                | GeoError::Nats(_)
                | GeoError::Redis(_)
                | GeoError::Http(_)
        )
    }
}

impl From<redis::RedisError> for GeoError {
    fn from(e: redis::RedisError) -> Self {
        GeoError::Redis(e.to_string())
    }
}

impl From<GeoError> for tonic::Status {
    fn from(e: GeoError) -> Self {
        match e {
            GeoError::Database(ref inner) => {
                tracing::error!(error = %inner, "database error in geo service");
                tonic::Status::internal("internal server error")
            }
            GeoError::EmailNotFound(id) => {
                tonic::Status::not_found(format!("email {id} not found"))
            }
            GeoError::Dns(ref msg) | GeoError::Redis(ref msg) => {
                tracing::warn!(error = %msg, "transient error in geo service");
                tonic::Status::unavailable("service temporarily unavailable")
            }
            GeoError::MalformedEnvelope(ref msg) => {
                tonic::Status::invalid_argument(format!("bad envelope: {msg}"))
            }
            GeoError::Mmdb(ref msg) => {
                tracing::warn!(error = %msg, "mmdb error in geo service");
                tonic::Status::internal("geoip lookup error")
            }
            GeoError::Http(ref msg) => {
                tracing::warn!(error = %msg, "http enrichment error in geo service");
                tonic::Status::internal("enrichment error")
            }
            GeoError::Nats(ref msg) | GeoError::Internal(ref msg) => {
                tracing::error!(error = %msg, "internal error in geo service");
                tonic::Status::internal("internal server error")
            }
        }
    }
}
