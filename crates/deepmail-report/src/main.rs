mod assembler;
mod config;
mod consumer;
mod db;
mod error;
mod http;
mod pipeline;
mod renderer;
mod s3;
mod service;

use std::net::SocketAddr;
use std::sync::Arc;

use sqlx::postgres::PgPoolOptions;
use tonic::transport::Server;

use deepmail_common::proto::report::report_service_server::ReportServiceServer;

use crate::assembler::ServicePools;
use crate::config::ReportConfig;
use crate::http::HttpState;
use crate::pipeline::PipelineCtx;
use crate::service::ReportGrpcService;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("deepmail-report starting");

    let cfg = ReportConfig::load().unwrap_or_else(|e| {
        tracing::warn!("config load failed ({}), using env vars", e);
        ReportConfig {
            database_url: env_or("DATABASE_URL", "postgres://deepmail:deepmailpw@localhost:5432/deepmail_report"),
            parser_database_url: env_or("PARSER_DATABASE_URL", "postgres://deepmail:deepmailpw@localhost:5432/deepmail_parser"),
            header_database_url: env_or("HEADER_DATABASE_URL", "postgres://deepmail:deepmailpw@localhost:5432/deepmail_header"),
            dkim_database_url: env_or("DKIM_DATABASE_URL", "postgres://deepmail:deepmailpw@localhost:5432/deepmail_dkim"),
            geo_database_url: env_or("GEO_DATABASE_URL", "postgres://deepmail:deepmailpw@localhost:5432/deepmail_geo"),
            ip_database_url: env_or("IP_DATABASE_URL", "postgres://deepmail:deepmailpw@localhost:5432/deepmail_ip"),
            intel_database_url: env_or("INTEL_DATABASE_URL", "postgres://deepmail:deepmailpw@localhost:5432/deepmail_intel"),
            ioc_database_url: env_or("IOC_DATABASE_URL", "postgres://deepmail:deepmailpw@localhost:5432/deepmail_ioc"),
            homograph_database_url: env_or("HOMOGRAPH_DATABASE_URL", "postgres://deepmail:deepmailpw@localhost:5432/deepmail_homograph"),
            body_database_url: env_or("BODY_DATABASE_URL", "postgres://deepmail:deepmailpw@localhost:5432/deepmail_body"),
            sandbox_url_database_url: env_or("SANDBOX_URL_DATABASE_URL", "postgres://deepmail:deepmailpw@localhost:5432/deepmail_sandbox_url"),
            sandbox_file_database_url: env_or("SANDBOX_FILE_DATABASE_URL", "postgres://deepmail:deepmailpw@localhost:5432/deepmail_sandbox_file"),
            sandbox_dynamic_database_url: env_or("SANDBOX_DYNAMIC_DATABASE_URL", "postgres://deepmail:deepmailpw@localhost:5432/deepmail_sandbox_dynamic"),
            scoring_database_url: env_or("SCORING_DATABASE_URL", "postgres://deepmail:deepmailpw@localhost:5432/deepmail_scoring"),
            nats_url: env_or("NATS_URL", "nats://localhost:4222"),
            grpc_port: env_parse("GRPC_PORT", 50065),
            http_port: env_parse("HTTP_PORT", 8080),
            s3_endpoint: env_or("S3_ENDPOINT", "http://localhost:9000"),
            s3_bucket: env_or("S3_BUCKET", "deepmail-reports"),
            s3_region: env_or("S3_REGION", "us-east-1"),
            aws_access_key_id: env_or("AWS_ACCESS_KEY_ID", ""),
            aws_secret_access_key: env_or("AWS_SECRET_ACCESS_KEY", ""),
        }
    });

    let own_pool = Arc::new(
        PgPoolOptions::new()
            .max_connections(15)
            .connect(&cfg.database_url)
            .await?,
    );
    tracing::info!("connected to deepmail_report DB");

    sqlx::migrate!("../../migrations/deepmail-report")
        .run(own_pool.as_ref())
        .await?;
    tracing::info!("migrations applied");

    let pools = Arc::new(ServicePools {
        parser: Arc::new(PgPoolOptions::new().max_connections(5).connect_lazy(&cfg.parser_database_url)?),
        header: Arc::new(PgPoolOptions::new().max_connections(5).connect_lazy(&cfg.header_database_url)?),
        dkim: Arc::new(PgPoolOptions::new().max_connections(5).connect_lazy(&cfg.dkim_database_url)?),
        geo: Arc::new(PgPoolOptions::new().max_connections(5).connect_lazy(&cfg.geo_database_url)?),
        ip: Arc::new(PgPoolOptions::new().max_connections(5).connect_lazy(&cfg.ip_database_url)?),
        intel: Arc::new(PgPoolOptions::new().max_connections(5).connect_lazy(&cfg.intel_database_url)?),
        ioc: Arc::new(PgPoolOptions::new().max_connections(5).connect_lazy(&cfg.ioc_database_url)?),
        homograph: Arc::new(PgPoolOptions::new().max_connections(5).connect_lazy(&cfg.homograph_database_url)?),
        body: Arc::new(PgPoolOptions::new().max_connections(5).connect_lazy(&cfg.body_database_url)?),
        sandbox_url: Arc::new(PgPoolOptions::new().max_connections(5).connect_lazy(&cfg.sandbox_url_database_url)?),
        sandbox_file: Arc::new(PgPoolOptions::new().max_connections(5).connect_lazy(&cfg.sandbox_file_database_url)?),
        sandbox_dynamic: Arc::new(PgPoolOptions::new().max_connections(5).connect_lazy(&cfg.sandbox_dynamic_database_url)?),
        scoring: Arc::new(PgPoolOptions::new().max_connections(5).connect_lazy(&cfg.scoring_database_url)?),
    });
    tracing::info!("service pools configured (lazy)");

    let s3_client = Arc::new(s3::build_s3_client(
        &cfg.s3_endpoint,
        &cfg.s3_region,
        &cfg.aws_access_key_id,
        &cfg.aws_secret_access_key,
    ));
    tracing::info!("S3 client configured");

    let pipeline_ctx = Arc::new(PipelineCtx {
        pools,
        s3: Arc::clone(&s3_client),
        own_pool: Arc::clone(&own_pool),
        bucket: cfg.s3_bucket.clone(),
    });

    let nats = async_nats::connect(&cfg.nats_url).await?;
    tracing::info!("connected to NATS");

    let consumer_ctx = Arc::clone(&pipeline_ctx);
    let consumer_nats = nats.clone();
    tokio::spawn(async move {
        consumer::run_consumer(consumer_nats, consumer_ctx, 2).await;
    });

    let grpc_addr: SocketAddr = format!("0.0.0.0:{}", cfg.grpc_port).parse()?;
    tracing::info!(%grpc_addr, "starting gRPC server");

    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<ReportServiceServer<ReportGrpcService>>()
        .await;

    let grpc_svc = ReportGrpcService::new(Arc::clone(&pipeline_ctx));

    let grpc_handle = tokio::spawn(async move {
        Server::builder()
            .add_service(health_service)
            .add_service(ReportServiceServer::new(grpc_svc))
            .serve_with_shutdown(grpc_addr, async {
                tokio::signal::ctrl_c().await.ok();
            })
            .await
    });

    let http_addr: SocketAddr = format!("0.0.0.0:{}", cfg.http_port).parse()?;
    tracing::info!(%http_addr, "starting HTTP server");

    let http_state = HttpState {
        s3: s3_client,
        bucket: cfg.s3_bucket,
        own_pool,
    };
    let router = http::build_router(http_state);
    let listener = tokio::net::TcpListener::bind(http_addr).await?;

    let http_handle = tokio::spawn(async move {
        axum::serve(listener, router)
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

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

fn env_parse<T: std::str::FromStr>(key: &str, default: T) -> T {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}
