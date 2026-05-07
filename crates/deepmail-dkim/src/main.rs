mod canon;
mod config;
mod consumer;
mod db;
mod dns;
mod error;
mod parse;
mod pipeline;
mod service;
mod signals;
mod verify;

use std::sync::Arc;

use deepmail_common::db::create_pool;
use deepmail_common::proto::dkim::dkim_analyzer_server::DkimAnalyzerServer;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,deepmail_dkim=debug".parse().unwrap()),
        )
        .json()
        .init();

    tracing::info!("deepmail-dkim starting");

    let cfg = config::DkimConfig::load()?;

    let dkim_pool = create_pool(&cfg.database_url).await?;
    let ingest_pool = create_pool(&cfg.ingest_database_url).await?;

    sqlx::migrate!("../../migrations/deepmail-dkim")
        .run(&dkim_pool)
        .await?;

    tracing::info!("database pools ready, migrations applied");

    let resolver = if cfg.dns_resolver.is_empty() {
        dns::Resolver::new_system()?
    } else {
        dns::Resolver::new_with_server(&cfg.dns_resolver)?
    };
    let resolver = Arc::new(resolver);

    let nats_client = async_nats::connect(&cfg.nats_url).await?;
    tracing::info!(url = %cfg.nats_url, "NATS connected");

    let grpc_addr = cfg.grpc_addr.parse()?;
    let dkim_service = service::DkimService {
        dkim_pool: dkim_pool.clone(),
        ingest_pool: ingest_pool.clone(),
        resolver: resolver.clone(),
    };

    let grpc_server = Server::builder()
        .add_service(DkimAnalyzerServer::new(dkim_service))
        .serve(grpc_addr);

    let consumer_handle = consumer::run(
        nats_client,
        dkim_pool.clone(),
        ingest_pool.clone(),
        resolver.clone(),
        cfg.concurrency,
    );

    tracing::info!(grpc_addr = %cfg.grpc_addr, "deepmail-dkim ready");

    tokio::select! {
        result = grpc_server => {
            if let Err(e) = result {
                tracing::error!(error = %e, "gRPC server error");
            }
        }
        result = consumer_handle => {
            if let Err(e) = result {
                tracing::error!(error = %e, "NATS consumer error");
            }
        }
        _ = async {
            if let Err(e) = tokio::signal::ctrl_c().await {
                tracing::error!(error = %e, "failed to listen for ctrl+c");
            }
        } => {
            tracing::info!("shutdown signal received");
        }
    }

    tracing::info!("deepmail-dkim stopped");
    Ok(())
}
