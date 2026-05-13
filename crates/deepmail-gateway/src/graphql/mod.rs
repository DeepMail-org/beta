use std::sync::Arc;

use async_graphql::http::playground_source;
use axum::{
    extract::{Extension, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};

use crate::auth_middleware::AuthClaims;
use crate::GatewayCtx;

pub mod schema;
pub use schema::{build_schema, AppSchema};

async fn graphql_handler(
    State(ctx): State<Arc<GatewayCtx>>,
    Extension(claims): Extension<AuthClaims>,
    Json(request): Json<async_graphql::Request>,
) -> impl IntoResponse {
    let request = request.data(claims).data(ctx.clone());
    let response = ctx.graphql_schema.execute(request).await;
    let body = serde_json::to_string(&response).unwrap_or_else(|_| r#"{"errors":[{"message":"serialization error"}]}"#.to_string());
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        body,
    )
}

async fn graphiql() -> impl IntoResponse {
    Html(playground_source(async_graphql::http::GraphQLPlaygroundConfig::new("/graphql")))
}

pub fn graphql_routes(ctx: Arc<GatewayCtx>) -> Router<Arc<GatewayCtx>> {
    let mut router = Router::new().route("/graphql", post(graphql_handler));

    if ctx.config.graphiql_enabled {
        router = router.route("/graphql", get(graphiql));
    }

    router
}
