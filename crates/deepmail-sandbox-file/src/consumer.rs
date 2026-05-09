/// NATS JetStream consumer for file sandbox jobs.

use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;
use tokio::sync::Semaphore;
use uuid::Uuid;

use crate::pipeline::{IncomingJob, JobCtx};

/// Start the NATS JetStream consumer for "deepmail.sandbox.file.>" subject.
pub async fn start_consumer(ctx: Arc<JobCtx>) -> anyhow::Result<()> {
    let jetstream = async_nats::jetstream::new(ctx.nats.clone());

    // Get or create stream
    let stream = jetstream
        .get_or_create_stream(async_nats::jetstream::stream::Config {
            name: "SANDBOX_FILE".to_string(),
            subjects: vec!["deepmail.sandbox.file.>".to_string()],
            retention: async_nats::jetstream::stream::RetentionPolicy::WorkQueue,
            max_age: Duration::from_secs(86400), // 24h
            ..Default::default()
        })
        .await?;

    // Create durable consumer
    let consumer = stream
        .get_or_create_consumer(
            "file-sandbox",
            async_nats::jetstream::consumer::pull::Config {
                durable_name: Some("file-sandbox".to_string()),
                ack_wait: Duration::from_secs(300), // 5 min
                ..Default::default()
            },
        )
        .await?;

    tracing::info!("NATS consumer 'file-sandbox' ready");

    // Concurrency limiter — max 4 concurrent file jobs
    let semaphore = Arc::new(Semaphore::new(4));

    loop {
        let mut messages = consumer.messages().await?;

        while let Some(Ok(msg)) = messages.next().await {
            let permit = Arc::clone(&semaphore);
            let ctx = Arc::clone(&ctx);

            tokio::spawn(async move {
                let _permit = match permit.acquire().await {
                    Ok(p) => p,
                    Err(_) => return,
                };

                match parse_job_message(&msg) {
                    Ok(job) => {
                        match crate::pipeline::run_file_job(ctx, job).await {
                            Ok(report) => {
                                tracing::info!(report_id = %report.id, "file job completed");
                                let _ = msg.ack().await;
                            }
                            Err(e) => {
                                tracing::error!("file job failed: {}", e);
                                // Negative ack for retry
                                let _ = msg.ack().await; // ack anyway to avoid infinite loop
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("bad job message: {}", e);
                        let _ = msg.ack().await;
                    }
                }
            });
        }

        // Reconnect on stream disconnect
        tracing::warn!("NATS message stream ended, reconnecting in 3s...");
        tokio::time::sleep(Duration::from_secs(3)).await;
    }
}

/// Parse a NATS message payload into IncomingJob.
fn parse_job_message(
    msg: &async_nats::jetstream::message::Message,
) -> Result<IncomingJob, String> {
    let payload: serde_json::Value = serde_json::from_slice(&msg.payload)
        .map_err(|e| format!("JSON parse: {}", e))?;

    let attachment_id = payload
        .get("attachment_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or("missing attachment_id")?;

    let email_id = payload
        .get("email_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or("missing email_id")?;

    let tenant_id = payload
        .get("tenant_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or("missing tenant_id")?;

    let s3_key = payload
        .get("s3_key")
        .and_then(|v| v.as_str())
        .ok_or("missing s3_key")?
        .to_string();

    let filename = payload
        .get("filename")
        .and_then(|v| v.as_str())
        .ok_or("missing filename")?
        .to_string();

    Ok(IncomingJob {
        attachment_id,
        email_id,
        tenant_id,
        s3_key,
        filename,
    })
}
