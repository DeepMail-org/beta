/// Full dynamic analysis pipeline orchestration.

use std::sync::Arc;
use std::time::Instant;

use sqlx::PgPool;
use uuid::Uuid;

use crate::cape::{CapeClient, CapeTaskStatus};
use crate::config::DynamicConfig;
use crate::db;
use crate::error::DynamicError;
use crate::fallback;
use crate::parser;
use crate::s3;
use crate::scorer;

/// Shared context for pipeline jobs.
#[allow(dead_code)]
pub struct JobCtx {
    pub pool: Arc<PgPool>,
    pub static_pool: Arc<PgPool>,
    pub ingest_pool: Arc<PgPool>,
    pub cape: Arc<CapeClient>,
    pub s3_client: Arc<aws_sdk_s3::Client>,
    pub s3_bucket: String,
    pub config: Arc<DynamicConfig>,
    pub nats: async_nats::Client,
}

/// Input for a single dynamic analysis job.
pub struct IncomingJob {
    pub attachment_id: Uuid,
    pub email_id: Uuid,
    pub tenant_id: Uuid,
    pub s3_key: String,
    pub filename: String,
    pub sha256_hash: Option<String>,
}

/// Run a complete dynamic analysis job.
pub async fn run_dynamic_job(
    ctx: Arc<JobCtx>,
    job: IncomingJob,
) -> Result<(), DynamicError> {
    tracing::info!(
        attachment_id = %job.attachment_id,
        filename = %job.filename,
        "starting dynamic analysis"
    );

    // ── a. Idempotency ─────────────────────────────────────────────────
    if let Some(existing) = db::jobs::get_by_attachment_id(&ctx.pool, job.attachment_id).await? {
        if existing.status == "completed" || existing.status == "running" {
            tracing::info!(
                attachment_id = %job.attachment_id,
                status = %existing.status,
                "job already exists, skipping"
            );
            return Ok(());
        }
    }

    // Create job record (ON CONFLICT → re-fetch)
    let job_id = db::jobs::create_job(
        &ctx.pool,
        job.email_id,
        job.tenant_id,
        job.attachment_id,
        &job.s3_key,
        &job.filename,
        job.sha256_hash.as_deref(),
    )
    .await?;

    // Re-check status after create (may have been created by race)
    if let Some(existing) = db::jobs::get_job(&ctx.pool, job_id).await? {
        if existing.status == "completed" || existing.status == "running" {
            return Ok(());
        }
    }

    db::jobs::increment_attempt(&ctx.pool, job_id).await?;

    // ── b. Download from MinIO ─────────────────────────────────────────
    let file_bytes =
        match s3::download_attachment(&ctx.s3_client, &ctx.s3_bucket, &job.s3_key).await {
            Ok(b) => b,
            Err(e) => {
                db::jobs::update_status(&ctx.pool, job_id, "failed", Some(&e.to_string())).await?;
                return Err(e);
            }
        };

    tracing::info!(
        attachment_id = %job.attachment_id,
        size = file_bytes.len(),
        "downloaded attachment"
    );

    // ── c/d. CAPEv2 or fallback ────────────────────────────────────────
    let findings = if ctx.cape.is_configured() {
        match run_cape_path(&ctx, job_id, &job, &file_bytes).await {
            Ok(f) => f,
            Err(DynamicError::CapeUnavailable) | Err(DynamicError::CapeApiError(..)) => {
                tracing::warn!(
                    attachment_id = %job.attachment_id,
                    "CAPE unavailable, using fallback"
                );
                db::jobs::mark_cape_unavailable(&ctx.pool, job_id).await?;
                fallback::fallback_from_static(
                    &ctx.static_pool,
                    job.attachment_id,
                    job.sha256_hash.as_deref(),
                )
                .await?
            }
            Err(e) => {
                db::jobs::update_status(&ctx.pool, job_id, "failed", Some(&e.to_string())).await?;
                return Err(e);
            }
        }
    } else {
        tracing::info!("CAPE not configured, using static fallback");
        db::jobs::mark_cape_unavailable(&ctx.pool, job_id).await?;
        fallback::fallback_from_static(
            &ctx.static_pool,
            job.attachment_id,
            job.sha256_hash.as_deref(),
        )
        .await?
    };

    // ── e. Score ────────────────────────────────────────────────────────
    let (score, verdict, notes) = scorer::compute_dynamic_score(&findings);

    tracing::info!(
        attachment_id = %job.attachment_id,
        verdict = verdict.as_str(),
        score = score,
        "dynamic analysis scored"
    );

    // ── f. Insert report ───────────────────────────────────────────────
    let http_json = serde_json::to_value(&findings.http_requests).unwrap_or_default();
    let dropped_json = serde_json::to_value(&findings.files_dropped).unwrap_or_default();
    let sigs_json = serde_json::to_value(&findings.cape_signatures).unwrap_or_default();

    let _report_id = db::reports::insert_report(
        &ctx.pool,
        job_id,
        job.email_id,
        job.tenant_id,
        job.attachment_id,
        &job.filename,
        job.sha256_hash.as_deref(),
        findings.malscore,
        findings.cape_unavailable,
        &findings.network_hosts,
        &findings.dns_requests,
        &http_json,
        findings.smtp_activity,
        &findings.processes_spawned,
        &dropped_json,
        &findings.registry_modifications,
        &findings.persistence_indicators,
        &findings.c2_indicators,
        &sigs_json,
        score,
        verdict.as_str(),
        &notes,
        None, // cape_report_s3_key set during CAPE path
    )
    .await?;

    // ── g. Mark job completed ──────────────────────────────────────────
    db::jobs::update_status(&ctx.pool, job_id, "completed", None).await?;

    // ── h. Update ingest job_progress ──────────────────────────────────
    let _ = sqlx::query(
        "UPDATE job_progress SET sandbox_dynamic_done = true, updated_at = now()
         WHERE email_id = $1",
    )
    .bind(job.email_id)
    .execute(ctx.ingest_pool.as_ref())
    .await;

    // ── i. Publish NATS event ──────────────────────────────────────────
    let event = serde_json::json!({
        "attachment_id": job.attachment_id.to_string(),
        "email_id": job.email_id.to_string(),
        "tenant_id": job.tenant_id.to_string(),
        "dynamic_verdict": verdict.as_str(),
        "dynamic_score": score,
        "cape_unavailable": findings.cape_unavailable,
    });

    if let Err(e) = ctx
        .nats
        .publish(
            "deepmail.events.sandbox.dynamic.completed",
            event.to_string().into(),
        )
        .await
    {
        tracing::warn!("failed to publish completion event: {}", e);
    }

    Ok(())
}

/// Run the CAPEv2 submission → poll → report path.
async fn run_cape_path(
    ctx: &JobCtx,
    job_id: Uuid,
    job: &IncomingJob,
    file_bytes: &[u8],
) -> Result<parser::DynamicFindings, DynamicError> {
    // Submit file to CAPEv2
    let cape_task_id = ctx.cape.submit_file(&job.filename, file_bytes).await?;

    tracing::info!(
        attachment_id = %job.attachment_id,
        cape_task_id,
        "submitted to CAPEv2"
    );

    db::jobs::update_cape_task_id(&ctx.pool, job_id, cape_task_id).await?;

    // Poll for completion
    let poll_interval = tokio::time::Duration::from_secs(ctx.config.cape_poll_interval_secs);
    let timeout_secs = ctx.config.cape_timeout_secs;
    let start = Instant::now();

    loop {
        tokio::time::sleep(poll_interval).await;

        let status = ctx.cape.poll_status(cape_task_id).await?;

        match status {
            CapeTaskStatus::Reported => break,
            CapeTaskStatus::Failed => {
                tracing::warn!(cape_task_id, "CAPE analysis failed");
                db::jobs::update_status(
                    &ctx.pool,
                    job_id,
                    "failed",
                    Some("CAPE analysis failed"),
                )
                .await?;
                // Fall through to CapeUnavailable for fallback
                return Err(DynamicError::CapeApiError(
                    0,
                    "CAPE analysis failed".into(),
                ));
            }
            CapeTaskStatus::Pending | CapeTaskStatus::Running => {
                db::jobs::update_status(&ctx.pool, job_id, "running", None).await?;
            }
            CapeTaskStatus::Unknown(s) => {
                tracing::debug!(cape_task_id, status = %s, "unknown CAPE status");
            }
        }

        if start.elapsed().as_secs() > timeout_secs {
            tracing::warn!(cape_task_id, "CAPE poll timeout");
            db::jobs::update_status(
                &ctx.pool,
                job_id,
                "timeout",
                Some(&format!("timeout after {}s", timeout_secs)),
            )
            .await?;
            return Err(DynamicError::CapeTimeout(cape_task_id));
        }
    }

    // Get full report
    let raw_report = ctx.cape.get_report(cape_task_id).await?;

    // Upload raw report to S3 (best-effort)
    let s3_key = format!(
        "dynamic/{}/{}/cape_report.json",
        job.tenant_id, job_id
    );
    let report_bytes = serde_json::to_vec(&raw_report).unwrap_or_default();
    if let Err(e) = s3::upload_report(&ctx.s3_client, &ctx.s3_bucket, &s3_key, &report_bytes).await
    {
        tracing::warn!("failed to upload CAPE report to S3: {}", e);
    }

    // Parse
    let findings = parser::parse_cape_report(&raw_report);

    // Delete task from CAPE (fire and forget)
    ctx.cape.delete_task(cape_task_id).await;

    Ok(findings)
}
