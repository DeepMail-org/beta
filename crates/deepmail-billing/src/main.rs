mod config;
mod consumer;
mod db;
mod error;
mod invoice;
mod meter;
mod razorpay;
mod service;
mod webhook;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::routing::{get, post};
use axum::{Extension, Router};
use sqlx::postgres::PgPoolOptions;
use tonic::transport::Server;

use deepmail_common::proto::billing::billing_service_server::BillingServiceServer;

use crate::config::BillingConfig;
use crate::invoice::BillingCtx;
use crate::razorpay::RazorpayClient;
use crate::service::BillingGrpcService;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("deepmail-billing starting");

    let cfg = BillingConfig::load().unwrap_or_else(|e| {
        tracing::warn!("config load failed ({}), using env vars", e);
        BillingConfig {
            database_url: env_or("DATABASE_URL", "postgres://deepmail:deepmailpw@localhost:5432/deepmail_billing"),
            auth_database_url: env_or("AUTH_DATABASE_URL", "postgres://deepmail:deepmailpw@localhost:5432/deepmail_auth"),
            tenant_database_url: env_or("TENANT_DATABASE_URL", "postgres://deepmail:deepmailpw@localhost:5432/deepmail_tenant"),
            nats_url: env_or("NATS_URL", "nats://localhost:4222"),
            grpc_port: env_parse("GRPC_PORT", 50067),
            http_port: env_parse("HTTP_PORT", 8082),
            razorpay_key_id: env_or("RAZORPAY_KEY_ID", ""),
            razorpay_key_secret: env_or("RAZORPAY_KEY_SECRET", ""),
            razorpay_webhook_secret: env_or("RAZORPAY_WEBHOOK_SECRET", ""),
        }
    });

    let own_pool = Arc::new(
        PgPoolOptions::new()
            .max_connections(15)
            .connect(&cfg.database_url)
            .await?,
    );
    tracing::info!("connected to deepmail_billing DB");

    sqlx::migrate!("../../migrations/deepmail-billing")
        .run(own_pool.as_ref())
        .await?;
    tracing::info!("migrations applied");

    let auth_pool = Arc::new(PgPoolOptions::new().max_connections(5).connect_lazy(&cfg.auth_database_url)?);
    let tenant_pool = Arc::new(PgPoolOptions::new().max_connections(5).connect_lazy(&cfg.tenant_database_url)?);
    tracing::info!("cross-service pools configured (lazy)");

    let razorpay = Arc::new(RazorpayClient::new(
        cfg.razorpay_key_id.clone(),
        cfg.razorpay_key_secret.clone(),
    ));

    if razorpay.is_configured() {
        tracing::info!("Razorpay client configured");
    } else {
        tracing::warn!("Razorpay keys empty, invoice creation disabled (metering still active)");
    }

    let billing_ctx = Arc::new(BillingCtx {
        pool: Arc::clone(&own_pool),
        auth_pool,
        tenant_pool,
        razorpay,
        config: cfg.clone(),
    });

    let nats = async_nats::connect(&cfg.nats_url).await?;
    tracing::info!("connected to NATS");

    let consumer_ctx = Arc::clone(&billing_ctx);
    let consumer_nats = nats.clone();
    tokio::spawn(async move {
        consumer::run_consumer(consumer_nats, consumer_ctx).await;
    });

    let grpc_addr: SocketAddr = format!("0.0.0.0:{}", cfg.grpc_port).parse()?;
    tracing::info!(%grpc_addr, "starting gRPC server");

    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<BillingServiceServer<BillingGrpcService>>()
        .await;

    let grpc_svc = BillingGrpcService::new(Arc::clone(&billing_ctx));

    let grpc_handle = tokio::spawn(async move {
        Server::builder()
            .add_service(health_service)
            .add_service(BillingServiceServer::new(grpc_svc))
            .serve_with_shutdown(grpc_addr, async {
                tokio::signal::ctrl_c().await.ok();
            })
            .await
    });

    let http_addr: SocketAddr = format!("0.0.0.0:{}", cfg.http_port).parse()?;
    tracing::info!(%http_addr, "starting HTTP server");

    let router = Router::new()
        .route("/webhook/razorpay", post(webhook::razorpay_webhook))
        .route("/health", get(health))
        .layer(Extension(billing_ctx));

    let listener = tokio::net::TcpListener::bind(http_addr).await?;

    let http_handle = tokio::spawn(async move {
        axum::serve(listener, router.into_make_service())
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
