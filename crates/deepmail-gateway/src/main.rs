use std::net::SocketAddr;

use axum::Router;
use deepmail_common::config::{init_tracing, env_or_default, require_env};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let log_level = env_or_default("LOG_LEVEL", "info".to_string());
    init_tracing("deepmail-gateway", &log_level);

    let http_addr: SocketAddr = env_or_default("HTTP_ADDR", "0.0.0.0:8080".to_string()).parse()?;
    let _database_url = require_env("DATABASE_URL")?;
    let _nats_url = require_env("NATS_URL")?;

    tracing::info!(%http_addr, "starting deepmail-gateway");

    let app = Router::new()
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(http_addr).await?;
    tracing::info!(%http_addr, "HTTP server listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("deepmail-gateway stopped gracefully");
    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
    tracing::info!("received shutdown signal");
}
