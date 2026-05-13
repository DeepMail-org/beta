mod auth_client;
mod config;
mod consumer;
mod db;
mod dispatcher;
mod error;
mod pipeline;
mod service;
mod smtp;
mod webhook;
mod ws;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::routing::get;
use axum::Router;
use sqlx::postgres::PgPoolOptions;
use tonic::transport::Server;

use deepmail_common::proto::notify::notify_service_server::NotifyServiceServer;

use crate::config::NotifyConfig;
use crate::dispatcher::NotifyCtx;
use crate::service::NotifyGrpcService;
use crate::ws::{WsHub, WsState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("deepmail-notify starting");

    let cfg = NotifyConfig::load().unwrap_or_else(|e| {
        tracing::warn!("config load failed ({}), using env vars", e);
        NotifyConfig {
            database_url: env_or("DATABASE_URL", "postgres://deepmail:deepmailpw@localhost:5432/deepmail_notify"),
            auth_database_url: env_or("AUTH_DATABASE_URL", "postgres://deepmail:deepmailpw@localhost:5432/deepmail_auth"),
            tenant_database_url: env_or("TENANT_DATABASE_URL", "postgres://deepmail:deepmailpw@localhost:5432/deepmail_tenant"),
            report_database_url: env_or("REPORT_DATABASE_URL", "postgres://deepmail:deepmailpw@localhost:5432/deepmail_report"),
            nats_url: env_or("NATS_URL", "nats://localhost:4222"),
            grpc_port: env_parse("GRPC_PORT", 50066),
            http_port: env_parse("HTTP_PORT", 8081),
            auth_grpc_url: env_or("AUTH_GRPC_URL", "http://localhost:50051"),
            smtp_host: env_or("SMTP_HOST", ""),
            smtp_port: env_parse("SMTP_PORT", 587),
            smtp_user: env_or("SMTP_USER", ""),
            smtp_password: env_or("SMTP_PASSWORD", ""),
            smtp_from: env_or("SMTP_FROM", "alerts@deepmail.io"),
            dashboard_url: env_or("DASHBOARD_URL", "http://localhost:3000"),
            webhook_timeout_secs: env_parse("WEBHOOK_TIMEOUT_SECS", 10),
            min_severity_default: env_or("MIN_SEVERITY_DEFAULT", "PHISHING"),
        }
    });

    let own_pool = Arc::new(
        PgPoolOptions::new()
            .max_connections(15)
            .connect(&cfg.database_url)
            .await?,
    );
    tracing::info!("connected to deepmail_notify DB");

    sqlx::migrate!("../../migrations/deepmail-notify")
        .run(own_pool.as_ref())
        .await?;
    tracing::info!("migrations applied");

    let auth_pool = Arc::new(PgPoolOptions::new().max_connections(5).connect_lazy(&cfg.auth_database_url)?);
    let tenant_pool = Arc::new(PgPoolOptions::new().max_connections(5).connect_lazy(&cfg.tenant_database_url)?);
    let report_pool = Arc::new(PgPoolOptions::new().max_connections(5).connect_lazy(&cfg.report_database_url)?);
    tracing::info!("cross-service pools configured (lazy)");

    let smtp_transport = match smtp::build_smtp_transport(&cfg) {
        Ok(t) => {
            tracing::info!("SMTP transport configured");
            Some(Arc::new(t))
        }
        Err(e) => {
            tracing::warn!("SMTP not available: {}, email dispatch disabled", e);
            None
        }
    };

    let auth_client = match auth_client::AuthClient::connect(&cfg.auth_grpc_url).await {
        Ok(c) => {
            tracing::info!("auth gRPC client connected");
            Some(Arc::new(c))
        }
        Err(e) => {
            tracing::warn!("auth client unavailable: {}, WS auth disabled", e);
            None
        }
    };

    let hub = WsHub::new();
    let http_client = reqwest::Client::new();

    let notify_ctx = Arc::new(NotifyCtx {
        own_pool: Arc::clone(&own_pool),
        auth_pool,
        tenant_pool,
        report_pool,
        hub: hub.clone(),
        smtp_transport,
        http_client,
        config: cfg.clone(),
    });

    let nats = async_nats::connect(&cfg.nats_url).await?;
    tracing::info!("connected to NATS");

    let consumer_ctx = Arc::clone(&notify_ctx);
    let consumer_nats = nats.clone();
    tokio::spawn(async move {
        consumer::run_consumers(consumer_nats, consumer_ctx).await;
    });

    let grpc_addr: SocketAddr = format!("0.0.0.0:{}", cfg.grpc_port).parse()?;
    tracing::info!(%grpc_addr, "starting gRPC server");

    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<NotifyServiceServer<NotifyGrpcService>>()
        .await;

    let grpc_svc = NotifyGrpcService::new(Arc::clone(&notify_ctx));

    let grpc_handle = tokio::spawn(async move {
        Server::builder()
            .add_service(health_service)
            .add_service(NotifyServiceServer::new(grpc_svc))
            .serve_with_shutdown(grpc_addr, async {
                tokio::signal::ctrl_c().await.ok();
            })
            .await
    });

    let http_addr: SocketAddr = format!("0.0.0.0:{}", cfg.http_port).parse()?;
    tracing::info!(%http_addr, "starting HTTP/WS server");

    let ws_state = WsState {
        hub,
        auth_client,
        own_pool,
    };

    let router = Router::new()
        .route("/ws", get(ws::ws_handler))
        .route("/health", get(health))
        .with_state(ws_state);

    let listener = tokio::net::TcpListener::bind(http_addr).await?;

    let http_handle = tokio::spawn(async move {
        axum::serve(listener, router.into_make_service_with_connect_info::<SocketAddr>())
            .with_graceful_shutdown(async {
                tokio::signal::ctrl_c().await.ok();
            })
            .await
    });

    tokio::select! {
        res = grpc_handle => {
            if let Err(e) = res {
                tracing::error!("gRPC server error: {}", e);
            }
        }
        res = http_handle => {
            if let Err(e) = res {
                tracing::error!("HTTP server error: {}", e);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("shutting down");
        }
    }

    Ok(())
}

async fn health() -> impl axum::response::IntoResponse {
    (
        axum::http::StatusCode::OK,
        [("content-type", "application/json")],
        r#"{"status":"ok"}"#,
    )
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

fn env_parse<T: std::str::FromStr>(key: &str, default: T) -> T {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}
