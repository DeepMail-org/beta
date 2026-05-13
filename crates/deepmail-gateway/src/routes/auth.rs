use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::json;

use deepmail_common::proto::auth;
use crate::error::GatewayError;
use crate::GatewayCtx;

#[derive(Deserialize)]
pub struct RegisterBody {
    pub username: String,
    pub email: String,
    pub password: String,
}

pub async fn register(
    State(ctx): State<Arc<GatewayCtx>>,
    Json(body): Json<RegisterBody>,
) -> Result<impl IntoResponse, GatewayError> {
    let resp = ctx
        .auth_client
        .register(auth::RegisterRequest {
            username: body.username,
            email: body.email,
            password: body.password,
        })
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "user_id": resp.user_id,
            "message": resp.message
        })),
    ))
}

#[derive(Deserialize)]
pub struct LoginBody {
    pub email: String,
    pub password: String,
}

pub async fn login(
    State(ctx): State<Arc<GatewayCtx>>,
    Json(body): Json<LoginBody>,
) -> Result<impl IntoResponse, GatewayError> {
    let resp = ctx
        .auth_client
        .login(auth::LoginRequest {
            email: body.email,
            password: body.password,
        })
        .await?;

    Ok(Json(json!({
        "otp_required": resp.otp_required,
        "session_token": resp.session_token
    })))
}

#[derive(Deserialize)]
pub struct VerifyOtpBody {
    pub session_token: String,
    pub otp_code: String,
}

pub async fn verify_otp(
    State(ctx): State<Arc<GatewayCtx>>,
    Json(body): Json<VerifyOtpBody>,
) -> Result<impl IntoResponse, GatewayError> {
    let resp = ctx
        .auth_client
        .verify_otp(auth::VerifyOtpRequest {
            session_token: body.session_token,
            otp_code: body.otp_code,
        })
        .await?;

    Ok(Json(json!({
        "access_token": resp.access_token,
        "refresh_token": resp.refresh_token,
        "expires_in": resp.expires_in
    })))
}

#[derive(Deserialize)]
pub struct RefreshBody {
    pub refresh_token: String,
}

pub async fn refresh(
    State(ctx): State<Arc<GatewayCtx>>,
    Json(body): Json<RefreshBody>,
) -> Result<impl IntoResponse, GatewayError> {
    let resp = ctx
        .auth_client
        .refresh_token(auth::RefreshTokenRequest {
            refresh_token: body.refresh_token,
        })
        .await?;

    Ok(Json(json!({
        "access_token": resp.access_token,
        "expires_in": resp.expires_in
    })))
}
