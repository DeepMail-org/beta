mod bgp;
mod config;
mod consumer;
mod db;
mod error;
mod feeds;
mod pdns;
mod pipeline;
mod scorer;
mod service;
mod shodan;


use deepmail_common::db::create_pool;
use deepmail_common::proto::ip::ip_intelligence_server::IpIntelligenceServer;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,deepmail_ip=debug".parse().unwrap()),
        )
        .json()
        .init();

    tracing::info!("deepmail-ip starting");

    // a. Load config
    let cfg = config::IpConfig::load()?;

    // b. Connect main pool (deepmail_ip)
    let ip_pool = create_pool(&cfg.database_url).await?;

    // c. Connect parser_pool (cross-service read)
    let parser_pool = create_pool(&cfg.parser_database_url).await?;

    // d. Connect ingest_pool (cross-service read/write)
    let ingest_pool = create_pool(&cfg.ingest_database_url).await?;

    // Run migrations
    sqlx::migrate!("../../migrations/deepmail-ip")
        .run(&ip_pool)
        .await?;

    tracing::info!("database pools ready, migrations applied");

    // e. Connect Redis
    let redis_client = redis::Client::open(cfg.redis_url.as_str())?;
    let redis_conn = redis::aio::ConnectionManager::new(redis_client).await?;

    tracing::info!(url = %cfg.redis_url, "Redis connected");

    // f. Build HTTP client
    let http_client = reqwest::Client::builder()
        .pool_idle_timeout(std::time::Duration::from_secs(60))
        .timeout(std::time::Duration::from_secs(cfg.http_timeout_secs))
        .build()?;

    // g. Refresh all blocklist feeds immediately (blocking before accepting requests)
    tracing::info!("refreshing all blocklist feeds on startup...");
    feeds::refresh_all_feeds(&ip_pool, &http_client, &mut redis_conn.clone()).await?;
    tracing::info!("initial feed refresh complete");

    // h. Start background feed refresh task
    let feed_handle = tokio::spawn(feeds::refresh_loop(
        ip_pool.clone(),
        http_client.clone(),
        redis_conn.clone(),
        cfg.feed_refresh_hours,
    ));

    // i. Connect NATS JetStream
    let nats_client = async_nats::connect(&cfg.nats_url).await?;
    tracing::info!(url = %cfg.nats_url, "NATS connected");

    // j. Start NATS consumer task
    let consumer_handle = consumer::run(
        nats_client,
        ip_pool.clone(),
        ingest_pool.clone(),
        parser_pool.clone(),
        redis_conn.clone(),
        http_client.clone(),
        cfg.shodan_api_key.clone(),
        cfg.concurrency,
    );

    // k. Start gRPC server
    let grpc_addr = format!("0.0.0.0:{}", cfg.grpc_port).parse()?;
    let ip_service = service::IpService {
        ip_pool: ip_pool.clone(),
        ingest_pool: ingest_pool.clone(),
        parser_pool: parser_pool.clone(),
        redis: redis_conn.clone(),
        http_client: http_client.clone(),
        shodan_api_key: cfg.shodan_api_key.clone(),
    };

    let grpc_server = Server::builder()
        .add_service(IpIntelligenceServer::new(ip_service))
        .serve(grpc_addr);

    tracing::info!(grpc_addr = %grpc_addr, "deepmail-ip ready");

    // l. tokio::select! on all tasks + ctrl-c
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
        _ = feed_handle => {
            tracing::warn!("feed refresh task exited");
        }
        _ = async {
            if let Err(e) = tokio::signal::ctrl_c().await {
                tracing::error!(error = %e, "failed to listen for ctrl+c");
            }
        } => {
            tracing::info!("shutdown signal received");
        }
    }

    tracing::info!("deepmail-ip stopped");
    Ok(())
}
