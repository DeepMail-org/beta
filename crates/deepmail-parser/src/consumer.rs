//! NATS JetStream consumer loop.
//!
//! Subscribes to `deepmail.jobs.parse`, processes messages concurrently
//! (bounded by config.concurrency), and ACKs/NAKs based on outcome.

use std::sync::Arc;
use std::time::Duration;

use async_nats::jetstream;
use futures::StreamExt;
use sqlx::PgPool;
use tracing::{error, info, warn};

use deepmail_common::nats::{self, subjects, NatsEnvelope};

use crate::config::Config;
use crate::db::insert::AttachmentRow;
use crate::error::ParserError;
use crate::{db, parse, s3};

/// Shared state passed into each message handler task.
pub struct ConsumerState {
    pub config: Arc<Config>,
    pub parser_pool: Arc<PgPool>,
    pub ingest_pool: Arc<PgPool>,
    pub s3_client: Arc<aws_sdk_s3::Client>,
    pub nats_js: Arc<jetstream::Context>,
}

/// NATS payload sent by ingest into `deepmail.jobs.parse`.
#[derive(Debug, serde::Deserialize)]
struct ParseJobPayload {
    /// S3 key of the quarantined email file.
    s3_key: String,
    /// S3 bucket of the quarantined email file.
    s3_bucket: String,
}

/// Run the consumer loop. This never returns under normal operation.
pub async fn run_consumer(state: Arc<ConsumerState>) -> Result<(), ParserError> {
    // ── Create / bind the stream ─────────────────────────────────
    let stream = state
        .nats_js
        .get_or_create_stream(jetstream::stream::Config {
            name: "DEEPMAIL_JOBS".into(),
            subjects: vec![
                subjects::JOBS_INGEST.into(),
                subjects::JOBS_PARSE.into(),
                subjects::JOBS_ANALYSIS.into(),
            ],
            retention: jetstream::stream::RetentionPolicy::WorkQueue,
            max_age: Duration::from_secs(86400 * 7), // 7 days
            storage: jetstream::stream::StorageType::File,
            ..Default::default()
        })
        .await
        .map_err(|e| ParserError::NatsPublish(format!("stream setup: {e}")))?;

    info!("bound to DEEPMAIL_JOBS stream");

    // ── Consumer (durable, pull-based) ───────────────────────────
    let consumer = stream
        .get_or_create_consumer(
            "parser-worker",
            jetstream::consumer::pull::Config {
                durable_name: Some("parser-worker".into()),
                filter_subject: subjects::JOBS_PARSE.into(),
                ack_policy: jetstream::consumer::AckPolicy::Explicit,
                ack_wait: Duration::from_secs(120),
                max_deliver: 5,
                ..Default::default()
            },
        )
        .await
        .map_err(|e| ParserError::NatsPublish(format!("consumer setup: {e}")))?;

    info!(
        subject = subjects::JOBS_PARSE,
        concurrency = state.config.concurrency,
        "parser consumer ready"
    );

    // ── Message loop ─────────────────────────────────────────────
    let mut messages = consumer
        .messages()
        .await
        .map_err(|e| ParserError::NatsPublish(format!("messages stream: {e}")))?;

    let semaphore = Arc::new(tokio::sync::Semaphore::new(state.config.concurrency));

    while let Some(Ok(msg)) = messages.next().await {
        let permit = Arc::clone(&semaphore)
            .acquire_owned()
            .await
            .map_err(|e| ParserError::Internal(format!("semaphore: {e}")))?;
        let state = Arc::clone(&state);

        tokio::spawn(async move {
            let _permit = permit;
            if let Err(e) = handle_message(&state, &msg).await {
                let nak_delay = Duration::from_secs(state.config.nak_delay_secs);
                if e.is_recoverable() {
                    warn!(error = %e, "recoverable parse error — NAK with delay");
                    if let Err(nak_err) = msg.ack_with(
                        async_nats::jetstream::AckKind::Nak(Some(nak_delay)),
                    ).await {
                        error!(error = %nak_err, "failed to NAK message");
                    }
                } else {
                    error!(error = %e, "permanent parse error — ACK (discard)");
                    if let Err(ack_err) = msg.ack().await {
                        error!(error = %ack_err, "failed to ACK message");
                    }
                }
            }
        });
    }

    warn!("NATS message stream ended — consumer shutting down");
    Ok(())
}

/// Handle a single NATS message: fetch → parse → persist → publish.
async fn handle_message(
    state: &ConsumerState,
    msg: &async_nats::jetstream::message::Message,
) -> Result<(), ParserError> {
    let started = std::time::Instant::now();

    // ── Decode envelope ──────────────────────────────────────────
    let envelope = NatsEnvelope::from_bytes(&msg.payload)
        .map_err(|e| ParserError::EnvelopeDecode(e.to_string()))?;

    let email_id = envelope.email_id;
    let tenant_id = envelope.tenant_id;
    let trace_id = envelope.trace_id.clone();

    info!(
        email_id = %email_id,
        tenant_id = %tenant_id,
        trace_id = %trace_id,
        "processing parse job"
    );

    // ── Decode payload ───────────────────────────────────────────
    let payload: ParseJobPayload =
        serde_json::from_value(envelope.payload.clone())
            .map_err(|e| ParserError::EnvelopeDecode(format!("payload: {e}")))?;

    // ── Mark parse as running ────────────────────────────────────
    let worker_id = format!("parser-{}", uuid::Uuid::new_v4());
    if let Err(e) = db::progress::mark_parse_running(
        &state.ingest_pool, email_id, &worker_id,
    ).await {
        warn!(error = %e, "failed to mark parse running (non-fatal)");
    }

    // ── Fetch raw email from S3 ──────────────────────────────────
    let raw_bytes = s3::fetch_quarantine(
        &state.s3_client,
        &payload.s3_bucket,
        &payload.s3_key,
    ).await?;

    info!(
        email_id = %email_id,
        size_bytes = raw_bytes.len(),
        "fetched quarantined email from S3"
    );

    // ── Parse (CPU-bound — offload) ──────────────────────────────
    let attachments_bucket = state.config.s3_attachments_bucket.clone();
    let parsed = {
        let raw_owned = raw_bytes;
        tokio::task::spawn_blocking(move || {
            parse::parse_email(
                &raw_owned,
                email_id,
                tenant_id,
                &attachments_bucket,
            )
        })
        .await
        .map_err(|e| ParserError::Internal(format!("spawn_blocking: {e}")))?
    }?;

    // ── Persist to parser DB ─────────────────────────────────────
    let parsed_email_id = db::insert::insert_parsed_email(
        &state.parser_pool,
        &parsed.email_row,
    ).await?;

    db::insert::insert_headers(
        &state.parser_pool,
        parsed_email_id,
        &parsed.headers,
    ).await?;

    db::insert::insert_received_hops(
        &state.parser_pool,
        parsed_email_id,
        &parsed.received_hops,
    ).await?;

    // ── Upload attachments to S3 + insert metadata ───────────────
    for att in &parsed.attachments {
        let s3_key = s3::upload_attachment(
            &state.s3_client,
            &state.config.s3_attachments_bucket,
            tenant_id,
            email_id,
            att.row.ordinal,
            att.row.filename.as_deref().unwrap_or("unnamed"),
            att.row.content_type.as_deref().unwrap_or("application/octet-stream"),
            att.bytes.clone(),
        ).await?;

        let row = AttachmentRow {
            email_id: att.row.email_id,
            tenant_id: att.row.tenant_id,
            filename: att.row.filename.clone(),
            content_type: att.row.content_type.clone(),
            size_bytes: att.row.size_bytes,
            sha256_hash: att.row.sha256_hash.clone(),
            s3_bucket: state.config.s3_attachments_bucket.clone(),
            s3_key,
            entropy: att.row.entropy,
            ordinal: att.row.ordinal,
        };

        // Use the real S3 key from upload
        db::insert::insert_attachment(
            &state.parser_pool,
            parsed_email_id,
            &row,
        ).await?;
    }

    // ── Publish to analysis pipeline ─────────────────────────────
    let analysis_payload = serde_json::json!({
        "parsed_email_id": parsed_email_id.to_string(),
        "attachment_count": parsed.email_row.attachment_count,
    });

    let analysis_envelope = NatsEnvelope::new(
        email_id,
        tenant_id,
        envelope.user_id,
        &trace_id,
        analysis_payload,
    );

    nats::publish_envelope(
        &state.nats_js,
        subjects::JOBS_ANALYSIS,
        &analysis_envelope,
    )
    .await
    .map_err(|e| ParserError::NatsPublish(e.to_string()))?;

    // ── Mark parse as completed ──────────────────────────────────
    let duration_ms = started.elapsed().as_millis() as i32;

    if let Err(e) = db::progress::mark_parse_completed(
        &state.ingest_pool, email_id, duration_ms,
    ).await {
        warn!(error = %e, "failed to mark parse completed (non-fatal)");
    }

    // ── ACK ──────────────────────────────────────────────────────
    msg.ack()
        .await
        .map_err(|e| ParserError::NatsPublish(format!("ack: {e}")))?;

    info!(
        email_id = %email_id,
        duration_ms = duration_ms,
        attachments = parsed.email_row.attachment_count,
        "parse complete"
    );

    Ok(())
}
