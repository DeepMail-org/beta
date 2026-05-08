/// Full URL sandbox pipeline: orchestrates Docker container, analysis, classification.

use std::sync::Arc;

use bollard::Docker;
use sqlx::PgPool;
use uuid::Uuid;

use crate::classifier;
use crate::config::SandboxUrlConfig;
use crate::db;
use crate::docker;
use crate::error::SandboxUrlError;
use crate::qr;
use crate::s3;

/// Shared context for pipeline jobs.
pub struct JobCtx {
    pub pool: Arc<PgPool>,
    pub docker: Arc<Docker>,
    pub s3_client: Arc<aws_sdk_s3::Client>,
    pub s3_bucket: String,
    pub config: Arc<SandboxUrlConfig>,
    pub nats: async_nats::Client,
}

/// Input for a single URL sandbox job.
pub struct UrlSandboxJob {
    pub job_id: Uuid,
    pub email_id: Uuid,
    pub tenant_id: Uuid,
    pub url: String,
    pub url_type: String,
}

/// Run a complete URL sandbox job.
pub async fn run_url_job(
    ctx: Arc<JobCtx>,
    job: UrlSandboxJob,
) -> Result<(), SandboxUrlError> {
    tracing::info!(
        job_id = %job.job_id,
        url = %job.url,
        url_type = %job.url_type,
        "starting sandbox job"
    );

    // ── QR candidate handling ───────────────────────────────────────────
    if job.url_type == "qr_candidate" {
        return handle_qr_candidate(&ctx, &job).await;
    }

    // ── Regular URL sandboxing ──────────────────────────────────────────
    run_browser_sandbox(&ctx, &job).await
}

/// Handle QR code candidate: download image, decode, publish sub-jobs.
async fn handle_qr_candidate(
    ctx: &JobCtx,
    job: &UrlSandboxJob,
) -> Result<(), SandboxUrlError> {
    let image_bytes = if job.url.starts_with("data:") {
        // Data URI: decode base64 payload
        decode_data_uri(&job.url)?
    } else {
        // External URL: download
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|e| SandboxUrlError::HttpFetch(e.to_string()))?;

        client
            .get(&job.url)
            .send()
            .await
            .map_err(|e| SandboxUrlError::HttpFetch(e.to_string()))?
            .bytes()
            .await
            .map_err(|e| SandboxUrlError::HttpFetch(e.to_string()))?
            .to_vec()
    };

    let decoded_urls = qr::decode_qr_from_bytes(&image_bytes);

    if decoded_urls.is_empty() {
        tracing::info!(job_id = %job.job_id, "no QR codes decoded from image");
        // Complete with empty benign report
        insert_benign_report(ctx, job, &[]).await?;
        db::jobs::update_status(&ctx.pool, job.job_id, "completed", None).await?;
        publish_completion(ctx, job, "benign", 0.0).await;
        return Ok(());
    }

    tracing::info!(
        job_id = %job.job_id,
        count = decoded_urls.len(),
        "decoded QR URLs"
    );

    // Publish sub-jobs for decoded URLs
    for decoded_url in &decoded_urls {
        let payload = serde_json::json!({
            "email_id": job.email_id.to_string(),
            "tenant_id": job.tenant_id.to_string(),
            "url": decoded_url,
            "url_type": "qr_decoded",
        });

        if let Err(e) = ctx.nats.publish(
            "deepmail.jobs.sandbox.url",
            payload.to_string().into(),
        ).await {
            tracing::warn!("failed to publish QR decoded sub-job: {}", e);
        }
    }

    // Complete this job with report noting decoded URLs
    insert_benign_report(ctx, job, &decoded_urls).await?;
    db::jobs::update_status(&ctx.pool, job.job_id, "completed", None).await?;
    publish_completion(ctx, job, "benign", 0.0).await;

    Ok(())
}

/// Run the full browser sandbox for a URL.
async fn run_browser_sandbox(
    ctx: &JobCtx,
    job: &UrlSandboxJob,
) -> Result<(), SandboxUrlError> {
    // Create temp dir for results volume
    let results_dir = tempfile::TempDir::new()
        .map_err(|e| SandboxUrlError::Docker(format!("create tempdir: {}", e)))?;

    // Spawn container
    let container_id = docker::spawn_analysis_container(
        &ctx.docker,
        &ctx.config,
        &job.url,
        results_dir.path(),
    ).await?;

    // Update DB with container info
    db::jobs::update_started(&ctx.pool, job.job_id, &container_id).await?;

    // Wait for container with timeout — always cleanup
    let analysis_result = run_container_and_analyze(
        ctx, job, &container_id, results_dir.path(),
    ).await;

    // Always cleanup container
    docker::cleanup_container(&ctx.docker, &container_id).await;

    analysis_result
}

/// Run the container, wait, read results, classify, persist.
async fn run_container_and_analyze(
    ctx: &JobCtx,
    job: &UrlSandboxJob,
    container_id: &str,
    results_dir: &std::path::Path,
) -> Result<(), SandboxUrlError> {
    // Wait for container
    match docker::wait_for_container(
        &ctx.docker,
        container_id,
        ctx.config.container_timeout_secs,
    ).await {
        Ok(exit_code) => {
            tracing::info!(
                job_id = %job.job_id,
                exit_code = exit_code,
                "container finished"
            );
        }
        Err(SandboxUrlError::Timeout(secs)) => {
            db::jobs::update_status(
                &ctx.pool,
                job.job_id,
                "timeout",
                Some(&format!("container timed out after {}s", secs)),
            ).await?;
            publish_completion(ctx, job, "suspicious", 0.10).await;
            return Ok(());
        }
        Err(e) => return Err(e),
    }

    // Read results from container volume
    let pw_result = match docker::read_results(results_dir) {
        Ok(r) => r,
        Err(_) => {
            tracing::warn!(job_id = %job.job_id, "no results from container, using empty result");
            docker::RawPlaywrightResult {
                error: Some("Container produced no results".into()),
                ..Default::default()
            }
        }
    };

    // Read screenshot
    let screenshot_bytes = docker::read_screenshot(results_dir);

    // Upload screenshot to S3
    let screenshot_key = if let Some(bytes) = screenshot_bytes {
        let key = format!(
            "sandbox/url/{}/{}/screenshot.png",
            job.tenant_id, job.job_id
        );
        match s3::upload_screenshot(&ctx.s3_client, &ctx.s3_bucket, &key, bytes).await {
            Ok(k) => Some(k),
            Err(e) => {
                tracing::warn!(job_id = %job.job_id, "screenshot upload failed: {}", e);
                None
            }
        }
    } else {
        None
    };

    // Log image-type network requests for future QR scanning
    for req in &pw_result.network_requests {
        if req.resource_type == "image" {
            tracing::debug!(
                job_id = %job.job_id,
                url = %req.url,
                "image resource loaded (QR scanning reserved for future iteration)"
            );
        }
    }

    // Classify the page
    let (threat_class, threat_score, notes) =
        classifier::classify_page(&pw_result, &job.url);

    // Prepare JSONB values
    let redirect_chain_json = serde_json::to_value(&pw_result.redirect_chain)
        .unwrap_or(serde_json::Value::Array(Vec::new()));
    let network_requests_json = serde_json::to_value(&pw_result.network_requests)
        .unwrap_or(serde_json::Value::Array(Vec::new()));
    let cookies_json = serde_json::to_value(&pw_result.cookies)
        .unwrap_or(serde_json::Value::Array(Vec::new()));
    let external_scripts_json = serde_json::to_value(&pw_result.external_scripts)
        .unwrap_or(serde_json::Value::Array(Vec::new()));
    let iframes_json = serde_json::to_value(&pw_result.iframes)
        .unwrap_or(serde_json::Value::Array(Vec::new()));
    let js_dialogs_json = serde_json::to_value(&pw_result.js_dialogs)
        .unwrap_or(serde_json::Value::Array(Vec::new()));

    // Insert report
    db::reports::insert_report(
        &ctx.pool,
        job.job_id,
        job.email_id,
        job.tenant_id,
        &job.url,
        pw_result.final_url.as_deref(),
        pw_result.redirect_chain.len() as i32,
        &redirect_chain_json,
        pw_result.title.as_deref(),
        &network_requests_json,
        &cookies_json,
        &external_scripts_json,
        &iframes_json,
        pw_result.has_password_field,
        pw_result.has_email_field,
        pw_result.has_login_form,
        pw_result.has_download_trigger,
        &js_dialogs_json,
        &[],  // qr_decoded_urls — not applicable for browser sandbox jobs
        screenshot_key.as_deref(),
        threat_class.as_str(),
        threat_score,
        &notes,
    ).await?;

    // Update job status
    db::jobs::update_status(&ctx.pool, job.job_id, "completed", None).await?;

    tracing::info!(
        job_id = %job.job_id,
        threat_class = %threat_class.as_str(),
        threat_score = %threat_score,
        "sandbox analysis complete"
    );

    // Publish completion event
    publish_completion(ctx, job, threat_class.as_str(), threat_score).await;

    Ok(())
}

// ─── Helpers ────────────────────────────────────────────────────────────────

/// Decode a data URI's base64 payload to bytes.
fn decode_data_uri(data_uri: &str) -> Result<Vec<u8>, SandboxUrlError> {
    // Format: data:image/png;base64,<data>
    let parts: Vec<&str> = data_uri.splitn(2, ',').collect();
    if parts.len() != 2 {
        return Err(SandboxUrlError::InvalidPayload(
            "invalid data URI format".into(),
        ));
    }
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(parts[1])
        .map_err(|e| SandboxUrlError::InvalidPayload(format!("base64 decode: {}", e)))
}

/// Insert a benign report (used for QR decoding results).
async fn insert_benign_report(
    ctx: &JobCtx,
    job: &UrlSandboxJob,
    qr_decoded_urls: &[String],
) -> Result<Uuid, SandboxUrlError> {
    let empty_json = serde_json::Value::Array(Vec::new());

    db::reports::insert_report(
        &ctx.pool,
        job.job_id,
        job.email_id,
        job.tenant_id,
        &job.url,
        None,
        0,
        &empty_json,
        None,
        &empty_json,
        &empty_json,
        &empty_json,
        &empty_json,
        false,
        false,
        false,
        false,
        &empty_json,
        qr_decoded_urls,
        None,
        "benign",
        0.0,
        &[],
    )
    .await
    .map_err(SandboxUrlError::from)
}

/// Publish a completion event to NATS.
async fn publish_completion(
    ctx: &JobCtx,
    job: &UrlSandboxJob,
    threat_class: &str,
    threat_score: f32,
) {
    let payload = serde_json::json!({
        "job_id": job.job_id.to_string(),
        "email_id": job.email_id.to_string(),
        "tenant_id": job.tenant_id.to_string(),
        "threat_class": threat_class,
        "threat_score": threat_score,
    });

    if let Err(e) = ctx.nats.publish(
        "deepmail.events.sandbox.url.completed",
        payload.to_string().into(),
    ).await {
        tracing::warn!(job_id = %job.job_id, "failed to publish completion event: {}", e);
    }
}
