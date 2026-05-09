/// Full static file analysis pipeline.

use std::io::Write;
use std::sync::Arc;

use sha2::Digest;
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::SandboxFileConfig;
use crate::db;
use crate::entropy;
use crate::error::SandboxFileError;
use crate::s3;
use crate::scorer::{self, FileFindings};
use crate::tools::{binwalk, exiftool, file_cmd, oletools, pdfid, pefile, strings_cmd, yara_scan};

/// Shared context for pipeline jobs.
#[allow(dead_code)]
pub struct JobCtx {
    pub pool: Arc<PgPool>,
    pub parser_pool: Arc<PgPool>,
    pub ingest_pool: Arc<PgPool>,
    pub s3_client: Arc<aws_sdk_s3::Client>,
    pub s3_bucket: String,
    pub yara_rules: Arc<yara_x::Rules>,
    pub config: Arc<SandboxFileConfig>,
    pub nats: async_nats::Client,
}

/// Input for a single file analysis job.
pub struct IncomingJob {
    pub attachment_id: Uuid,
    pub email_id: Uuid,
    pub tenant_id: Uuid,
    pub s3_key: String,
    pub filename: String,
}

/// Run a complete file analysis job.
pub async fn run_file_job(
    ctx: Arc<JobCtx>,
    job: IncomingJob,
) -> Result<db::reports::FileReportRow, SandboxFileError> {
    tracing::info!(
        attachment_id = %job.attachment_id,
        filename = %job.filename,
        "starting file analysis"
    );

    // ── a. Idempotency check ────────────────────────────────────────────
    if let Some(existing) = db::reports::get_by_attachment_id(&ctx.pool, job.attachment_id).await? {
        tracing::info!(attachment_id = %job.attachment_id, "already analyzed, returning existing");
        return Ok(existing);
    }

    // ── b. Download file from MinIO ─────────────────────────────────────
    let file_bytes = s3::download_attachment(&ctx.s3_client, &ctx.s3_bucket, &job.s3_key).await?;
    let file_size = file_bytes.len() as i64;

    tracing::info!(
        attachment_id = %job.attachment_id,
        size = file_size,
        "downloaded attachment"
    );

    // ── c. Write to tempfile ────────────────────────────────────────────
    let mut tmp = tempfile::NamedTempFile::new()?;
    tmp.write_all(&file_bytes)?;
    tmp.flush()?;
    let tmp_path = tmp.path().to_path_buf();

    // ── d. Compute hashes ───────────────────────────────────────────────
    let sha256 = hex::encode(sha2::Sha256::digest(&file_bytes));
    let md5 = hex::encode(md5::Md5::digest(&file_bytes));

    // ── e. Compute entropy ──────────────────────────────────────────────
    let file_entropy = entropy::compute_entropy(&file_bytes);

    // ── f. Run all tools concurrently ───────────────────────────────────
    let timeout = ctx.config.tool_timeout_secs;

    // Determine file type hints for conditional tools
    let is_likely_pe = file_bytes.len() >= 2 && file_bytes[0] == 0x4D && file_bytes[1] == 0x5A;
    let is_likely_pdf = file_bytes.len() >= 4 && &file_bytes[..4] == b"%PDF";
    let mime_lower = job.filename.to_lowercase();
    let is_likely_ole = mime_lower.ends_with(".doc")
        || mime_lower.ends_with(".docx")
        || mime_lower.ends_with(".xls")
        || mime_lower.ends_with(".xlsx")
        || mime_lower.ends_with(".ppt")
        || mime_lower.ends_with(".pptx")
        || mime_lower.ends_with(".docm")
        || mime_lower.ends_with(".xlsm");

    // Run tools concurrently
    let tmp1 = tmp_path.clone();
    let file_handle = tokio::spawn(async move {
        file_cmd::run_file_command(&tmp1, timeout).await
    });

    let tmp2 = tmp_path.clone();
    let exif_handle = tokio::spawn(async move {
        exiftool::run_exiftool(&tmp2, timeout).await
    });

    let tmp3 = tmp_path.clone();
    let strings_handle = tokio::spawn(async move {
        strings_cmd::run_strings(&tmp3, timeout).await
    });

    let tmp4 = tmp_path.clone();
    let binwalk_handle = tokio::spawn(async move {
        binwalk::run_binwalk(&tmp4, timeout).await
    });

    let tmp5 = tmp_path.clone();
    let pdfid_handle = if is_likely_pdf {
        Some(tokio::spawn(async move {
            pdfid::run_pdfid(&tmp5, timeout).await
        }))
    } else {
        None
    };

    let tmp6 = tmp_path.clone();
    let tmp7 = tmp_path.clone();
    let (olevba_handle, oleid_handle) = if is_likely_ole {
        (
            Some(tokio::spawn(async move {
                oletools::run_olevba(&tmp6, timeout).await
            })),
            Some(tokio::spawn(async move {
                oletools::run_oleid(&tmp7, timeout).await
            })),
        )
    } else {
        (None, None)
    };

    let tmp8 = tmp_path.clone();
    let pe_handle = if is_likely_pe {
        Some(tokio::spawn(async move {
            pefile::run_pefile(&tmp8, timeout).await
        }))
    } else {
        None
    };

    // YARA scan — always run
    let yara_rules = Arc::clone(&ctx.yara_rules);
    let yara_data = file_bytes.clone();
    let yara_handle = tokio::spawn(async move {
        yara_scan::run_yara_scan(&yara_rules, &yara_data).await
    });

    // ── Collect results ─────────────────────────────────────────────────
    let file_result = file_handle.await.ok().and_then(|r| r.ok())
        .unwrap_or_default();

    let exif_result = exif_handle.await.ok().and_then(|r| r.ok())
        .unwrap_or_default();

    let strings_result = strings_handle.await.ok().and_then(|r| r.ok())
        .unwrap_or_default();

    let binwalk_result = binwalk_handle.await.ok().and_then(|r| r.ok())
        .unwrap_or_default();

    let pdfid_result = if let Some(handle) = pdfid_handle {
        handle.await.ok().and_then(|r| r.ok()).unwrap_or_default()
    } else {
        pdfid::PdfidResult::default()
    };

    let olevba_result = if let Some(handle) = olevba_handle {
        handle.await.ok().and_then(|r| r.ok()).unwrap_or_default()
    } else {
        oletools::OleResult::default()
    };

    let oleid_result = if let Some(handle) = oleid_handle {
        handle.await.ok().and_then(|r| r.ok()).unwrap_or_default()
    } else {
        oletools::OleIdResult::default()
    };

    let pe_result = if let Some(handle) = pe_handle {
        handle.await.ok().and_then(|r| r.ok()).unwrap_or_default()
    } else {
        pefile::PeResult::default()
    };

    let yara_matches = yara_handle.await.ok().and_then(|r| r.ok())
        .unwrap_or_default();

    // ── g. Build FileFindings ───────────────────────────────────────────
    let has_macros = olevba_result.has_macros || oleid_result.has_macros;

    let findings = FileFindings {
        entropy: file_entropy,
        has_macros,
        has_vba: olevba_result.has_vba,
        vba_suspicious: olevba_result.is_suspicious,
        is_pe: pe_result.is_pe,
        pe_is_packed: pe_result.is_packed,
        pe_suspicious_imports: pe_result.suspicious_imports.clone(),
        is_pdf: pdfid_result.is_pdf || is_likely_pdf,
        pdf_has_js: pdfid_result.js_count > 0,
        pdf_has_launch: pdfid_result.launch_count > 0,
        pdf_is_encrypted: pdfid_result.encrypt_count > 0,
        has_embedded: binwalk_result.has_embedded,
        yara_matches: yara_matches.clone(),
        suspicious_strings: strings_result.suspicious.clone(),
        macro_count: olevba_result.macro_count,
        any_tool_ran: true, // file command always available on Linux
    };

    // Build tool_outputs JSONB (truncate each to 10KB)
    let truncate = |s: &str| -> String {
        if s.len() > 10240 {
            format!("{}... [truncated]", &s[..10240])
        } else {
            s.to_string()
        }
    };
    let tool_outputs = serde_json::json!({
        "file": truncate(&format!("{} | {}", file_result.mime_type, file_result.magic)),
        "exiftool": truncate(&serde_json::to_string(&exif_result.raw).unwrap_or_default()),
        "strings_count": strings_result.total_count,
        "binwalk_count": binwalk_result.embedded_files.len(),
        "pdfid": truncate(&format!("js={} launch={} embed={} enc={}",
            pdfid_result.js_count, pdfid_result.launch_count,
            pdfid_result.embedded_count, pdfid_result.encrypt_count)),
        "olevba": truncate(&format!("macros={} suspicious={} autoexec={}",
            olevba_result.macro_count, olevba_result.is_suspicious, olevba_result.has_autoexec)),
        "pe": truncate(&format!("is_pe={} packed={} signed={} sections={}",
            pe_result.is_pe, pe_result.is_packed, pe_result.has_signature, pe_result.num_sections)),
        "yara": yara_matches,
    });

    // ── h. Compute threat score ─────────────────────────────────────────
    let (threat_score, verdict, notes) = scorer::compute_threat_score(&findings);

    tracing::info!(
        attachment_id = %job.attachment_id,
        verdict = %verdict.as_str(),
        score = %threat_score,
        "analysis complete"
    );

    // ── i. Insert report ────────────────────────────────────────────────
    let embedded_json = serde_json::to_value(&binwalk_result.embedded_files)
        .unwrap_or(serde_json::Value::Array(Vec::new()));

    // Take top 20 suspicious strings
    let top_suspicious: Vec<String> = strings_result.suspicious.into_iter().take(20).collect();

    let report = db::reports::insert_report(
        &ctx.pool,
        job.email_id,
        job.tenant_id,
        job.attachment_id,
        &job.s3_key,
        &job.filename,
        file_size,
        Some(&file_result.mime_type),
        Some(&file_result.magic),
        Some(&sha256),
        Some(&md5),
        file_entropy,
        has_macros,
        olevba_result.macro_count,
        olevba_result.has_vba,
        olevba_result.is_suspicious,
        pe_result.is_pe,
        pe_result.has_signature,
        pe_result.is_packed,
        &pe_result.suspicious_imports,
        pdfid_result.is_pdf || is_likely_pdf,
        pdfid_result.js_count > 0,
        pdfid_result.launch_count > 0,
        pdfid_result.embedded_count > 0,
        pdfid_result.encrypt_count > 0,
        &embedded_json,
        &yara_matches,
        &top_suspicious,
        exif_result.author.as_deref(),
        exif_result.created,
        exif_result.modified,
        exif_result.software.as_deref(),
        &exif_result.raw,
        strings_result.total_count as i32,
        &tool_outputs,
        threat_score,
        verdict.as_str(),
        &notes,
    ).await?;

    // ── j. Update ingest job_progress (cross-service) ───────────────────
    let _ = sqlx::query(
        "UPDATE job_progress SET sandbox_file_done = true, updated_at = now()
         WHERE email_id = $1"
    )
    .bind(job.email_id)
    .execute(ctx.ingest_pool.as_ref())
    .await;

    // ── k. Publish NATS completion event ────────────────────────────────
    let event = serde_json::json!({
        "attachment_id": job.attachment_id.to_string(),
        "email_id": job.email_id.to_string(),
        "tenant_id": job.tenant_id.to_string(),
        "threat_verdict": verdict.as_str(),
        "threat_score": threat_score,
        "yara_matches": yara_matches,
        "has_macros": has_macros,
    });

    if let Err(e) = ctx.nats.publish(
        "deepmail.events.sandbox.file.completed",
        event.to_string().into(),
    ).await {
        tracing::warn!("failed to publish completion event: {}", e);
    }

    // ── l. Increment YARA match counts ──────────────────────────────────
    if !yara_matches.is_empty() {
        let _ = db::yara_rules::increment_match_counts(&ctx.pool, &yara_matches).await;
    }

    Ok(report)
}
