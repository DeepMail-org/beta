use std::sync::Arc;
use std::time::Instant;

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::canon;
use crate::db::{analyses, key_cache, signatures};
use crate::dns::Resolver;
use crate::error::DkimError;
use crate::parse;
use crate::signals;
use crate::verify;

pub struct PipelineState {
    pub dkim_pool: PgPool,
    pub ingest_pool: PgPool,
    pub resolver: Arc<Resolver>,
}

pub struct AnalysisOutput {
    pub email_id: Uuid,
    pub replay_confidence: f32,
    pub verdict: String,
    pub signals: signals::SignalScores,
    pub signature_count: usize,
    pub duration_ms: i32,
}

pub async fn analyze(
    state: &PipelineState,
    email_id: Uuid,
    tenant_id: Uuid,
) -> Result<AnalysisOutput, DkimError> {
    let start = Instant::now();

    let raw_email = fetch_raw_email(&state.ingest_pool, email_id).await?;

    let dkim_header_values = parse::extract_dkim_headers_from_raw(&raw_email);

    if dkim_header_values.is_empty() {
        let empty_signals = signals::SignalScores {
            body_hash_mismatch: 0.0,
            timestamp_drift: 0.0,
            received_before_signed: 0.0,
            key_rotated_since: 0.0,
            excessive_replay_gap: 0.0,
            header_order_tampered: 0.0,
        };
        let verdict = "NONE";
        analyses::upsert_analysis(
            &state.dkim_pool,
            email_id,
            tenant_id,
            &dkim_header_values,
            verdict,
            0.0,
            &empty_signals,
        )
        .await?;

        let duration_ms = start.elapsed().as_millis() as i32;
        update_job_progress(&state.ingest_pool, email_id, duration_ms).await?;

        return Ok(AnalysisOutput {
            email_id,
            replay_confidence: 0.0,
            verdict: verdict.to_string(),
            signals: empty_signals,
            signature_count: 0,
            duration_ms,
        });
    }

    let email_str = String::from_utf8_lossy(&raw_email);
    let body_start = email_str
        .find("\r\n\r\n")
        .map(|p| p + 4)
        .or_else(|| email_str.find("\n\n").map(|p| p + 2))
        .unwrap_or(raw_email.len());
    let body_bytes = &raw_email[body_start..];

    let date_header = extract_date_header(&raw_email);
    let received_time = extract_first_received_time(&raw_email);
    let actual_header_names = extract_header_names(&raw_email);

    let mut aggregate_signals = signals::SignalScores {
        body_hash_mismatch: 0.0,
        timestamp_drift: 0.0,
        received_before_signed: 0.0,
        key_rotated_since: 0.0,
        excessive_replay_gap: 0.0,
        header_order_tampered: 0.0,
    };

    let mut per_sig_results: Vec<SigResult> = Vec::new();

    for raw_sig_header in &dkim_header_values {
        let sig = match parse::parse_dkim_signature_header(raw_sig_header) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!(error = %e, "skipping unparseable DKIM-Signature");
                continue;
            }
        };

        let (header_canon, body_canon) = canon::parse_canonicalization(&sig.canonicalization);

        let canonical_body = canon::canonicalize_body(body_bytes, body_canon);
        let computed_bh = verify::compute_body_hash(&canonical_body);
        let bh_match = computed_bh == sig.body_hash;

        let (key_txt, key_found, _key_ttl) =
            fetch_or_cache_key(&state.dkim_pool, &state.resolver, &sig.selector, &sig.domain)
                .await;

        let mut sig_valid = false;
        if let Some(ref key_record_txt) = key_txt {
            if let Ok(dns_key) = verify::parse_dns_key_record(key_record_txt) {
                let canonical_headers =
                    canon::canonicalize_headers(&raw_email, &sig, header_canon);
                let vr = verify::verify_signature(
                    &sig.algorithm,
                    &canonical_headers,
                    &sig.signature_data,
                    &dns_key.public_key_b64,
                    Some(&dns_key.algorithm),
                );
                sig_valid = vr.valid;
                if let Some(ref err) = vr.error {
                    tracing::debug!(
                        selector = %sig.selector,
                        domain = %sig.domain,
                        error = %err,
                        "signature verification detail"
                    );
                }
            }
        }

        let sig_signals = signals::evaluate_signals(&signals::SignalInput {
            body_hash_matches: bh_match,
            signature_timestamp: sig.timestamp,
            date_header,
            received_time,
            key_found_in_dns: key_found,
            signed_headers: sig.signed_headers.clone(),
            actual_headers_present: actual_header_names.clone(),
        });

        aggregate_signals.body_hash_mismatch = aggregate_signals
            .body_hash_mismatch
            .max(sig_signals.body_hash_mismatch);
        aggregate_signals.timestamp_drift = aggregate_signals
            .timestamp_drift
            .max(sig_signals.timestamp_drift);
        aggregate_signals.received_before_signed = aggregate_signals
            .received_before_signed
            .max(sig_signals.received_before_signed);
        aggregate_signals.key_rotated_since = aggregate_signals
            .key_rotated_since
            .max(sig_signals.key_rotated_since);
        aggregate_signals.excessive_replay_gap = aggregate_signals
            .excessive_replay_gap
            .max(sig_signals.excessive_replay_gap);
        aggregate_signals.header_order_tampered = aggregate_signals
            .header_order_tampered
            .max(sig_signals.header_order_tampered);

        per_sig_results.push(SigResult {
            selector: sig.selector.clone(),
            domain: sig.domain.clone(),
            algorithm: sig.algorithm.clone(),
            canonicalization: sig.canonicalization.clone(),
            signed_headers: sig.signed_headers.clone(),
            body_hash_claimed: sig.body_hash.clone(),
            body_hash_computed: computed_bh,
            body_hash_match: bh_match,
            signature_timestamp: sig.timestamp,
            signature_valid: sig_valid,
            key_found_in_dns: key_found,
        });
    }

    let confidence = signals::compute_confidence(&aggregate_signals);
    let verdict = signals::confidence_to_verdict(confidence);

    let analysis_id = analyses::upsert_analysis(
        &state.dkim_pool,
        email_id,
        tenant_id,
        &dkim_header_values,
        verdict,
        confidence,
        &aggregate_signals,
    )
    .await?;

    for sr in &per_sig_results {
        signatures::insert_signature(
            &state.dkim_pool,
            analysis_id,
            &sr.selector,
            &sr.domain,
            &sr.algorithm,
            &sr.canonicalization,
            &sr.signed_headers,
            &sr.body_hash_claimed,
            &sr.body_hash_computed,
            sr.body_hash_match,
            sr.signature_timestamp,
            sr.signature_valid,
            sr.key_found_in_dns,
        )
        .await?;
    }

    let duration_ms = start.elapsed().as_millis() as i32;
    update_job_progress(&state.ingest_pool, email_id, duration_ms).await?;

    Ok(AnalysisOutput {
        email_id,
        replay_confidence: confidence,
        verdict: verdict.to_string(),
        signals: aggregate_signals,
        signature_count: per_sig_results.len(),
        duration_ms,
    })
}

struct SigResult {
    selector: String,
    domain: String,
    algorithm: String,
    canonicalization: String,
    signed_headers: Vec<String>,
    body_hash_claimed: String,
    body_hash_computed: String,
    body_hash_match: bool,
    signature_timestamp: Option<DateTime<Utc>>,
    signature_valid: bool,
    key_found_in_dns: bool,
}

async fn fetch_raw_email(ingest_pool: &PgPool, email_id: Uuid) -> Result<Vec<u8>, DkimError> {
    let row = sqlx::query(
        r#"SELECT raw_bytes FROM emails WHERE id = $1"#,
    )
    .bind(email_id)
    .fetch_optional(ingest_pool)
    .await?;

    let row = row.ok_or(DkimError::EmailNotFound(email_id))?;
    use sqlx::Row;
    let raw_bytes: Vec<u8> = row.get("raw_bytes");
    Ok(raw_bytes)
}

async fn fetch_or_cache_key(
    dkim_pool: &PgPool,
    resolver: &Resolver,
    selector: &str,
    domain: &str,
) -> (Option<String>, bool, i32) {
    if let Ok(Some(cached)) = key_cache::get_cached_key(dkim_pool, selector, domain).await {
        if !cached.is_expired() {
            let found = cached.public_key_pem.is_some();
            return (
                cached.public_key_pem,
                found,
                cached.ttl_seconds,
            );
        }
    }

    let dns_result = resolver.lookup_dkim_key(selector, domain).await;
    let ttl = dns_result.ttl_seconds.unwrap_or(300);

    let _ = key_cache::upsert_key(
        dkim_pool,
        selector,
        domain,
        dns_result.raw_txt.as_deref(),
        None,
        ttl,
    )
    .await;

    (dns_result.raw_txt, dns_result.found, ttl)
}

fn extract_date_header(raw_email: &[u8]) -> Option<DateTime<Utc>> {
    let parsed = mail_parser::MessageParser::default().parse(raw_email)?;
    parsed.date().and_then(|d| {
        chrono::DateTime::from_timestamp(d.to_timestamp(), 0)
    })
}

fn extract_first_received_time(raw_email: &[u8]) -> Option<DateTime<Utc>> {
    let parsed = mail_parser::MessageParser::default().parse(raw_email)?;
    for header in parsed.headers() {
        if header.name() == "Received" {
            if let mail_parser::HeaderValue::Received(recv) = header.value() {
                if let Some(date) = recv.date() {
                    return chrono::DateTime::from_timestamp(date.to_timestamp(), 0);
                }
            }
        }
    }
    None
}

fn extract_header_names(raw_email: &[u8]) -> Vec<String> {
    let email_str = String::from_utf8_lossy(raw_email);
    let header_section = email_str
        .split("\r\n\r\n")
        .next()
        .or_else(|| email_str.split("\n\n").next())
        .unwrap_or(&email_str);

    let mut names = Vec::new();
    for line in header_section.lines() {
        if !line.starts_with(' ') && !line.starts_with('\t') {
            if let Some(colon_pos) = line.find(':') {
                names.push(line[..colon_pos].to_string());
            }
        }
    }
    names
}

async fn update_job_progress(
    ingest_pool: &PgPool,
    email_id: Uuid,
    duration_ms: i32,
) -> Result<(), DkimError> {
    sqlx::query(
        r#"UPDATE job_progress
           SET status = 'completed',
               completed_at = now(),
               duration_ms = $1
           WHERE email_id = $2
             AND stage = 'dkim'"#,
    )
    .bind(duration_ms)
    .bind(email_id)
    .execute(ingest_pool)
    .await?;
    Ok(())
}
