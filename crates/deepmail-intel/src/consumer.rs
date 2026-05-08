/// NATS JetStream consumer "intel-enricher" for the analysis pipeline.

use std::sync::Arc;

use async_nats::jetstream::consumer::PullConsumer;
use futures::StreamExt;
use sqlx::PgPool;

use deepmail_common::nats::NatsEnvelope;

use crate::enricher::EnrichCtx;
use crate::pipeline;

pub async fn run_consumer(
    consumer: PullConsumer,
    ctx: Arc<EnrichCtx>,
    parser_pool: PgPool,
    ingest_pool: PgPool,
    js: async_nats::jetstream::Context,
) {
    let semaphore = Arc::new(tokio::sync::Semaphore::new(6));

    tracing::info!("intel-enricher NATS consumer started");

    let mut messages = consumer
        .messages()
        .await
        .expect("failed to get message stream");

    while let Some(msg_result) = messages.next().await {
        let msg = match msg_result {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!(error = %e, "error receiving NATS message");
                continue;
            }
        };

        let ctx = ctx.clone();
        let parser_pool = parser_pool.clone();
        let ingest_pool = ingest_pool.clone();
        let sem = semaphore.clone();
        let js = js.clone();

        tokio::spawn(async move {
            let _permit = match sem.acquire().await {
                Ok(p) => p,
                Err(_) => {
                    tracing::error!("semaphore closed");
                    return;
                }
            };

            // Parse envelope
            let envelope = match NatsEnvelope::from_bytes(&msg.payload) {
                Ok(env) => env,
                Err(e) => {
                    tracing::warn!(error = %e, "malformed envelope, terminating message");
                    let _ = msg.ack().await;
                    return;
                }
            };

            let email_id = envelope.email_id;
            let tenant_id = envelope.tenant_id;

            tracing::info!(%email_id, "processing intel enrichment");

            match pipeline::run_pipeline(email_id, tenant_id, ctx.clone(), &parser_pool, &ingest_pool)
                .await
            {
                Ok(()) => {
                    tracing::info!(%email_id, "intel enrichment complete");
                    // Publish completion event
                    let completion_envelope = NatsEnvelope::new(
                        email_id,
                        tenant_id,
                        envelope.user_id,
                        &envelope.trace_id,
                        serde_json::json!({"service": "intel", "status": "completed"}),
                    );
                    if let Ok(payload) = completion_envelope.to_bytes() {
                        let _ = js
                            .publish("deepmail.events.intel.completed".to_string(), payload)
                            .await;
                    }
                    let _ = msg.ack().await;
                }
                Err(e) => {
                    if e.is_transient() {
                        tracing::warn!(%email_id, error = %e, "transient error, nak-ing");
                        let _ = msg.ack_with(async_nats::jetstream::AckKind::Nak(None)).await;
                    } else {
                        tracing::error!(%email_id, error = %e, "permanent error, acking to skip");
                        let _ = msg.ack().await;
                    }
                }
            }
        });
    }

    tracing::warn!("intel-enricher consumer stream ended");
}
