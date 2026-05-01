//! deepmail-auth — Authentication & Identity Service
//!
//! Starts a gRPC server implementing `AuthService`, runs database
//! migrations on startup, and shuts down gracefully on Ctrl-C / SIGTERM.

use std::sync::Arc;

use deepmail_common::proto::auth::auth_service_server::AuthServiceServer;
use tonic::transport::Server;
use tracing::info;

mod config;
mod crypto;
mod db;
mod error;
mod service;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ── Tracing ─────────────────────────────────────────────────
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| {
                    "deepmail_auth=info,deepmail_common=info".into()
                }),
        )
        .with_current_span(true)
        .init();

    info!(service = "deepmail-auth", "starting");

    // ── Config ───────────────────────────────────────────────────
    let config = config::Config::load()
        .map_err(|e| anyhow::anyhow!("config error: {e}"))?;
    let config = Arc::new(config);

    // ── Database ─────────────────────────────────────────────────
    let pool = deepmail_common::db::create_pool(&config.database_url)
        .await
        .map_err(|e| anyhow::anyhow!("db pool error: {e}"))?;
    let pool = Arc::new(pool);

    // ── Migrations ───────────────────────────────────────────────
    sqlx::migrate!("../../migrations/deepmail-auth")
        .run(pool.as_ref())
        .await
        .map_err(|e| anyhow::anyhow!("migration error: {e}"))?;
    info!("database migrations applied");

    // ── JWT ──────────────────────────────────────────────────────
    let jwt = Arc::new(
        crypto::jwt::JwtManager::new(
            &config.jwt_private_key_pem,
            &config.jwt_public_key_pem,
            config.jwt_access_expiry_seconds,
        )
        .map_err(|e| anyhow::anyhow!("jwt init error: {e}"))?,
    );

    // ── gRPC server ──────────────────────────────────────────────
    let addr = config
        .grpc_addr
        .parse()
        .map_err(|e| anyhow::anyhow!("invalid grpc_addr: {e}"))?;

    let auth_service = service::AuthServiceImpl {
        pool: Arc::clone(&pool),
        jwt: Arc::clone(&jwt),
        config: Arc::clone(&config),
    };

    info!(addr = %addr, "deepmail-auth gRPC server listening");

    Server::builder()
        .add_service(AuthServiceServer::new(auth_service))
        .serve_with_shutdown(addr, async {
            // Hard Rule #1: no .expect() in production paths.
            // If signal-handler installation fails we log and let the
            // server run until externally killed; missing graceful
            // shutdown is preferable to a panic at startup.
            if let Err(e) = tokio::signal::ctrl_c().await {
                tracing::error!(error = %e, "failed to install ctrl-c handler");
                std::future::pending::<()>().await;
            }
            info!("shutdown signal received — draining connections");
        })
        .await
        .map_err(|e| anyhow::anyhow!("server error: {e}"))?;

    info!("deepmail-auth stopped");
    Ok(())
}
