use std::sync::Arc;

use sqlx::PgPool;
use uuid::Uuid;

use deepmail_common::nats::NatsEnvelope;

use crate::dns::Resolver;
use crate::error::DkimError;
use crate::pipeline;

pub async fn run(
    nats_client: async_nats::Client,
    dkim_pool: PgPool,
    ingest_pool: PgPool,
    resolver: Arc<Resolver>,
    concurrency: usize,
) -> Result<(), DkimError> {
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
        .map_err(|e| DkimError::Nats(format!("stream setup: {e}")))?;

    let consumer = js
        .create_consumer_on_stream(
            jetstream::consumer::pull::Config {
                durable_name: Some("dkim-analyzer".to_string()),
                filter_subject: "deepmail.jobs.analysis".to_string(),
                ack_policy: jetstream::consumer::AckPolicy::Explicit,
                max_deliver: 3,
                ..Default::default()
            },
            "DEEPMAIL_ANALYSIS",
        )
        .await
        .map_err(|e| DkimError::Nats(format!("consumer setup: {e}")))?;

    tracing::info!(
        concurrency = concurrency,
        "dkim consumer started, pulling from deepmail.jobs.analysis"
    );

    let mut messages = consumer
        .messages()
        .await
        .map_err(|e| DkimError::Nats(format!("message stream: {e}")))?;

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

        let dkim_pool = dkim_pool.clone();
        let ingest_pool = ingest_pool.clone();
        let resolver = resolver.clone();

        tokio::spawn(async move {
            let _permit = permit;
            if let Err(e) = process_message(msg, &dkim_pool, &ingest_pool, &resolver).await {
                tracing::error!(error = %e, "failed to process dkim analysis message");
            }
        });
    }

    Ok(())
}

async fn process_message(
    msg: async_nats::jetstream::Message,
    dkim_pool: &PgPool,
    ingest_pool: &PgPool,
    resolver: &Resolver,
) -> Result<(), DkimError> {
    let envelope: NatsEnvelope = serde_json::from_slice(&msg.payload)
        .map_err(|e| DkimError::MalformedEnvelope(e.to_string()))?;

    let email_id = envelope.email_id;
    let tenant_id = envelope.tenant_id;

    tracing::info!(
        email_id = %email_id,
        tenant_id = %tenant_id,
        trace_id = %envelope.trace_id,
        "processing dkim analysis"
    );

    mark_running(ingest_pool, email_id).await?;

    let state = pipeline::PipelineState {
        dkim_pool: dkim_pool.clone(),
        ingest_pool: ingest_pool.clone(),
        resolver: Arc::new(Resolver {
            inner: resolver.inner.clone(),
        }),
    };

    match pipeline::analyze(&state, email_id, tenant_id).await {
        Ok(output) => {
            tracing::info!(
                email_id = %email_id,
                replay_confidence = output.replay_confidence,
                verdict = %output.verdict,
                signatures = output.signature_count,
                duration_ms = output.duration_ms,
                "dkim analysis completed"
            );
            msg.ack()
                .await
                .map_err(|e| DkimError::Nats(format!("ack: {e}")))?;
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
                .map_err(|e| DkimError::Nats(format!("nak: {e}")))?;
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
                .map_err(|e| DkimError::Nats(format!("term: {e}")))?;
        }
    }

    Ok(())
}

async fn mark_running(pool: &PgPool, email_id: Uuid) -> Result<(), DkimError> {
    sqlx::query(
        r#"UPDATE job_progress
           SET status = 'running', started_at = now()
           WHERE email_id = $1 AND stage = 'dkim'"#,
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
) -> Result<(), DkimError> {
    sqlx::query(
        r#"UPDATE job_progress
           SET status = 'failed',
               completed_at = now(),
               error_message = $1,
               retry_count = retry_count + 1
           WHERE email_id = $2 AND stage = 'dkim'"#,
    )
    .bind(error_message)
    .bind(email_id)
    .execute(pool)
    .await?;
    Ok(())
}
