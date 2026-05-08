/// NATS JetStream consumer "ioc-extractor" for the analysis pipeline.

use std::sync::Arc;

use async_nats::jetstream::consumer::PullConsumer;
use futures::StreamExt;

use deepmail_common::nats::NatsEnvelope;

use crate::pipeline::PipelineCtx;

pub async fn run_consumer(
    consumer: PullConsumer,
    ctx: Arc<PipelineCtx>,
) {
    let semaphore = Arc::new(tokio::sync::Semaphore::new(6));

    tracing::info!("ioc-extractor NATS consumer started");

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
        let sem = semaphore.clone();

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
                    tracing::warn!(error = %e, "malformed envelope, acking to skip");
                    let _ = msg.ack().await;
                    return;
                }
            };

            let email_id = envelope.email_id;
            let tenant_id = envelope.tenant_id;

            tracing::info!(%email_id, "processing IOC extraction");

            match crate::pipeline::run_pipeline(&ctx, email_id, tenant_id).await {
                Ok(result) => {
                    tracing::info!(
                        %email_id,
                        ioc_count = result.ioc_count,
                        malicious = result.malicious_count,
                        "IOC extraction complete"
                    );
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

    tracing::warn!("ioc-extractor consumer stream ended");
}
