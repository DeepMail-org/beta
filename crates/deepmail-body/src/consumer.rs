/// NATS durable consumer: body-analyzer.

use std::sync::Arc;

use async_nats::jetstream;
use futures::StreamExt;
use tokio::sync::Semaphore;
use uuid::Uuid;

use crate::pipeline::PipelineCtx;

/// Envelope for job payloads.
#[derive(serde::Deserialize)]
struct JobPayload {
    email_id: String,
    tenant_id: String,
}

/// Run the NATS consumer loop.
pub async fn run_consumer(ctx: Arc<PipelineCtx>) {
    let js = jetstream::new(ctx.nats.clone());
    let sem = Arc::new(Semaphore::new(6));

    // Get or create stream
    let stream = match js
        .get_or_create_stream(jetstream::stream::Config {
            name: "ANALYSIS".to_string(),
            subjects: vec!["deepmail.jobs.analysis".to_string()],
            retention: jetstream::stream::RetentionPolicy::WorkQueue,
            ..Default::default()
        })
        .await
    {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("failed to get/create ANALYSIS stream: {}", e);
            return;
        }
    };

    // Create durable consumer
    let consumer = match stream
        .get_or_create_consumer(
            "body-analyzer",
            jetstream::consumer::pull::Config {
                durable_name: Some("body-analyzer".to_string()),
                filter_subject: "deepmail.jobs.analysis".to_string(),
                ack_policy: jetstream::consumer::AckPolicy::Explicit,
                ..Default::default()
            },
        )
        .await
    {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("failed to create body-analyzer consumer: {}", e);
            return;
        }
    };

    let mut messages = match consumer.messages().await {
        Ok(m) => m,
        Err(e) => {
            tracing::error!("failed to subscribe to messages: {}", e);
            return;
        }
    };

    tracing::info!("body-analyzer consumer started");

    while let Some(msg_result) = messages.next().await {
        let msg = match msg_result {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!("consumer message error: {}", e);
                continue;
            }
        };

        let permit = match sem.clone().acquire_owned().await {
            Ok(p) => p,
            Err(_) => break,
        };

        let ctx = Arc::clone(&ctx);

        tokio::spawn(async move {
            let _permit = permit;

            let payload: JobPayload = match serde_json::from_slice(&msg.payload) {
                Ok(p) => p,
                Err(e) => {
                    tracing::error!("invalid job payload: {}", e);
                    let _ = msg.ack_with(jetstream::AckKind::Term).await;
                    return;
                }
            };

            let email_id: Uuid = match payload.email_id.parse() {
                Ok(id) => id,
                Err(_) => {
                    tracing::error!("invalid email_id in payload: {}", payload.email_id);
                    let _ = msg.ack_with(jetstream::AckKind::Term).await;
                    return;
                }
            };

            let tenant_id: Uuid = match payload.tenant_id.parse() {
                Ok(id) => id,
                Err(_) => {
                    tracing::error!("invalid tenant_id in payload: {}", payload.tenant_id);
                    let _ = msg.ack_with(jetstream::AckKind::Term).await;
                    return;
                }
            };

            match crate::pipeline::run_pipeline(ctx, email_id, tenant_id).await {
                Ok(result) => {
                    tracing::info!(
                        %email_id, verdict = %result.verdict,
                        "body analysis job complete"
                    );
                    let _ = msg.ack().await;
                }
                Err(e) if e.is_transient() => {
                    tracing::warn!(%email_id, "transient error, naking: {}", e);
                    let _ = msg.ack_with(jetstream::AckKind::Nak(None)).await;
                }
                Err(e) => {
                    tracing::error!(%email_id, "permanent error, terminating: {}", e);
                    let _ = msg.ack_with(jetstream::AckKind::Term).await;
                }
            }
        });
    }
}
