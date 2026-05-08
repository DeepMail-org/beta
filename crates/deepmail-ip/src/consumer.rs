/// NATS JetStream consumer for IP analysis jobs.

use std::sync::Arc;

use redis::aio::ConnectionManager;
use sqlx::PgPool;
use uuid::Uuid;

use deepmail_common::nats::NatsEnvelope;

use crate::error::IpError;
use crate::pipeline;

pub async fn run(
    nats_client: async_nats::Client,
    ip_pool: PgPool,
    ingest_pool: PgPool,
    parser_pool: PgPool,
    redis: ConnectionManager,
    http_client: reqwest::Client,
    shodan_api_key: String,
    concurrency: usize,
) -> Result<(), IpError> {
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
        .map_err(|e| IpError::Nats(format!("stream setup: {e}")))?;

    let consumer = js
        .create_consumer_on_stream(
            jetstream::consumer::pull::Config {
                durable_name: Some("ip-analyzer".to_string()),
                filter_subject: "deepmail.jobs.analysis".to_string(),
                ack_policy: jetstream::consumer::AckPolicy::Explicit,
                max_deliver: 3,
                ..Default::default()
            },
            "DEEPMAIL_ANALYSIS",
        )
        .await
        .map_err(|e| IpError::Nats(format!("consumer setup: {e}")))?;

    tracing::info!(
        concurrency = concurrency,
        "ip consumer started, pulling from deepmail.jobs.analysis"
    );

    let mut messages = consumer
        .messages()
        .await
        .map_err(|e| IpError::Nats(format!("message stream: {e}")))?;

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

        let ip_pool = ip_pool.clone();
        let ingest_pool = ingest_pool.clone();
        let parser_pool = parser_pool.clone();
        let redis = redis.clone();
        let http_client = http_client.clone();
        let shodan_api_key = shodan_api_key.clone();

        tokio::spawn(async move {
            let _permit = permit;
            if let Err(e) = process_message(
                msg,
                &ip_pool,
                &ingest_pool,
                &parser_pool,
                redis,
                &http_client,
                &shodan_api_key,
            )
            .await
            {
                tracing::error!(error = %e, "failed to process ip analysis message");
            }
        });
    }

    Ok(())
}

async fn process_message(
    msg: async_nats::jetstream::Message,
    ip_pool: &PgPool,
    ingest_pool: &PgPool,
    parser_pool: &PgPool,
    redis: ConnectionManager,
    http_client: &reqwest::Client,
    shodan_api_key: &str,
) -> Result<(), IpError> {
    let envelope: NatsEnvelope = serde_json::from_slice(&msg.payload)
        .map_err(|e| IpError::MalformedEnvelope(e.to_string()))?;

    let email_id = envelope.email_id;
    let tenant_id = envelope.tenant_id;

    tracing::info!(
        email_id = %email_id,
        tenant_id = %tenant_id,
        trace_id = %envelope.trace_id,
        "processing ip analysis"
    );

    mark_running(ingest_pool, email_id).await?;

    let state = pipeline::PipelineState {
        ip_pool: ip_pool.clone(),
        ingest_pool: ingest_pool.clone(),
        parser_pool: parser_pool.clone(),
        redis,
        http_client: http_client.clone(),
        shodan_api_key: shodan_api_key.to_string(),
    };

    match pipeline::analyze(&state, email_id, tenant_id).await {
        Ok(output) => {
            tracing::info!(
                email_id = %email_id,
                ips_analyzed = output.ips_analyzed,
                max_score = output.max_threat_score,
                max_verdict = %output.max_verdict,
                "ip analysis completed"
            );
            msg.ack()
                .await
                .map_err(|e| IpError::Nats(format!("ack: {e}")))?;
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
                .map_err(|e| IpError::Nats(format!("nak: {e}")))?;
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
                .map_err(|e| IpError::Nats(format!("term: {e}")))?;
        }
    }

    Ok(())
}

async fn mark_running(pool: &PgPool, email_id: Uuid) -> Result<(), IpError> {
    sqlx::query(
        r#"UPDATE job_progress
           SET status = 'running', started_at = now()
           WHERE email_id = $1 AND stage = 'ip'"#,
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
) -> Result<(), IpError> {
    sqlx::query(
        r#"UPDATE job_progress
           SET status = 'failed',
               completed_at = now(),
               error_message = $1,
               retry_count = retry_count + 1
           WHERE email_id = $2 AND stage = 'ip'"#,
    )
    .bind(error_message)
    .bind(email_id)
    .execute(pool)
    .await?;
    Ok(())
}
