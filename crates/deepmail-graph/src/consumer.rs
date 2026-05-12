use crate::sync::SyncCtx;
use async_nats::jetstream;
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::Semaphore;
use uuid::Uuid;

#[derive(serde::Deserialize)]
struct EventPayload {
    email_id: String,
    tenant_id: String,
}

pub async fn run_consumers(ctx: Arc<SyncCtx>, nats: async_nats::Client, concurrency: usize) {
    let js = jetstream::new(nats);
    let sem = Arc::new(Semaphore::new(concurrency));

    let ctx1 = Arc::clone(&ctx);
    let sem1 = Arc::clone(&sem);
    let js1 = js.clone();
    let t1 = tokio::spawn(async move {
        run_scoring_consumer(js1, ctx1, sem1).await;
    });

    let ctx2 = Arc::clone(&ctx);
    let sem2 = Arc::clone(&sem);
    let js2 = js.clone();
    let t2 = tokio::spawn(async move {
        run_ioc_consumer(js2, ctx2, sem2).await;
    });

    let _ = tokio::join!(t1, t2);
}

async fn run_scoring_consumer(
    js: jetstream::Context,
    ctx: Arc<SyncCtx>,
    sem: Arc<Semaphore>,
) {
    let stream = match js
        .get_or_create_stream(jetstream::stream::Config {
            name: "GRAPH_SCORING_EVENTS".to_string(),
            subjects: vec!["deepmail.events.scoring.completed".to_string()],
            retention: jetstream::stream::RetentionPolicy::Interest,
            ..Default::default()
        })
        .await
    {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("failed to get/create GRAPH_SCORING_EVENTS stream: {}", e);
            return;
        }
    };

    let consumer = match stream
        .get_or_create_consumer(
            "graph-scoring-events",
            jetstream::consumer::pull::Config {
                durable_name: Some("graph-scoring-events".to_string()),
                filter_subject: "deepmail.events.scoring.completed".to_string(),
                ack_policy: jetstream::consumer::AckPolicy::Explicit,
                ..Default::default()
            },
        )
        .await
    {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("failed to create graph-scoring-events consumer: {}", e);
            return;
        }
    };

    let mut messages = match consumer.messages().await {
        Ok(m) => m,
        Err(e) => {
            tracing::error!("failed to subscribe graph-scoring-events: {}", e);
            return;
        }
    };

    tracing::info!("graph-scoring-events consumer started");

    while let Some(msg_result) = messages.next().await {
        let msg = match msg_result {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!("graph-scoring-events message error: {}", e);
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

async fn run_ioc_consumer(
    js: jetstream::Context,
    ctx: Arc<SyncCtx>,
    sem: Arc<Semaphore>,
) {
    let stream = match js
        .get_or_create_stream(jetstream::stream::Config {
            name: "GRAPH_IOC_EVENTS".to_string(),
            subjects: vec!["deepmail.events.ioc.completed".to_string()],
            retention: jetstream::stream::RetentionPolicy::Interest,
            ..Default::default()
        })
        .await
    {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("failed to get/create GRAPH_IOC_EVENTS stream: {}", e);
            return;
        }
    };

    let consumer = match stream
        .get_or_create_consumer(
            "graph-ioc-events",
            jetstream::consumer::pull::Config {
                durable_name: Some("graph-ioc-events".to_string()),
                filter_subject: "deepmail.events.ioc.completed".to_string(),
                ack_policy: jetstream::consumer::AckPolicy::Explicit,
                ..Default::default()
            },
        )
        .await
    {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("failed to create graph-ioc-events consumer: {}", e);
            return;
        }
    };

    let mut messages = match consumer.messages().await {
        Ok(m) => m,
        Err(e) => {
            tracing::error!("failed to subscribe graph-ioc-events: {}", e);
            return;
        }
    };

    tracing::info!("graph-ioc-events consumer started");

    while let Some(msg_result) = messages.next().await {
        let msg = match msg_result {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!("graph-ioc-events message error: {}", e);
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

async fn handle_message(ctx: &SyncCtx, msg: &async_nats::jetstream::Message) {
    let payload: EventPayload = match serde_json::from_slice(&msg.payload) {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!(error = %e, "bad event payload, acking");
            let _ = msg.ack().await;
            return;
        }
    };

    let email_id = match Uuid::parse_str(&payload.email_id) {
        Ok(id) => id,
        Err(e) => {
            tracing::warn!(error = %e, "invalid email_id in payload, acking");
            let _ = msg.ack().await;
            return;
        }
    };

    let tenant_id = match Uuid::parse_str(&payload.tenant_id) {
        Ok(id) => id,
        Err(e) => {
            tracing::warn!(error = %e, "invalid tenant_id in payload, acking");
            let _ = msg.ack().await;
            return;
        }
    };

    match crate::sync::sync_email(ctx, email_id, tenant_id).await {
        Ok((nodes, edges)) => {
            tracing::info!(
                %email_id,
                nodes_created = nodes,
                edges_created = edges,
                "graph sync completed"
            );
            let _ = msg.ack().await;
        }
        Err(e) if e.is_transient() => {
            tracing::warn!(%email_id, error = %e, "transient sync error, naking for retry");
            let _ = msg.ack_with(jetstream::AckKind::Nak(None)).await;
        }
        Err(e) => {
            tracing::error!(%email_id, error = %e, "sync failed permanently, acking");
            let _ = msg.ack().await;
        }
    }
}
