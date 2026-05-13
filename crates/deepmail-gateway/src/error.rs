use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum GatewayError {
    #[error("gRPC error: {0}")]
    Grpc(#[from] tonic::Status),
    #[error("redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("request error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("validation: {0}")]
    Validation(String),
}

impl IntoResponse for GatewayError {
    fn into_response(self) -> Response {
        let (status, msg) = match &self {
            GatewayError::Grpc(s) => match s.code() {
                tonic::Code::NotFound => (StatusCode::NOT_FOUND, s.message().to_string()),
                tonic::Code::Unauthenticated => {
                    (StatusCode::UNAUTHORIZED, "unauthorized".to_string())
                }
                tonic::Code::AlreadyExists => (StatusCode::CONFLICT, s.message().to_string()),
                tonic::Code::InvalidArgument => {
                    (StatusCode::BAD_REQUEST, s.message().to_string())
                }
                tonic::Code::ResourceExhausted => {
                    (StatusCode::TOO_MANY_REQUESTS, s.message().to_string())
                }
                _ => (StatusCode::BAD_GATEWAY, "upstream error".to_string()),
            },
            GatewayError::Redis(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal error".to_string())
            }
            GatewayError::Db(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal error".to_string())
            }
            GatewayError::Request(_) => (StatusCode::BAD_GATEWAY, "upstream error".to_string()),
            GatewayError::Validation(m) => (StatusCode::BAD_REQUEST, m.clone()),
        };
        (status, Json(json!({"error": msg}))).into_response()
    }
}
