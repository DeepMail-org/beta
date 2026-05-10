/// NATS JetStream consumer for dynamic analysis jobs.

use std::sync::Arc;

use futures::StreamExt;
use uuid::Uuid;

use crate::error::DynamicError;
use crate::pipeline::{self, IncomingJob, JobCtx};

/// Start the NATS durable consumer.
pub async fn start_consumer(ctx: Arc<JobCtx>) -> Result<(), DynamicError> {
    let jetstream = async_nats::jetstream::new(ctx.nats.clone());

    // Ensure stream exists
    let _ = jetstream
        .get_or_create_stream(async_nats::jetstream::stream::Config {
            name: "SANDBOX_DYNAMIC".into(),
            subjects: vec!["deepmail.jobs.sandbox.dynamic".into()],
            retention: async_nats::jetstream::stream::RetentionPolicy::WorkQueue,
            ..Default::default()
        })
        .await
        .map_err(|e| DynamicError::Nats(format!("stream: {}", e)))?;

    // Create durable consumer
    let consumer = jetstream
        .get_or_create_stream("SANDBOX_DYNAMIC")
        .await
        .map_err(|e| DynamicError::Nats(format!("get stream: {}", e)))?
        .get_or_create_consumer(
            "dynamic-sandbox",
            async_nats::jetstream::consumer::pull::Config {
                durable_name: Some("dynamic-sandbox".into()),
                filter_subject: "deepmail.jobs.sandbox.dynamic".into(),
                ack_policy: async_nats::jetstream::consumer::AckPolicy::Explicit,
                ..Default::default()
            },
        )
        .await
        .map_err(|e| DynamicError::Nats(format!("consumer: {}", e)))?;

    tracing::info!("NATS consumer 'dynamic-sandbox' started");

    // Semaphore: 2 concurrent CAPEv2 analyses
    let semaphore = Arc::new(tokio::sync::Semaphore::new(2));

    let mut messages = consumer
        .messages()
        .await
        .map_err(|e| DynamicError::Nats(format!("messages: {}", e)))?;

    while let Some(Ok(msg)) = messages.next().await {
        let ctx = Arc::clone(&ctx);
        let sem = Arc::clone(&semaphore);

        tokio::spawn(async move {
            let _permit = match sem.acquire().await {
                Ok(p) => p,
                Err(_) => return,
            };

            let payload = String::from_utf8_lossy(&msg.payload);
            match parse_job_payload(&payload) {
                Ok(job) => {
                    if let Err(e) = pipeline::run_dynamic_job(ctx, job).await {
                        tracing::error!(error = %e, "dynamic job failed");
                        if e.is_transient() {
                            if let Err(ne) = msg.ack_with(
                                async_nats::jetstream::AckKind::Nak(None),
                            ).await {
                                tracing::warn!("nak failed: {}", ne);
                            }
                        } else {
                            // Permanent error — drop the message
                            if let Err(ne) = msg.ack().await {
                                tracing::warn!("term/ack failed: {}", ne);
                            }
                        }
                        return;
                    }
                    if let Err(e) = msg.ack().await {
                        tracing::warn!("ack failed: {}", e);
                    }
                }
                Err(e) => {
                    tracing::error!(error = %e, "bad payload, dropping");
                    // Permanent error — ack to remove from queue
                    let _ = msg.ack().await;
                }
            }
        });
    }

    Ok(())
}

/// Parse job payload JSON.
fn parse_job_payload(payload: &str) -> Result<IncomingJob, DynamicError> {
    let v: serde_json::Value = serde_json::from_str(payload)
        .map_err(|e| DynamicError::PayloadParse(format!("json: {}", e)))?;

    let attachment_id = v
        .get("attachment_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| DynamicError::PayloadParse("missing attachment_id".into()))?;

    let email_id = v
        .get("email_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| DynamicError::PayloadParse("missing email_id".into()))?;

    let tenant_id = v
        .get("tenant_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| DynamicError::PayloadParse("missing tenant_id".into()))?;

    let s3_key = v
        .get("s3_key")
        .and_then(|v| v.as_str())
        .ok_or_else(|| DynamicError::PayloadParse("missing s3_key".into()))?
        .to_string();

    let filename = v
        .get("filename")
        .and_then(|v| v.as_str())
        .ok_or_else(|| DynamicError::PayloadParse("missing filename".into()))?
        .to_string();

    let sha256_hash = v
        .get("sha256_hash")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    Ok(IncomingJob {
        attachment_id,
        email_id,
        tenant_id,
        s3_key,
        filename,
        sha256_hash,
    })
}
