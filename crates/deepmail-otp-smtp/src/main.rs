use std::sync::Arc;

use deepmail_common::proto::otp_smtp::otp_smtp_service_server::OtpSmtpServiceServer;
use tonic::transport::Server;
use tracing::info;

mod config;
mod error;
mod service;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "deepmail_otp_smtp=info,deepmail_common=info".into()),
        )
        .with_current_span(true)
        .init();

    let cfg = Arc::new(config::Config::load().map_err(|e| anyhow::anyhow!("config error: {e}"))?);
    let addr = cfg
        .grpc_addr
        .parse()
        .map_err(|e| anyhow::anyhow!("invalid grpc addr: {e}"))?;

    let svc = service::OtpSmtpServiceImpl::new(cfg.clone())
        .map_err(|e| anyhow::anyhow!("smtp service init error: {e}"))?;

    info!(addr = %addr, "deepmail-otp-smtp listening");

    Server::builder()
        .add_service(OtpSmtpServiceServer::new(svc))
        .serve_with_shutdown(addr, async {
            if let Err(e) = tokio::signal::ctrl_c().await {
                tracing::error!(error = %e, "failed to install ctrl-c handler");
                std::future::pending::<()>().await;
            }
            info!("shutdown signal received");
        })
        .await
        .map_err(|e| anyhow::anyhow!("server error: {e}"))?;

    Ok(())
}
