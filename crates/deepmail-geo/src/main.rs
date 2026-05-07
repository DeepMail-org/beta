mod blocklist;
mod classifier;
mod config;
mod consumer;
mod db;
mod enrichment;
mod error;
mod geoip;
mod pipeline;
mod risk;
mod service;
mod tor;
mod vpn;

use std::sync::Arc;

use deepmail_common::db::create_pool;
use deepmail_common::proto::geo::geo_intel_server::GeoIntelServer;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,deepmail_geo=debug".parse().unwrap()),
        )
        .json()
        .init();

    tracing::info!("deepmail-geo starting");

    let cfg = config::GeoConfig::load()?;

    let geo_pool = create_pool(&cfg.database_url).await?;
    let ingest_pool = create_pool(&cfg.ingest_database_url).await?;
    let parser_pool = create_pool(&cfg.parser_database_url).await?;

    sqlx::migrate!("../../migrations/deepmail-geo")
        .run(&geo_pool)
        .await?;

    tracing::info!("database pools ready, migrations applied");

    let redis_client = redis::Client::open(cfg.redis_url.as_str())?;
    let redis_conn = redis::aio::ConnectionManager::new(redis_client).await?;

    tracing::info!(url = %cfg.redis_url, "Redis connected");

    let geoip_reader = geoip::GeoIpReader::open(
        &cfg.geoip_city_db_path,
        &cfg.geoip_asn_db_path,
    );
    let geoip_reader = Arc::new(geoip_reader);

    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(cfg.http_timeout_secs))
        .pool_max_idle_per_host(10)
        .build()?;

    let enrichment_config = Arc::new(enrichment::EnrichmentConfig {
        ipinfo_token: cfg.ipinfo_token.clone(),
        abuseipdb_api_key: cfg.abuseipdb_api_key.clone(),
        greynoise_api_key: cfg.greynoise_api_key.clone(),
        cache_ttl: cfg.enrichment_cache_ttl,
    });

    let tor_handle = tokio::spawn(tor::refresh_loop(
        http_client.clone(),
        redis_conn.clone(),
    ));

    let vpn_handle = tokio::spawn(vpn::refresh_loop(
        redis_conn.clone(),
        cfg.whois_timeout_secs,
    ));

    let blocklist_handle = tokio::spawn(blocklist::refresh_loop(
        geo_pool.clone(),
        http_client.clone(),
    ));

    tracing::info!("background refresh tasks started (TOR/VPN/blocklist)");

    let nats_client = async_nats::connect(&cfg.nats_url).await?;
    tracing::info!(url = %cfg.nats_url, "NATS connected");

    let grpc_addr = format!("0.0.0.0:{}", cfg.grpc_port).parse()?;
    let geo_service = service::GeoService {
        geo_pool: geo_pool.clone(),
        ingest_pool: ingest_pool.clone(),
        parser_pool: parser_pool.clone(),
        redis: redis_conn.clone(),
        geoip: geoip_reader.clone(),
        http_client: http_client.clone(),
        enrichment_config: enrichment_config.clone(),
    };

    let grpc_server = Server::builder()
        .add_service(GeoIntelServer::new(geo_service))
        .serve(grpc_addr);

    let consumer_handle = consumer::run(
        nats_client,
        geo_pool.clone(),
        ingest_pool.clone(),
        parser_pool.clone(),
        redis_conn.clone(),
        geoip_reader.clone(),
        http_client.clone(),
        enrichment_config.clone(),
        cfg.concurrency,
    );

    tracing::info!(grpc_addr = %grpc_addr, "deepmail-geo ready");

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
        _ = tor_handle => {
            tracing::warn!("TOR refresh task exited");
        }
        _ = vpn_handle => {
            tracing::warn!("VPN refresh task exited");
        }
        _ = blocklist_handle => {
            tracing::warn!("blocklist refresh task exited");
        }
        _ = async {
            if let Err(e) = tokio::signal::ctrl_c().await {
                tracing::error!(error = %e, "failed to listen for ctrl+c");
            }
        } => {
            tracing::info!("shutdown signal received");
        }
    }

    tracing::info!("deepmail-geo stopped");
    Ok(())
}
