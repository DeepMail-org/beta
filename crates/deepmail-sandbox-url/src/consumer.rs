/// NATS durable consumer for URL sandbox jobs.

use std::sync::Arc;

use async_nats::jetstream::{self, consumer::PullConsumer};
use futures::StreamExt;
use serde::Deserialize;
use tokio::sync::Semaphore;
use uuid::Uuid;

use crate::db;
use crate::pipeline::{self, JobCtx, UrlSandboxJob};

/// NATS job payload for URL sandbox.
#[derive(Debug, Deserialize)]
struct JobPayload {
    email_id: String,
    tenant_id: String,
    url: String,
    #[serde(default = "default_url_type")]
    url_type: String,
    #[serde(default)]
    job_id: Option<String>,
}

fn default_url_type() -> String {
    "href".to_string()
}

/// Run the NATS consumer loop. Never returns under normal operation.
pub async fn run_consumer(ctx: Arc<JobCtx>) {
    let concurrency = ctx.config.sandbox_concurrency;
    let semaphore = Arc::new(Semaphore::new(concurrency));

    tracing::info!(concurrency = concurrency, "starting url-sandbox consumer");

    loop {
        if let Err(e) = consumer_loop(Arc::clone(&ctx), Arc::clone(&semaphore)).await {
            tracing::error!("consumer loop error, restarting in 5s: {}", e);
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    }
}

async fn consumer_loop(
    ctx: Arc<JobCtx>,
    semaphore: Arc<Semaphore>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let jetstream = jetstream::new(ctx.nats.clone());

    // Get or create stream
    let stream = jetstream
        .get_or_create_stream(async_nats::jetstream::stream::Config {
            name: "SANDBOX_URL".into(),
            subjects: vec!["deepmail.jobs.sandbox.url".into()],
            retention: async_nats::jetstream::stream::RetentionPolicy::WorkQueue,
            ..Default::default()
        })
        .await?;

    // Get or create durable consumer
    let consumer: PullConsumer = stream
        .get_or_create_consumer(
            "url-sandbox",
            async_nats::jetstream::consumer::pull::Config {
                durable_name: Some("url-sandbox".into()),
                ack_policy: async_nats::jetstream::consumer::AckPolicy::Explicit,
                max_deliver: 3,
                ..Default::default()
            },
        )
        .await?;

    tracing::info!("NATS consumer ready: url-sandbox");

    let mut messages = consumer.messages().await?;

    while let Some(msg_result) = messages.next().await {
        let msg = match msg_result {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!("message receive error: {}", e);
                continue;
            }
        };

        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let ctx = Arc::clone(&ctx);

        tokio::spawn(async move {
            let _permit = permit;

            let payload_bytes = msg.payload.to_vec();
            let payload_str = String::from_utf8_lossy(&payload_bytes);

            // Parse payload
            let payload: JobPayload = match serde_json::from_str(&payload_str) {
                Ok(p) => p,
                Err(e) => {
                    tracing::error!("invalid payload, terminating message: {}", e);
                    if let Err(e) = msg.ack_with(async_nats::jetstream::AckKind::Term).await {
                        tracing::warn!("term ack failed: {}", e);
                    }
                    return;
                }
            };

            // Parse UUIDs
            let email_id: Uuid = match payload.email_id.parse() {
                Ok(id) => id,
                Err(_) => {
                    tracing::error!("invalid email_id in payload: {}", payload.email_id);
                    let _ = msg.ack_with(async_nats::jetstream::AckKind::Term).await;
                    return;
                }
            };
            let tenant_id: Uuid = match payload.tenant_id.parse() {
                Ok(id) => id,
                Err(_) => {
                    tracing::error!("invalid tenant_id in payload: {}", payload.tenant_id);
                    let _ = msg.ack_with(async_nats::jetstream::AckKind::Term).await;
                    return;
                }
            };

            // Idempotency: check if already processed
            match db::jobs::find_existing(&ctx.pool, email_id, &payload.url).await {
                Ok(Some(existing_id)) => {
                    tracing::debug!(
                        existing_id = %existing_id,
                        "URL already processed, skipping"
                    );
                    let _ = msg.ack().await;
                    return;
                }
                Ok(None) => {}
                Err(e) => {
                    tracing::warn!("idempotency check failed: {}", e);
                    // Continue anyway — worst case we reprocess
                }
            }

            // Create or use existing job record
            let job_id = if let Some(ref jid) = payload.job_id {
                match jid.parse::<Uuid>() {
                    Ok(id) => id,
                    Err(_) => {
                        match db::jobs::create_job(
                            &ctx.pool, email_id, tenant_id,
                            &payload.url, &payload.url_type,
                        ).await {
                            Ok(id) => id,
                            Err(e) => {
                                tracing::error!("create_job failed: {}", e);
                                let _ = msg.ack_with(async_nats::jetstream::AckKind::Nak(None)).await;
                                return;
                            }
                        }
                    }
                }
            } else {
                match db::jobs::create_job(
                    &ctx.pool, email_id, tenant_id,
                    &payload.url, &payload.url_type,
                ).await {
                    Ok(id) => id,
                    Err(e) => {
                        tracing::error!("create_job failed: {}", e);
                        let _ = msg.ack_with(async_nats::jetstream::AckKind::Nak(None)).await;
                        return;
                    }
                }
            };

            let job = UrlSandboxJob {
                job_id,
                email_id,
                tenant_id,
                url: payload.url.clone(),
                url_type: payload.url_type.clone(),
            };

            // Run the pipeline
            match pipeline::run_url_job(ctx, job).await {
                Ok(()) => {
                    tracing::info!(job_id = %job_id, "sandbox job completed");
                    let _ = msg.ack().await;
                }
                Err(e) => {
                    tracing::error!(job_id = %job_id, "sandbox job failed: {}", e);
                    let _ = msg.ack_with(async_nats::jetstream::AckKind::Nak(None)).await;
                }
            }
        });
    }

    Ok(())
}
