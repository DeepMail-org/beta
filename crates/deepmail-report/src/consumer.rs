use crate::pipeline::PipelineCtx;
use async_nats::jetstream;
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::Semaphore;
use uuid::Uuid;

#[derive(serde::Deserialize)]
struct EventPayload {
    email_id: Uuid,
    tenant_id: Uuid,
}

pub async fn run_consumer(nats: async_nats::Client, ctx: Arc<PipelineCtx>, concurrency: usize) {
    let js = jetstream::new(nats);

    let stream = match js
        .get_or_create_stream(jetstream::stream::Config {
            name: "REPORT_SCORING_EVENTS".to_string(),
            subjects: vec!["deepmail.events.scoring.completed".to_string()],
            retention: jetstream::stream::RetentionPolicy::Interest,
            ..Default::default()
        })
        .await
    {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("failed to get/create REPORT_SCORING_EVENTS stream: {}", e);
            return;
        }
    };

    let consumer = match stream
        .get_or_create_consumer(
            "report-scoring-events",
            jetstream::consumer::pull::Config {
                durable_name: Some("report-scoring-events".to_string()),
                filter_subject: "deepmail.events.scoring.completed".to_string(),
                ack_policy: jetstream::consumer::AckPolicy::Explicit,
                ..Default::default()
            },
        )
        .await
    {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("failed to create report-scoring-events consumer: {}", e);
            return;
        }
    };

    let mut messages = match consumer.messages().await {
        Ok(m) => m,
        Err(e) => {
            tracing::error!("failed to subscribe report-scoring-events: {}", e);
            return;
        }
    };

    tracing::info!("report-scoring-events consumer started");

    let sem = Arc::new(Semaphore::new(concurrency));

    while let Some(msg_result) = messages.next().await {
        let msg = match msg_result {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!("report consumer message error: {}", e);
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
            handle_message(&ctx, &msg).await;
        });
    }
}

async fn handle_message(ctx: &PipelineCtx, msg: &jetstream::Message) {
    let payload: EventPayload = match serde_json::from_slice(&msg.payload) {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!(error = %e, "bad report event payload, acking");
            let _ = msg.ack().await;
            return;
        }
    };

    match crate::pipeline::generate_report(ctx, payload.email_id, payload.tenant_id, false).await {
        Ok(result) => {
            tracing::info!(
                email_id = %payload.email_id,
                verdict = result.final_verdict.as_str(),
                json_key = result.json_s3_key.as_str(),
                "report generated"
            );
            let _ = msg.ack().await;
        }
        Err(e) if e.is_transient() => {
            tracing::warn!(
                email_id = %payload.email_id,
                error = %e,
                "transient error generating report, naking for retry"
            );
            let _ = msg.ack_with(jetstream::AckKind::Nak(None)).await;
        }
        Err(e) => {
            tracing::error!(
                email_id = %payload.email_id,
                error = %e,
                "permanent error generating report, acking"
            );
            let _ = msg.ack().await;
        }
    }
}
