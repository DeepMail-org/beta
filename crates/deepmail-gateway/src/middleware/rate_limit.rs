use std::sync::Arc;

use axum::{
    body::Body,
    extract::{ConnectInfo, State},
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::net::SocketAddr;

use crate::GatewayCtx;
use deepmail_common::zig_ratelimiter::auth::{check_login_ip, check_login_email, check_otp_email, check_reset_ip};

/// Rate limiting middleware for auth endpoints
pub async fn auth_rate_limit_middleware(
    State(ctx): State<Arc<GatewayCtx>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    let ip = addr.ip().to_string();
    
    // Extract email from request body if present
    let email = extract_email_from_body(&req).unwrap_or_default();
    
    // Check rate limits based on endpoint
    let path = req.uri().path();
    
    match path {
        "/auth/login" => {
            // Check both IP and email limits
            if !check_login_ip(&ip) {
                return (
                    StatusCode::TOO_MANY_REQUESTS,
                    Json(json!({"error": "too many login attempts from this IP, try again later"})),
                ).into_response();
            }
            if !email.is_empty() && !check_login_email(&email) {
                return (
                    StatusCode::TOO_MANY_REQUESTS,
                    Json(json!({"error": "too many login attempts for this email, try again later"})),
                ).into_response();
            }
        }
        "/auth/register" => {
            // Check IP limit for registration
            if !check_login_ip(&ip) {
                return (
                    StatusCode::TOO_MANY_REQUESTS,
                    Json(json!({"error": "too many registration attempts from this IP, try again later"})),
                ).into_response();
            }
        }
        "/auth/verify-otp" => {
            // Check OTP limit
            if !email.is_empty() && !check_otp_email(&email) {
                return (
                    StatusCode::TOO_MANY_REQUESTS,
                    Json(json!({"error": "too many OTP requests for this email, try again later"})),
                ).into_response();
            }
        }
        "/auth/refresh" => {
            // Check IP limit for token refresh
            if !check_login_ip(&ip) {
                return (
                    StatusCode::TOO_MANY_REQUESTS,
                    Json(json!({"error": "too many token refresh attempts from this IP, try again later"})),
                ).into_response();
            }
        }
        "/auth/reset" | "/auth/forgot-password" => {
            // Check password reset limit
            if !check_reset_ip(&ip) {
                return (
                    StatusCode::TOO_MANY_REQUESTS,
                    Json(json!({"error": "too many password reset attempts from this IP, try again later"})),
                ).into_response();
            }
        }
        _ => {}
    }
    
    next.run(req).await
}

/// Extract email from request body (for rate limiting)
fn extract_email_from_body(req: &Request<Body>) -> Option<String> {
    // This is a simplified extraction - in practice you might want to
    // buffer the body and extract the email field
    // For now, we'll use a header if available
    req.headers()
        .get("x-forwarded-email")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}