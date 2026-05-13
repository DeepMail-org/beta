use crate::db::exports;
use crate::s3;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct HttpState {
    pub s3: Arc<aws_sdk_s3::Client>,
    pub bucket: String,
    pub own_pool: Arc<PgPool>,
}

pub fn build_router(state: HttpState) -> Router {
    Router::new()
        .route("/reports/{email_id}/json", get(get_json_report))
        .route("/reports/{email_id}/html", get(get_html_report))
        .route("/health", get(health))
        .with_state(state)
}

async fn health() -> impl IntoResponse {
    (StatusCode::OK, [("content-type", "application/json")], r#"{"status":"ok"}"#)
}

async fn get_json_report(
    State(state): State<HttpState>,
    Path(email_id_str): Path<String>,
) -> Result<Response, AppError> {
    let email_id = Uuid::parse_str(&email_id_str)
        .map_err(|_| AppError::BadRequest("invalid email_id".into()))?;

    let row = exports::get_by_email_format(&state.own_pool, email_id, "json")
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("report not found".into()))?;

    let bytes = s3::download_report(&state.s3, &state.bucket, &row.s3_key)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok((
        StatusCode::OK,
        [
            ("content-type", "application/json"),
            ("content-disposition", "inline"),
        ],
        bytes,
    )
        .into_response())
}

async fn get_html_report(
    State(state): State<HttpState>,
    Path(email_id_str): Path<String>,
) -> Result<Response, AppError> {
    let email_id = Uuid::parse_str(&email_id_str)
        .map_err(|_| AppError::BadRequest("invalid email_id".into()))?;

    let row = exports::get_by_email_format(&state.own_pool, email_id, "html")
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("report not found".into()))?;

    let bytes = s3::download_report(&state.s3, &state.bucket, &row.s3_key)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok((
        StatusCode::OK,
        [
            ("content-type", "text/html; charset=utf-8"),
            ("content-disposition", "inline"),
        ],
        bytes,
    )
        .into_response())
}

enum AppError {
    BadRequest(String),
    NotFound(String),
    Internal(String),
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::Internal(e.to_string())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, m),
            AppError::NotFound(m) => (StatusCode::NOT_FOUND, m),
            AppError::Internal(m) => {
                tracing::error!(error = m.as_str(), "http error");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal error".to_string())
            }
        };

        (
            status,
            [("content-type", "application/json")],
            format!(r#"{{"error":"{}"}}"#, msg.replace('"', r#"\""#)),
        )
            .into_response()
    }
}
