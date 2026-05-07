use std::sync::Arc;

use redis::aio::ConnectionManager;
use sqlx::PgPool;
use uuid::Uuid;

use deepmail_common::nats::NatsEnvelope;

use crate::enrichment::EnrichmentConfig;
use crate::error::GeoError;
use crate::geoip::GeoIpReader;
use crate::pipeline;

pub async fn run(
    nats_client: async_nats::Client,
    geo_pool: PgPool,
    ingest_pool: PgPool,
    parser_pool: PgPool,
    redis: ConnectionManager,
    geoip: Arc<GeoIpReader>,
    http_client: reqwest::Client,
    enrichment_config: Arc<EnrichmentConfig>,
    concurrency: usize,
) -> Result<(), GeoError> {
    use async_nats::jetstream;
    use futures::StreamExt;

    let js = jetstream::new(nats_client);

    let _stream = js
        .get_or_create_stream(jetstream::stream::Config {
            name: "DEEPMAIL_ANALYSIS".to_string(),
            subjects: vec!["deepmail.jobs.analysis".to_string()],
            retention: jetstream::stream::RetentionPolicy::WorkQueue,
            ..Default::default()
        })
        .await
        .map_err(|e| GeoError::Nats(format!("stream setup: {e}")))?;

    let consumer = js
        .create_consumer_on_stream(
            jetstream::consumer::pull::Config {
                durable_name: Some("geo-analyzer".to_string()),
                filter_subject: "deepmail.jobs.analysis".to_string(),
                ack_policy: jetstream::consumer::AckPolicy::Explicit,
                max_deliver: 3,
                ..Default::default()
            },
            "DEEPMAIL_ANALYSIS",
        )
        .await
        .map_err(|e| GeoError::Nats(format!("consumer setup: {e}")))?;

    tracing::info!(
        concurrency = concurrency,
        "geo consumer started, pulling from deepmail.jobs.analysis"
    );

    let mut messages = consumer
        .messages()
        .await
        .map_err(|e| GeoError::Nats(format!("message stream: {e}")))?;

    let semaphore = Arc::new(tokio::sync::Semaphore::new(concurrency));

    while let Some(msg_result) = messages.next().await {
        let msg = match msg_result {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!(error = %e, "failed to receive NATS message");
                continue;
            }
        };

        let permit = match semaphore.clone().acquire_owned().await {
            Ok(p) => p,
            Err(_) => {
                tracing::error!("semaphore closed, exiting consumer");
                break;
            }
        };

        let geo_pool = geo_pool.clone();
        let ingest_pool = ingest_pool.clone();
        let parser_pool = parser_pool.clone();
        let redis = redis.clone();
        let geoip = geoip.clone();
        let http_client = http_client.clone();
        let enrichment_config = enrichment_config.clone();

        tokio::spawn(async move {
            let _permit = permit;
            if let Err(e) = process_message(
                msg, &geo_pool, &ingest_pool, &parser_pool,
                redis, geoip, &http_client, enrichment_config,
            ).await {
                tracing::error!(error = %e, "failed to process geo analysis message");
            }
        });
    }

    Ok(())
}

async fn process_message(
    msg: async_nats::jetstream::Message,
    geo_pool: &PgPool,
    ingest_pool: &PgPool,
    parser_pool: &PgPool,
    redis: ConnectionManager,
    geoip: Arc<GeoIpReader>,
    http_client: &reqwest::Client,
    enrichment_config: Arc<EnrichmentConfig>,
) -> Result<(), GeoError> {
    let envelope: NatsEnvelope = serde_json::from_slice(&msg.payload)
        .map_err(|e| GeoError::MalformedEnvelope(e.to_string()))?;

    let email_id = envelope.email_id;
    let tenant_id = envelope.tenant_id;

    tracing::info!(
        email_id = %email_id,
        tenant_id = %tenant_id,
        trace_id = %envelope.trace_id,
        "processing geo analysis"
    );

    mark_running(ingest_pool, email_id).await?;

    let state = pipeline::PipelineState {
        geo_pool: geo_pool.clone(),
        ingest_pool: ingest_pool.clone(),
        parser_pool: parser_pool.clone(),
        redis,
        geoip,
        http_client: http_client.clone(),
        enrichment_config,
    };

    match pipeline::analyze(&state, email_id, tenant_id).await {
        Ok(output) => {
            tracing::info!(
                email_id = %email_id,
                risk_score = output.risk_score,
                overall_risk = %output.overall_risk,
                hops = output.hop_count,
                duration_ms = output.duration_ms,
                "geo analysis completed"
            );
            msg.ack()
                .await
                .map_err(|e| GeoError::Nats(format!("ack: {e}")))?;
        }
        Err(e) if e.is_recoverable() => {
            tracing::warn!(
                email_id = %email_id,
                error = %e,
                "recoverable error, NAK for redelivery"
            );
            mark_failed(ingest_pool, email_id, &e.to_string()).await?;
            msg.ack_with(async_nats::jetstream::AckKind::Nak(None))
                .await
                .map_err(|e| GeoError::Nats(format!("nak: {e}")))?;
        }
        Err(e) => {
            tracing::error!(
                email_id = %email_id,
                error = %e,
                "permanent error, terminating message"
            );
            mark_failed(ingest_pool, email_id, &e.to_string()).await?;
            msg.ack_with(async_nats::jetstream::AckKind::Term)
                .await
                .map_err(|e| GeoError::Nats(format!("term: {e}")))?;
        }
    }

    Ok(())
}

async fn mark_running(pool: &PgPool, email_id: Uuid) -> Result<(), GeoError> {
    sqlx::query(
        r#"UPDATE job_progress
           SET status = 'running', started_at = now()
           WHERE email_id = $1 AND stage = 'geo'"#,
    )
    .bind(email_id)
    .execute(pool)
    .await?;
    Ok(())
}

async fn mark_failed(
    pool: &PgPool,
    email_id: Uuid,
    error_message: &str,
) -> Result<(), GeoError> {
    sqlx::query(
        r#"UPDATE job_progress
           SET status = 'failed',
               completed_at = now(),
               error_message = $1,
               retry_count = retry_count + 1
           WHERE email_id = $2 AND stage = 'geo'"#,
    )
    .bind(error_message)
    .bind(email_id)
    .execute(pool)
    .await?;
    Ok(())
}
