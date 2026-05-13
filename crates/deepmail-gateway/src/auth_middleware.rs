use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use uuid::Uuid;

use crate::GatewayCtx;

#[derive(Clone, Debug)]
pub struct AuthClaims {
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub email: String,
}

/// JWT validation middleware. Only applied to protected route group.
/// Extracts Bearer token, validates via auth gRPC, injects claims, checks rate limit.
pub async fn auth_middleware(
    State(ctx): State<Arc<GatewayCtx>>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    let token = match req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
    {
        Some(t) => t.to_string(),
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "missing authorization"})),
            )
                .into_response();
        }
    };

    let resp = match ctx.auth_client.validate_token(&token).await {
        Ok(r) => r,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "unauthorized"})),
            )
                .into_response();
        }
    };

    if !resp.valid {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "unauthorized"})),
        )
            .into_response();
    }

    let user_id = match Uuid::parse_str(&resp.user_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "unauthorized"})),
            )
                .into_response();
        }
    };

    let tenant_id = match Uuid::parse_str(&resp.tenant_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "unauthorized"})),
            )
                .into_response();
        }
    };

    // Rate limit check
    let mut redis = ctx.redis.clone();
    match crate::rate_limit::check_rate_limit(
        &mut redis,
        tenant_id,
        ctx.config.rate_limit_per_minute,
    )
    .await
    {
        Ok(true) => {}
        Ok(false) => {
            return (
                StatusCode::TOO_MANY_REQUESTS,
                Json(json!({"error": "rate limit exceeded"})),
            )
                .into_response();
        }
        Err(e) => {
            // Fail open: if Redis is down, allow the request but log
            tracing::error!(error = %e, "rate limit check failed, allowing request");
        }
    }

    let claims = AuthClaims {
        user_id,
        tenant_id,
        email: resp.email,
    };

    req.extensions_mut().insert(claims);
    next.run(req).await
}
