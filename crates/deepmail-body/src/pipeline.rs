/// Full body analysis pipeline: fetch → analyze → persist → publish.

use std::sync::Arc;

use regex::Regex;
use sqlx::PgPool;
use uuid::Uuid;

use crate::db;
use crate::error::BodyError;
use crate::html;
use crate::keywords;
use crate::ml::MlClient;
use crate::urls::{self, ExtractedUrl};

/// Pipeline context shared across tasks.
pub struct PipelineCtx {
    pub pool: Arc<PgPool>,
    pub parser_pool: Arc<PgPool>,
    pub ingest_pool: Arc<PgPool>,
    pub ml_client: Option<Arc<MlClient>>,
    pub nats: async_nats::Client,
}

/// Pipeline result summary.
#[derive(Debug, Clone)]
pub struct PipelineResult {
    pub analysis_id: Uuid,
    pub email_id: Uuid,
    pub url_count: i32,
    pub qr_code_count: i32,
    pub final_phishing_score: f32,
    pub verdict: String,
    pub has_obfuscation: bool,
    pub has_tracking_pixels: bool,
}

/// Row from parser DB for email body data.
#[derive(Debug)]
struct ParserBodyRow {
    body_text: Option<String>,
    body_html: Option<String>,
    #[allow(dead_code)]
    subject: Option<String>,
    from_header: Option<String>,
}

/// Run the full body analysis pipeline for an email.
pub async fn run_pipeline(
    ctx: Arc<PipelineCtx>,
    email_id: Uuid,
    tenant_id: Uuid,
) -> Result<PipelineResult, BodyError> {
    // ── a. Idempotency check ───────────────────────────────────────────
    if let Some(existing) = db::analyses::get_by_email_id(&ctx.pool, email_id).await? {
        tracing::info!(%email_id, "body analysis already exists, returning cached");
        return Ok(PipelineResult {
            analysis_id: existing.id,
            email_id: existing.email_id,
            url_count: existing.url_count,
            qr_code_count: existing.qr_code_count,
            final_phishing_score: existing.final_phishing_score,
            verdict: existing.verdict,
            has_obfuscation: existing.has_obfuscation,
            has_tracking_pixels: existing.has_tracking_pixels,
        });
    }

    // ── b. Fetch email body from parser DB ─────────────────────────────
    let parser_row = fetch_email_body(&ctx.parser_pool, email_id).await?;

    // ── c. Extract sender domain ───────────────────────────────────────
    let sender_domain = extract_sender_domain(
        parser_row.from_header.as_deref().unwrap_or_default(),
    );

    let body_text = parser_row.body_text.as_deref().unwrap_or_default();
    let body_html = parser_row.body_html.as_deref().unwrap_or_default();

    let plain_text_length = body_text.len() as i32;
    let html_length = body_html.len() as i32;

    // ── d + e. Extract text, URLs, obfuscation, tracking ──────────────
    let mut all_urls: Vec<ExtractedUrl> = Vec::new();
    let mut combined_text = String::new();
    let mut obfuscation_report = html::ObfuscationReport {
        has_obfuscation: false,
        techniques: Vec::new(),
    };
    let mut tracking_hits = Vec::new();
    let mut qr_findings = Vec::new();

    if !body_html.is_empty() {
        // Extract plain text from HTML
        let html_text = html::extract_plain_text(body_html);
        combined_text.push_str(&html_text);

        // Extract URLs from HTML
        let html_urls = html::extract_urls_from_html(body_html, &sender_domain);
        all_urls.extend(html_urls);

        // Obfuscation detection
        obfuscation_report = html::detect_obfuscation(body_html);

        // Tracking pixel detection
        tracking_hits = html::detect_tracking_pixels(body_html);

        // QR code detection
        qr_findings = html::detect_qr_candidates(body_html);
    }

    if !body_text.is_empty() {
        if !combined_text.is_empty() {
            combined_text.push(' ');
        }
        combined_text.push_str(body_text);

        // Extract URLs from plain text (avoid duplicates)
        let text_urls = html::extract_urls_from_text(body_text, &sender_domain);
        for tu in text_urls {
            if !all_urls.iter().any(|u| u.normalized_url == tu.normalized_url) {
                all_urls.push(tu);
            }
        }
    }

    // ── f. Extract base64-encoded URLs ─────────────────────────────────
    let base64_urls = urls::extract_base64_urls(&combined_text);
    for b64_url in &base64_urls {
        let eu = urls::classify_url(b64_url, "base64_decoded", &sender_domain);
        if !all_urls.iter().any(|u| u.normalized_url == eu.normalized_url) {
            all_urls.push(eu);
        }
    }

    // ── h. Count URL categories ────────────────────────────────────────
    let url_count = all_urls.len() as i32;
    let external_url_count = all_urls.iter().filter(|u| u.is_external).count() as i32;
    let shortener_url_count = all_urls.iter().filter(|u| u.is_shortened).count() as i32;
    let base64_url_count = base64_urls.len() as i32;
    let qr_code_count = qr_findings.len() as i32;

    // ── i. Compute keyword score ───────────────────────────────────────
    let keyword_score = keywords::compute_keyword_score(&combined_text);

    // ── j. Detect urgency ──────────────────────────────────────────────
    let urgency_score = keywords::detect_urgency(&combined_text);

    // ── k. ML classification ───────────────────────────────────────────
    let ml_score = if let Some(ref ml_client) = ctx.ml_client {
        match ml_client.classify_phishing(&combined_text).await {
            Ok(score) => Some(score),
            Err(e) => {
                tracing::info!("ML service unavailable, using keyword score only: {}", e);
                None
            }
        }
    } else {
        None
    };

    // ── l. Final phishing score ────────────────────────────────────────
    let final_phishing_score = match ml_score {
        Some(ml) => 0.4 * keyword_score + 0.6 * ml,
        None => keyword_score,
    };

    // ── m. Determine verdict ───────────────────────────────────────────
    let verdict = if final_phishing_score >= 0.80 {
        "MALICIOUS"
    } else if final_phishing_score >= 0.60 {
        "PHISHING"
    } else if final_phishing_score >= 0.35 {
        "SUSPICIOUS"
    } else {
        "CLEAN"
    };

    // ── p. Persist in DB ───────────────────────────────────────────────
    let has_tracking = !tracking_hits.is_empty();

    let analysis_id = db::analyses::upsert_analysis(
        &ctx.pool,
        email_id,
        tenant_id,
        plain_text_length,
        html_length,
        url_count,
        external_url_count,
        shortener_url_count,
        qr_code_count,
        base64_url_count,
        keyword_score,
        ml_score,
        final_phishing_score,
        obfuscation_report.has_obfuscation,
        has_tracking,
        false, // has_c2_beacons — set by tracking hits from known C2 domains
        &obfuscation_report.techniques,
        urgency_score,
        verdict,
    )
    .await?;

    // Insert URLs
    db::urls::bulk_insert_urls(&ctx.pool, analysis_id, &all_urls).await?;

    // Insert QR findings
    for qr in &qr_findings {
        db::qr::insert_qr_finding(&ctx.pool, analysis_id, qr).await?;
    }

    // ── n. Publish NATS for suspicious URLs → sandbox ──────────────────
    for u in &all_urls {
        if u.is_shortened || u.is_suspicious {
            let payload = serde_json::json!({
                "email_id": email_id.to_string(),
                "tenant_id": tenant_id.to_string(),
                "url": u.normalized_url,
                "analysis_id": analysis_id.to_string(),
            });
            if let Err(e) = ctx
                .nats
                .publish(
                    "deepmail.jobs.sandbox.url".to_string(),
                    payload.to_string().into(),
                )
                .await
            {
                tracing::warn!("failed to publish sandbox URL job: {}", e);
            } else {
                // Mark sent
                let _ = db::urls::mark_sent_to_sandbox(
                    &ctx.pool,
                    analysis_id,
                    &u.normalized_url,
                )
                .await;
            }
        }
    }

    // ── o. Publish NATS for QR candidates → sandbox ────────────────────
    for qr in &qr_findings {
        let payload = serde_json::json!({
            "email_id": email_id.to_string(),
            "tenant_id": tenant_id.to_string(),
            "url": qr.image_src,
            "url_type": "qr_candidate",
            "analysis_id": analysis_id.to_string(),
        });
        if let Err(e) = ctx
            .nats
            .publish(
                "deepmail.jobs.sandbox.url".to_string(),
                payload.to_string().into(),
            )
            .await
        {
            tracing::warn!("failed to publish sandbox QR job: {}", e);
        }
    }

    // ── q. Update ingest job progress ──────────────────────────────────
    let _ = sqlx::query(
        "UPDATE ingest_jobs SET
           stages_completed = array_append(stages_completed, 'body'),
           updated_at = now()
         WHERE email_id = $1",
    )
    .bind(email_id)
    .execute(ctx.ingest_pool.as_ref())
    .await;

    // ── r. Publish completion event ────────────────────────────────────
    let event_payload = serde_json::json!({
        "email_id": email_id.to_string(),
        "tenant_id": tenant_id.to_string(),
        "verdict": verdict,
        "final_phishing_score": final_phishing_score,
        "url_count": url_count,
        "qr_code_count": qr_code_count,
    });
    if let Err(e) = ctx
        .nats
        .publish(
            "deepmail.events.body.completed".to_string(),
            event_payload.to_string().into(),
        )
        .await
    {
        tracing::warn!("failed to publish body.completed event: {}", e);
    }

    tracing::info!(
        %email_id, %analysis_id, %verdict,
        score = final_phishing_score, urls = url_count,
        "body analysis complete"
    );

    Ok(PipelineResult {
        analysis_id,
        email_id,
        url_count,
        qr_code_count,
        final_phishing_score,
        verdict: verdict.to_string(),
        has_obfuscation: obfuscation_report.has_obfuscation,
        has_tracking_pixels: has_tracking,
    })
}

/// Fetch email body from the parser database (cross-service).
async fn fetch_email_body(
    parser_pool: &PgPool,
    email_id: Uuid,
) -> Result<ParserBodyRow, BodyError> {
    let row = sqlx::query_as::<_, (Option<String>, Option<String>, Option<String>, Option<String>)>(
        r#"SELECT pe.body_text, pe.body_html, pe.subject,
                  COALESCE(CAST(eh_from.value AS TEXT), '') as from_header
           FROM parsed_emails pe
           LEFT JOIN email_headers eh_from ON (
             eh_from.parsed_email_id = pe.id AND eh_from.name = 'from'
           )
           WHERE pe.email_id = $1
           LIMIT 1"#,
    )
    .bind(email_id)
    .fetch_optional(parser_pool)
    .await
    .map_err(BodyError::Db)?
    .ok_or_else(|| BodyError::NotFound(format!("email {} not found in parser DB", email_id)))?;

    Ok(ParserBodyRow {
        body_text: row.0,
        body_html: row.1,
        subject: row.2,
        from_header: row.3,
    })
}

/// Extract sender domain from the From header value.
fn extract_sender_domain(from_header: &str) -> String {
    let re = Regex::new(r"@([\w.\-]+)").unwrap();
    re.captures(from_header)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_lowercase())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_sender_domain() {
        assert_eq!(extract_sender_domain("user@example.com"), "example.com");
        assert_eq!(
            extract_sender_domain("\"John\" <john@sub.domain.org>"),
            "sub.domain.org"
        );
        assert_eq!(extract_sender_domain("no-at-sign"), "");
    }
}
