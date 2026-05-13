use crate::invoice::BillingCtx;
use crate::meter;
use async_nats::jetstream;
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::Semaphore;
use uuid::Uuid;

#[derive(serde::Deserialize)]
struct EventPayload {
    tenant_id: Uuid,
    email_id: Uuid,
}

fn subject_to_event_type(subject: &str) -> Option<&'static str> {
    match subject {
        "deepmail.events.ingest.received" => Some("email_ingested"),
        "deepmail.events.header.completed" => Some("header_analyzed"),
        "deepmail.events.body.completed" => Some("body_analyzed"),
        "deepmail.events.sandbox.url.completed" => Some("url_sandboxed"),
        "deepmail.events.sandbox.file.completed" => Some("file_sandboxed"),
        "deepmail.events.sandbox.dynamic.completed" => Some("dynamic_sandboxed"),
        "deepmail.events.ioc.completed" => Some("ioc_extracted"),
        "deepmail.events.ml.completed" => Some("ml_inference"),
        _ => None,
    }
}

pub async fn run_consumer(nats: async_nats::Client, ctx: Arc<BillingCtx>) {
    let js = jetstream::new(nats);

    let stream = match js
        .get_or_create_stream(jetstream::stream::Config {
            name: "BILLING_EVENTS".to_string(),
            subjects: vec!["deepmail.events.>".to_string()],
            retention: jetstream::stream::RetentionPolicy::Interest,
            ..Default::default()
        })
        .await
    {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(error = %e, "failed to get/create BILLING_EVENTS stream");
            return;
        }
    };

    let consumer = match stream
        .get_or_create_consumer(
            "billing-meter",
            jetstream::consumer::pull::Config {
                durable_name: Some("billing-meter".to_string()),
                filter_subject: "deepmail.events.>".to_string(),
                ack_policy: jetstream::consumer::AckPolicy::Explicit,
                ..Default::default()
            },
        )
        .await
    {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(error = %e, "failed to create billing-meter consumer");
            return;
        }
    };

    let mut messages = match consumer.messages().await {
        Ok(m) => m,
        Err(e) => {
            tracing::error!(error = %e, "failed to subscribe to billing-meter");
            return;
        }
    };

    tracing::info!("billing-meter consumer started on deepmail.events.>");

    let sem = Arc::new(Semaphore::new(20));

    while let Some(msg_result) = messages.next().await {
        let msg = match msg_result {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!(error = %e, "message error");
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

async fn handle_message(ctx: &BillingCtx, msg: &jetstream::Message) {
    let subject = msg.subject.as_str();

    let event_type = match subject_to_event_type(subject) {
        Some(et) => et,
        None => {
            let _ = msg.ack().await;
            return;
        }
    };

    let payload: EventPayload = match serde_json::from_slice(&msg.payload) {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!(subject = subject, error = %e, "bad payload, acking");
            let _ = msg.ack().await;
            return;
        }
    };

    match meter::record_event(&ctx.pool, payload.tenant_id, payload.email_id, event_type).await {
        Ok(is_new) => {
            if is_new {
                tracing::debug!(
                    tenant_id = %payload.tenant_id,
                    email_id = %payload.email_id,
                    event_type = event_type,
                    "metered new event"
                );
            }
            let _ = msg.ack().await;
        }
        Err(e) => {
            if e.is_transient() {
                tracing::warn!(
                    event_type = event_type,
                    error = %e,
                    "transient meter error, naking"
                );
                let _ = msg.ack_with(async_nats::jetstream::AckKind::Nak(None)).await;
            } else {
                tracing::error!(
                    event_type = event_type,
                    error = %e,
                    "permanent meter error, acking to skip"
                );
                let _ = msg.ack().await;
            }
        }
    }
}
