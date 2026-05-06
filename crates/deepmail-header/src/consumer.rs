//! NATS JetStream consumer for header analysis jobs.
//!
//! Listens on the `deepmail.jobs.analysis` subject and processes
//! messages concurrently up to the configured concurrency limit.
//! Each message is deserialized as a NatsEnvelope, triggering the
//! header analysis pipeline.

use std::sync::Arc;

use sqlx::PgPool;
use uuid::Uuid;

use deepmail_common::nats::NatsEnvelope;

use crate::dns::Resolver;
use crate::error::HeaderError;
use crate::pipeline;

/// Start the NATS consumer loop.
/// Runs until the provided cancellation token signals shutdown.
pub async fn run(
    nats_client: async_nats::Client,
    header_pool: PgPool,
    ingest_pool: PgPool,
    parser_pool: PgPool,
    resolver: Arc<Resolver>,
    concurrency: usize,
) -> Result<(), HeaderError> {
    use async_nats::jetstream;
    use futures::StreamExt;

    let js = jetstream::new(nats_client);

    // Ensure the stream exists (idempotent).
    let _stream = js
        .get_or_create_stream(jetstream::stream::Config {
            name: "DEEPMAIL_ANALYSIS".to_string(),
            subjects: vec!["deepmail.jobs.analysis".to_string()],
            retention: jetstream::stream::RetentionPolicy::WorkQueue,
            ..Default::default()
        })
        .await
        .map_err(|e| HeaderError::Nats(format!("stream setup: {e}")))?;

    // Create or bind a durable consumer.
    let consumer = js
        .create_consumer_on_stream(
            jetstream::consumer::pull::Config {
                durable_name: Some("header-analyzer".to_string()),
                filter_subject: "deepmail.jobs.analysis".to_string(),
                ack_policy: jetstream::consumer::AckPolicy::Explicit,
                max_deliver: 3,
                ..Default::default()
            },
            "DEEPMAIL_ANALYSIS",
        )
        .await
        .map_err(|e| HeaderError::Nats(format!("consumer setup: {e}")))?;

    tracing::info!(
        concurrency = concurrency,
        "header consumer started, pulling from deepmail.jobs.analysis"
    );

    // Pull messages with the configured concurrency.
    let mut messages = consumer
        .messages()
        .await
        .map_err(|e| HeaderError::Nats(format!("message stream: {e}")))?;

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

        let header_pool = header_pool.clone();
        let ingest_pool = ingest_pool.clone();
        let parser_pool = parser_pool.clone();
        let resolver = resolver.clone();

        tokio::spawn(async move {
            let _permit = permit;
            if let Err(e) = process_message(
                msg,
                &header_pool,
                &ingest_pool,
                &parser_pool,
                &resolver,
            )
            .await
            {
                tracing::error!(error = %e, "failed to process header analysis message");
            }
        });
    }

    Ok(())
}

/// Process a single NATS message.
async fn process_message(
    msg: async_nats::jetstream::Message,
    header_pool: &PgPool,
    ingest_pool: &PgPool,
    parser_pool: &PgPool,
    resolver: &Resolver,
) -> Result<(), HeaderError> {
    let envelope: NatsEnvelope = serde_json::from_slice(&msg.payload)
        .map_err(|e| HeaderError::MalformedEnvelope(e.to_string()))?;

    let email_id = envelope.email_id;
    let tenant_id = envelope.tenant_id;

    tracing::info!(
        email_id = %email_id,
        tenant_id = %tenant_id,
        trace_id = %envelope.trace_id,
        "processing header analysis"
    );

    // Mark job_progress as running
    mark_running(ingest_pool, email_id).await?;

    let state = pipeline::PipelineState {
        header_pool: header_pool.clone(),
        ingest_pool: ingest_pool.clone(),
        parser_pool: parser_pool.clone(),
        resolver: Arc::new(crate::dns::Resolver {
            inner: resolver.inner.clone(),
        }),
    };

    match pipeline::analyze(&state, email_id, tenant_id).await {
        Ok(output) => {
            tracing::info!(
                email_id = %email_id,
                risk_score = output.risk_score,
                spf = %output.spf_result,
                dkim = %output.dkim_result,
                dmarc = %output.dmarc_result,
                "header analysis completed"
            );
            msg.ack()
                .await
                .map_err(|e| HeaderError::Nats(format!("ack: {e}")))?;
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
                .map_err(|e| HeaderError::Nats(format!("nak: {e}")))?;
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
                .map_err(|e| HeaderError::Nats(format!("term: {e}")))?;
        }
    }

    Ok(())
}

/// Mark job_progress as running. Cross-service — runtime SQL.
async fn mark_running(pool: &PgPool, email_id: Uuid) -> Result<(), HeaderError> {
    sqlx::query(
        r#"UPDATE job_progress
           SET status = 'running', started_at = now()
           WHERE email_id = $1 AND stage = 'header'"#,
    )
    .bind(email_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Mark job_progress as failed. Cross-service — runtime SQL.
async fn mark_failed(
    pool: &PgPool,
    email_id: Uuid,
    error_message: &str,
) -> Result<(), HeaderError> {
    sqlx::query(
        r#"UPDATE job_progress
           SET status = 'failed',
               completed_at = now(),
               error_message = $1,
               retry_count = retry_count + 1
           WHERE email_id = $2 AND stage = 'header'"#,
    )
    .bind(error_message)
    .bind(email_id)
    .execute(pool)
    .await?;
    Ok(())
}
