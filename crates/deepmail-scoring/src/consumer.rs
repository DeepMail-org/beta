use crate::db;
use crate::scorer::ScoringCtx;
use crate::signals::{self, SignalName};
use async_nats::jetstream;
use chrono::{DateTime, Utc};
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::Semaphore;

pub async fn run_consumers(ctx: Arc<ScoringCtx>, concurrency: usize, timeout_minutes: u64) {
    let js = jetstream::new(ctx.nats.clone());
    let sem = Arc::new(Semaphore::new(concurrency));

    let ctx1 = Arc::clone(&ctx);
    let sem1 = Arc::clone(&sem);
    let js1 = js.clone();
    let t1 = tokio::spawn(async move {
        run_event_consumer(js1, ctx1, sem1, timeout_minutes).await;
    });

    let ctx2 = Arc::clone(&ctx);
    let sem2 = Arc::clone(&sem);
    let js2 = js.clone();
    let t2 = tokio::spawn(async move {
        run_sandbox_consumer(js2, ctx2, sem2, timeout_minutes).await;
    });

    let _ = tokio::join!(t1, t2);
}

async fn run_event_consumer(
    js: jetstream::Context,
    ctx: Arc<ScoringCtx>,
    sem: Arc<Semaphore>,
    timeout_minutes: u64,
) {
    let stream = match js
        .get_or_create_stream(jetstream::stream::Config {
            name: "SCORING_EVENTS".to_string(),
            subjects: vec!["deepmail.events.*.completed".to_string()],
            retention: jetstream::stream::RetentionPolicy::Interest,
            ..Default::default()
        })
        .await
    {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("failed to get/create SCORING_EVENTS stream: {}", e);
            return;
        }
    };

    let consumer = match stream
        .get_or_create_consumer(
            "scoring-events",
            jetstream::consumer::pull::Config {
                durable_name: Some("scoring-events".to_string()),
                filter_subject: "deepmail.events.*.completed".to_string(),
                ack_policy: jetstream::consumer::AckPolicy::Explicit,
                ..Default::default()
            },
        )
        .await
    {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("failed to create scoring-events consumer: {}", e);
            return;
        }
    };

    let mut messages = match consumer.messages().await {
        Ok(m) => m,
        Err(e) => {
            tracing::error!("failed to subscribe scoring-events: {}", e);
            return;
        }
    };

    tracing::info!("scoring-events consumer started");

    while let Some(msg_result) = messages.next().await {
        let msg = match msg_result {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!("scoring-events message error: {}", e);
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
            handle_event(&ctx, &msg, timeout_minutes).await;
        });
    }
}

async fn run_sandbox_consumer(
    js: jetstream::Context,
    ctx: Arc<ScoringCtx>,
    sem: Arc<Semaphore>,
    timeout_minutes: u64,
) {
    let stream = match js
        .get_or_create_stream(jetstream::stream::Config {
            name: "SCORING_SANDBOX_EVENTS".to_string(),
            subjects: vec!["deepmail.events.sandbox.*.completed".to_string()],
            retention: jetstream::stream::RetentionPolicy::Interest,
            ..Default::default()
        })
        .await
    {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("failed to get/create SCORING_SANDBOX_EVENTS stream: {}", e);
            return;
        }
    };

    let consumer = match stream
        .get_or_create_consumer(
            "scoring-sandbox-events",
            jetstream::consumer::pull::Config {
                durable_name: Some("scoring-sandbox-events".to_string()),
                filter_subject: "deepmail.events.sandbox.*.completed".to_string(),
                ack_policy: jetstream::consumer::AckPolicy::Explicit,
                ..Default::default()
            },
        )
        .await
    {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("failed to create scoring-sandbox-events consumer: {}", e);
            return;
        }
    };

    let mut messages = match consumer.messages().await {
        Ok(m) => m,
        Err(e) => {
            tracing::error!("failed to subscribe scoring-sandbox-events: {}", e);
            return;
        }
    };

    tracing::info!("scoring-sandbox-events consumer started");

    while let Some(msg_result) = messages.next().await {
        let msg = match msg_result {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!("scoring-sandbox-events message error: {}", e);
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
            handle_event(&ctx, &msg, timeout_minutes).await;
        });
    }
}

async fn handle_event(
    ctx: &Arc<ScoringCtx>,
    msg: &async_nats::jetstream::Message,
    timeout_minutes: u64,
) {
    let subject = msg.subject.as_str();

    let signal = match signals::subject_to_signal(subject) {
        Some(s) => s,
        None => {
            tracing::warn!(subject, "unknown signal subject, acking");
            let _ = msg.ack().await;
            return;
        }
    };

    let payload = match signals::parse_event_payload(&msg.payload) {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!(subject, error = %e, "bad event payload, acking");
            let _ = msg.ack().await;
            return;
        }
    };

    if let Some(score) = payload.score {
        if let Err(e) = db::signals::upsert_signal(
            &ctx.pool,
            payload.email_id,
            payload.tenant_id,
            signal.as_str(),
            score,
            &serde_json::json!({}),
        )
        .await
        {
            tracing::error!(email_id = %payload.email_id, error = %e, "failed to upsert signal");
            let _ = msg.ack_with(jetstream::AckKind::Nak(None)).await;
            return;
        }
    }

    let should_compute = match should_compute_now(ctx, payload.email_id, timeout_minutes).await {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(email_id = %payload.email_id, error = %e, "failed check, skipping compute");
            let _ = msg.ack().await;
            return;
        }
    };

    if should_compute {
        match crate::scorer::compute_and_store(ctx, payload.email_id, payload.tenant_id, false)
            .await
        {
            Ok(result) => {
                tracing::info!(
                    email_id = %payload.email_id,
                    score = result.final_score,
                    verdict = result.final_verdict.as_str(),
                    signals = result.signals_available,
                    "score computed"
                );
            }
            Err(e) => {
                tracing::error!(email_id = %payload.email_id, error = %e, "scoring failed");
            }
        }
    }

    let _ = msg.ack().await;
}

async fn should_compute_now(
    ctx: &ScoringCtx,
    email_id: uuid::Uuid,
    timeout_minutes: u64,
) -> Result<bool, crate::error::ScoringError> {
    let count = db::signals::count_by_email(&ctx.pool, email_id).await?;

    if count >= SignalName::all().len() as i64 {
        return Ok(true);
    }

    if count >= 5 {
        let row: Option<(DateTime<Utc>,)> =
            sqlx::query_as("SELECT uploaded_at FROM emails WHERE id = $1")
                .bind(email_id)
                .fetch_optional(ctx.ingest_pool.as_ref())
                .await?;

        if let Some((uploaded_at,)) = row {
            let elapsed = Utc::now() - uploaded_at;
            if elapsed.num_minutes() >= timeout_minutes as i64 {
                return Ok(true);
            }
        }
    }

    Ok(false)
}
