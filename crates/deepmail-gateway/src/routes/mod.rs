use std::sync::Arc;

use axum::{middleware, routing::{get, post, put}, Router};

use crate::GatewayCtx;

pub mod auth;
pub mod emails;
pub mod health;
pub mod notify;
pub mod tenant;

pub fn build_router(ctx: Arc<GatewayCtx>) -> Router {
    // Public routes with auth rate limiting — no auth required but rate limited
    let public_routes = Router::new()
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/auth/verify-otp", post(auth::verify_otp))
        .route("/auth/refresh", post(auth::refresh))
        .route("/health", get(health::health_check))
        // Apply rate limiting to auth endpoints
        .layer(middleware::from_fn_with_state(
            ctx.clone(),
            crate::middleware::rate_limit::auth_rate_limit_middleware,
        ));

    // Protected routes — auth + rate limit middleware applied here
    // Logging is INSIDE the auth layer so it can read claims from extensions
    let protected_routes = Router::new()
        .route("/api/v1/emails/upload", post(emails::upload))
        .route("/api/v1/emails/:id/status", get(emails::status))
        .route("/api/v1/emails/:id/score", get(emails::score))
        .route("/api/v1/emails/:id/report", get(emails::report))
        .route("/api/v1/emails/:id/iocs", get(emails::iocs))
        .route("/api/v1/emails/:id/graph", get(emails::graph))
        .route("/api/v1/tenant", get(tenant::get_tenant))
        .route("/api/v1/tenant/invite", post(tenant::invite_member))
        .route("/api/v1/tenant/usage", get(tenant::usage))
        .route("/api/v1/notify/config", get(notify::get_config))
        .route("/api/v1/notify/config", put(notify::upsert_config))
        .merge(crate::graphql::graphql_routes(ctx.clone()))
        // Logging runs innermost — sees claims already inserted
        .layer(middleware::from_fn_with_state(
            ctx.clone(),
            crate::middleware::logging_middleware,
        ))
        // Auth runs outermost on protected group — inserts claims
        .layer(middleware::from_fn_with_state(
            ctx.clone(),
            crate::auth_middleware::auth_middleware,
        ));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(ctx)
}