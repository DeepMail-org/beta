use std::sync::Arc;
use std::time::Instant;

use axum::{
    body::Body,
    extract::State,
    http::Request,
    middleware::Next,
    response::Response,
};

use crate::auth_middleware::AuthClaims;
use crate::GatewayCtx;

// Re-export rate limit middleware
pub mod rate_limit;

pub async fn logging_middleware(
    State(ctx): State<Arc<GatewayCtx>>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let start = Instant::now();
    let method = req.method().to_string();
    let path = req.uri().path().to_string();
    let ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s).trim().to_string());

    let claims = req.extensions().get::<AuthClaims>().cloned();

    let response = next.run(req).await;

    let latency_ms = start.elapsed().as_millis() as i32;
    let status_code = response.status().as_u16() as i32;

    let pool = ctx.pool.clone();
    let tenant_id = claims.as_ref().map(|c| c.tenant_id);
    let user_id = claims.as_ref().map(|c| c.user_id);

    tokio::spawn(async move {
        if let Err(e) = crate::db::request_log::insert_request_log(
            &pool,
            tenant_id,
            user_id,
            &method,
            &path,
            status_code,
            latency_ms,
            ip.as_deref(),
        )
        .await
        {
            tracing::warn!(error = %e, "failed to log request");
        }
    });

    response
}
