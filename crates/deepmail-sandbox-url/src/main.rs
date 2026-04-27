use std::sync::Arc;

use deepmail_common::config::{init_tracing, ServiceConfig};
use deepmail_common::db::create_pg_pool;
use deepmail_common::nats::create_jetstream_context;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ServiceConfig::from_env("deepmail-sandbox-url")?;
    init_tracing(&config.service_name, &config.log_level);

    tracing::info!(
        service = "deepmail-sandbox-url",
        grpc_addr = %config.grpc_addr,
        "starting service"
    );

    let pool = Arc::new(create_pg_pool(&config.database_url).await?);
    let _js = Arc::new(create_jetstream_context(&config.nats_url).await?);

    let addr: std::net::SocketAddr = config.grpc_addr.parse()?;

    tracing::info!(%addr, "gRPC server listening");

    // gRPC server will be wired here in the service's implementation phase.
    // For now, we set up graceful shutdown to validate the scaffold compiles.
    tokio::signal::ctrl_c().await?;
    tracing::info!("received shutdown signal, draining connections");

    pool.close().await;
    tracing::info!("deepmail-sandbox-url stopped gracefully");
    Ok(())
}
