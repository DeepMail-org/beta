use crate::dispatcher::{self, NotifyCtx};
use crate::pipeline;
use async_nats::jetstream;
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::Semaphore;

struct ConsumerDef {
    stream_name: &'static str,
    subject: &'static str,
    durable: &'static str,
}

const CONSUMERS: &[ConsumerDef] = &[
    ConsumerDef {
        stream_name: "NOTIFY_SCORING_EVENTS",
        subject: "deepmail.events.scoring.completed",
        durable: "notify-scoring-events",
    },
    ConsumerDef {
        stream_name: "NOTIFY_REPORT_EVENTS",
        subject: "deepmail.events.report.completed",
        durable: "notify-report-events",
    },
    ConsumerDef {
        stream_name: "NOTIFY_IOC_EVENTS",
        subject: "deepmail.events.ioc.completed",
        durable: "notify-ioc-events",
    },
    ConsumerDef {
        stream_name: "NOTIFY_SANDBOX_EVENTS",
        subject: "deepmail.events.sandbox.dynamic.completed",
        durable: "notify-sandbox-events",
    },
];

pub async fn run_consumers(nats: async_nats::Client, ctx: Arc<NotifyCtx>) {
    let js = jetstream::new(nats);

    for def in CONSUMERS {
        let js = js.clone();
        let ctx = Arc::clone(&ctx);
        tokio::spawn(async move {
            run_single_consumer(js, ctx, def).await;
        });
    }
}

async fn run_single_consumer(
    js: jetstream::Context,
    ctx: Arc<NotifyCtx>,
    def: &'static ConsumerDef,
) {
    let stream = match js
        .get_or_create_stream(jetstream::stream::Config {
            name: def.stream_name.to_string(),
            subjects: vec![def.subject.to_string()],
            retention: jetstream::stream::RetentionPolicy::Interest,
            ..Default::default()
        })
        .await
    {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(stream = def.stream_name, error = %e, "failed to get/create stream");
            return;
        }
    };

    let consumer = match stream
        .get_or_create_consumer(
            def.durable,
            jetstream::consumer::pull::Config {
                durable_name: Some(def.durable.to_string()),
                filter_subject: def.subject.to_string(),
                ack_policy: jetstream::consumer::AckPolicy::Explicit,
                ..Default::default()
            },
        )
        .await
    {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(durable = def.durable, error = %e, "failed to create consumer");
            return;
        }
    };

    let mut messages = match consumer.messages().await {
        Ok(m) => m,
        Err(e) => {
            tracing::error!(durable = def.durable, error = %e, "failed to subscribe");
            return;
        }
    };

    tracing::info!(durable = def.durable, subject = def.subject, "consumer started");

    let sem = Arc::new(Semaphore::new(10));

    while let Some(msg_result) = messages.next().await {
        let msg = match msg_result {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!(durable = def.durable, error = %e, "message error");
                continue;
            }
        };

        let permit = match sem.clone().acquire_owned().await {
            Ok(p) => p,
            Err(_) => break,
        };

        let ctx = Arc::clone(&ctx);
        let subject = def.subject;
        tokio::spawn(async move {
            let _permit = permit;
            handle_message(&ctx, &msg, subject).await;
        });
    }
}

async fn handle_message(ctx: &NotifyCtx, msg: &jetstream::Message, subject: &str) {
    let event = match subject {
        "deepmail.events.scoring.completed" => {
            match pipeline::parse_scoring_event(&msg.payload) {
                Ok(e) => Some(e),
                Err(e) => {
                    tracing::warn!(error = %e, subject = subject, "bad payload, acking");
                    let _ = msg.ack().await;
                    return;
                }
            }
        }
        "deepmail.events.report.completed" => {
            match pipeline::parse_report_event(&msg.payload) {
                Ok(e) => Some(e),
                Err(e) => {
                    tracing::warn!(error = %e, subject = subject, "bad payload, acking");
                    let _ = msg.ack().await;
                    return;
                }
            }
        }
        "deepmail.events.ioc.completed" => pipeline::parse_ioc_event(&msg.payload),
        "deepmail.events.sandbox.dynamic.completed" => {
            pipeline::parse_dynamic_event(&msg.payload)
        }
        _ => {
            let _ = msg.ack().await;
            return;
        }
    };

    match event {
        Some(evt) => {
            let result = dispatcher::dispatch_event(ctx, &evt).await;
            tracing::info!(
                email_id = %evt.email_id,
                event_type = evt.event_type.as_str(),
                ws = result.websocket_sent,
                ws_count = result.ws_recipients,
                email = result.email_sent,
                webhook = result.webhook_sent,
                "dispatch complete"
            );
            let _ = msg.ack().await;
        }
        None => {
            let _ = msg.ack().await;
        }
    }
}
