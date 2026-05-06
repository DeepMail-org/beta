//! Main analysis pipeline.
//!
//! Orchestrates:
//! 1. Fetch parsed email data from deepmail_parser DB (cross-service, runtime SQL)
//! 2. Run SPF evaluation
//! 3. Extract DKIM result from Authentication-Results header
//! 4. Evaluate DMARC alignment
//! 5. Run 8 forensic header checks
//! 6. Compute overall risk score
//! 7. Persist results into deepmail_header DB (offline-prepared macros)
//! 8. Update job_progress in deepmail_ingest DB (cross-service, runtime SQL)

use std::net::IpAddr;
use std::sync::Arc;
use std::time::Instant;

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::checks::{
    ChecksOutput, HeaderInput, MtaFingerprint, ReceivedHop,
};
use crate::dkim;
use crate::dmarc;
use crate::dns::Resolver;
use crate::error::HeaderError;
use crate::spf;

/// Shared state for the pipeline.
pub struct PipelineState {
    pub header_pool: PgPool,
    pub ingest_pool: PgPool,
    pub parser_pool: PgPool,
    pub resolver: Arc<Resolver>,
}

/// Run the full header analysis pipeline for one email.
pub async fn analyze(
    state: &PipelineState,
    email_id: Uuid,
    tenant_id: Uuid,
) -> Result<AnalysisOutput, HeaderError> {
    let start = Instant::now();

    // ── Step 1: Fetch parsed email data from parser DB ────────────
    let parsed = fetch_parsed_email(&state.parser_pool, email_id).await?;

    // ── Step 2: Load MTA fingerprints ─────────────────────────────
    let fingerprints = load_fingerprints(&state.header_pool).await?;

    // ── Step 3: Extract sender IP (from first Received hop) ───────
    let sender_ip = extract_sender_ip(&parsed.received_hops);
    let from_domain = parsed.from_address.as_deref()
        .and_then(|a| extract_domain(a))
        .unwrap_or_default();

    // ── Step 4: SPF evaluation ────────────────────────────────────
    let (spf_result, _spf_raw) = if !from_domain.is_empty() {
        if let Some(ip) = sender_ip {
            spf::evaluate(&state.resolver, ip, &from_domain).await
        } else {
            (spf::SpfResult::None, None)
        }
    } else {
        (spf::SpfResult::None, None)
    };

    // ── Step 5: DKIM check from Authentication-Results header ─────
    let auth_results_header = parsed.headers.iter()
        .find(|(n, _)| n.eq_ignore_ascii_case("Authentication-Results"))
        .map(|(_, v)| v.clone());

    let dkim_sig_header = parsed.headers.iter()
        .find(|(n, _)| n.eq_ignore_ascii_case("DKIM-Signature"))
        .map(|(_, v)| v.clone());

    let dkim_check = if let Some(ref ar) = auth_results_header {
        dkim::check_authentication_results(ar)
    } else if let Some(ref sig) = dkim_sig_header {
        dkim::parse_dkim_signature(sig)
    } else {
        dkim::DkimCheckResult {
            result: dkim::DkimVerdict::None,
            domain: None,
            selector: None,
        }
    };

    // ── Step 6: DMARC evaluation ──────────────────────────────────
    let dkim_passed = dkim_check.result == dkim::DkimVerdict::Pass;
    let spf_passed = spf_result == spf::SpfResult::Pass;

    let (dmarc_result, dmarc_policy) = if !from_domain.is_empty() {
        dmarc::evaluate(
            &state.resolver,
            &from_domain,
            spf_passed,
            &from_domain,
            dkim_passed,
            dkim_check.domain.as_deref().unwrap_or(""),
        )
        .await
    } else {
        (dmarc::DmarcResult::None, None)
    };

    // ── Step 7: Run forensic checks ───────────────────────────────
    let header_input = HeaderInput {
        from_address: parsed.from_address.clone(),
        reply_to: parsed.reply_to.clone(),
        message_id: parsed.message_id.clone(),
        date_sent: parsed.date_sent,
        headers: parsed.headers.clone(),
        received_hops: parsed
            .received_hops
            .iter()
            .map(|h| ReceivedHop {
                hop_index: h.hop_index,
                from_host: h.from_host.clone(),
                by_host: h.by_host.clone(),
                received_at: h.received_at,
                raw_value: h.raw_value.clone(),
            })
            .collect(),
    };

    let checks = crate::checks::run_all_checks(&header_input, &fingerprints);

    // ── Step 8: Compute overall risk score ────────────────────────
    let risk_score = compute_risk_score(&spf_result, &dkim_check, &dmarc_result, &checks);

    // ── Step 9: Persist results ───────────────────────────────────
    persist_analysis(
        &state.header_pool,
        email_id,
        tenant_id,
        &spf_result,
        &from_domain,
        sender_ip,
        &dkim_check,
        &dmarc_result,
        &dmarc_policy,
        &checks,
        risk_score,
    )
    .await?;

    // ── Step 10: Update job_progress in ingest DB ─────────────────
    let duration_ms = start.elapsed().as_millis() as i32;
    update_job_progress(&state.ingest_pool, email_id, duration_ms).await?;

    Ok(AnalysisOutput {
        email_id,
        risk_score,
        spf_result: spf_result.as_str().to_string(),
        spf_domain: from_domain.clone(),
        dkim_result: dkim_check.result.as_str().to_string(),
        dkim_domain: dkim_check.domain.unwrap_or_default(),
        dmarc_result: dmarc_result.as_str().to_string(),
        dmarc_policy: dmarc_policy
            .as_ref()
            .map(|p| p.policy.clone())
            .unwrap_or_default(),
        checks,
    })
}

/// Output from one analysis run.
pub struct AnalysisOutput {
    pub email_id: Uuid,
    pub risk_score: f32,
    pub spf_result: String,
    pub spf_domain: String,
    pub dkim_result: String,
    pub dkim_domain: String,
    pub dmarc_result: String,
    pub dmarc_policy: String,
    pub checks: ChecksOutput,
}

// ── Parsed email data fetched from the parser DB ─────────────────

struct ParsedEmailData {
    from_address: Option<String>,
    reply_to: Option<String>,
    message_id: Option<String>,
    date_sent: Option<DateTime<Utc>>,
    headers: Vec<(String, String)>,
    received_hops: Vec<ParsedReceivedHop>,
}

struct ParsedReceivedHop {
    hop_index: i32,
    from_host: Option<String>,
    by_host: Option<String>,
    received_at: Option<DateTime<Utc>>,
    raw_value: String,
}

/// Fetch parsed email + headers + received hops from the parser DB.
/// Uses runtime sqlx::query (not macro) because this is cross-service.
async fn fetch_parsed_email(
    parser_pool: &PgPool,
    email_id: Uuid,
) -> Result<ParsedEmailData, HeaderError> {
    // 1. Fetch the parsed_emails row
    let row = sqlx::query(
        r#"SELECT id, from_address, reply_to, message_id, date_sent
           FROM parsed_emails
           WHERE email_id = $1"#,
    )
    .bind(email_id)
    .fetch_optional(parser_pool)
    .await?;

    let row = row.ok_or(HeaderError::EmailNotFound(email_id))?;

    use sqlx::Row;
    let parsed_email_id: Uuid = row.get("id");
    let from_address: Option<String> = row.get("from_address");
    let reply_to: Option<String> = row.get("reply_to");
    let message_id: Option<String> = row.get("message_id");
    let date_sent: Option<DateTime<Utc>> = row.get("date_sent");

    // 2. Fetch all headers
    let header_rows = sqlx::query(
        r#"SELECT header_name, header_value
           FROM email_headers
           WHERE parsed_email_id = $1
           ORDER BY ordinal"#,
    )
    .bind(parsed_email_id)
    .fetch_all(parser_pool)
    .await?;

    let headers: Vec<(String, String)> = header_rows
        .iter()
        .map(|r| {
            let name: String = r.get("header_name");
            let value: String = r.get("header_value");
            (name, value)
        })
        .collect();

    // 3. Fetch received hops
    let hop_rows = sqlx::query(
        r#"SELECT hop_index, from_host, by_host, received_at, raw_value
           FROM received_hops
           WHERE parsed_email_id = $1
           ORDER BY hop_index"#,
    )
    .bind(parsed_email_id)
    .fetch_all(parser_pool)
    .await?;

    let received_hops: Vec<ParsedReceivedHop> = hop_rows
        .iter()
        .map(|r| ParsedReceivedHop {
            hop_index: r.get("hop_index"),
            from_host: r.get("from_host"),
            by_host: r.get("by_host"),
            received_at: r.get("received_at"),
            raw_value: r.get("raw_value"),
        })
        .collect();

    Ok(ParsedEmailData {
        from_address,
        reply_to,
        message_id,
        date_sent,
        headers,
        received_hops,
    })
}

/// Load all MTA fingerprints from the header DB.
async fn load_fingerprints(
    header_pool: &PgPool,
) -> Result<Vec<MtaFingerprint>, HeaderError> {
    let rows = sqlx::query!(
        r#"SELECT pattern, match_type, kit_name, risk_level
           FROM mta_fingerprints"#
    )
    .fetch_all(header_pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| MtaFingerprint {
            pattern: r.pattern,
            match_type: r.match_type,
            kit_name: r.kit_name,
            risk_level: r.risk_level,
        })
        .collect())
}

/// Extract the sender IP address from the first Received hop.
/// Looks for IP-like patterns in from_host.
fn extract_sender_ip(hops: &[ParsedReceivedHop]) -> Option<IpAddr> {
    // The last hop (highest hop_index) is the originating MTA
    let last_hop = hops.last()?;
    let from_host = last_hop.from_host.as_deref()?;
    extract_ip_from_string(from_host)
}

/// Best-effort IP extraction from a string like "mx.example.com (1.2.3.4)"
/// or "[10.0.0.1]".
fn extract_ip_from_string(s: &str) -> Option<IpAddr> {
    // Try parsing the whole string
    if let Ok(ip) = s.parse::<IpAddr>() {
        return Some(ip);
    }

    // Look for bracketed IP: [10.0.0.1]
    if let (Some(start), Some(end)) = (s.find('['), s.find(']')) {
        if start < end {
            if let Ok(ip) = s[start + 1..end].parse::<IpAddr>() {
                return Some(ip);
            }
        }
    }

    // Look for parenthesized IP: (1.2.3.4)
    if let (Some(start), Some(end)) = (s.find('('), s.find(')')) {
        if start < end {
            if let Ok(ip) = s[start + 1..end].parse::<IpAddr>() {
                return Some(ip);
            }
        }
    }

    // Fallback: try each whitespace-separated token
    for token in s.split_whitespace() {
        let cleaned = token
            .trim_start_matches(['[', '('])
            .trim_end_matches([']', ')']);
        if let Ok(ip) = cleaned.parse::<IpAddr>() {
            return Some(ip);
        }
    }

    None
}

/// Compute overall risk score from all analysis results.
fn compute_risk_score(
    spf: &spf::SpfResult,
    dkim: &dkim::DkimCheckResult,
    dmarc: &dmarc::DmarcResult,
    checks: &ChecksOutput,
) -> f32 {
    let mut score: f32 = 0.0;

    // SPF scoring
    match spf {
        spf::SpfResult::Fail     => score += 0.25,
        spf::SpfResult::SoftFail => score += 0.15,
        spf::SpfResult::None     => score += 0.10,
        spf::SpfResult::TempError
        | spf::SpfResult::PermError => score += 0.10,
        _                         => {}
    }

    // DKIM scoring
    match dkim.result {
        dkim::DkimVerdict::Fail      => score += 0.20,
        dkim::DkimVerdict::None      => score += 0.10,
        dkim::DkimVerdict::TempError
        | dkim::DkimVerdict::PermError => score += 0.10,
        _                             => {}
    }

    // DMARC scoring
    match dmarc {
        dmarc::DmarcResult::Fail => score += 0.20,
        dmarc::DmarcResult::None => score += 0.10,
        _                        => {}
    }

    // Findings risk weights
    for finding in &checks.findings {
        if !finding.passed {
            score += finding.risk_weight;
        }
    }

    // Clamp to [0.0, 1.0]
    score.clamp(0.0, 1.0)
}

/// Persist the analysis results into the deepmail_header database.
async fn persist_analysis(
    pool: &PgPool,
    email_id: Uuid,
    tenant_id: Uuid,
    spf_result: &spf::SpfResult,
    from_domain: &str,
    sender_ip: Option<IpAddr>,
    dkim_check: &dkim::DkimCheckResult,
    dmarc_result: &dmarc::DmarcResult,
    dmarc_policy: &Option<dmarc::DmarcPolicy>,
    checks: &ChecksOutput,
    risk_score: f32,
) -> Result<Uuid, HeaderError> {
    let spf_str = spf_result.as_str();
    let dkim_str = dkim_check.result.as_str();
    let dmarc_str = dmarc_result.as_str();
    let dmarc_policy_str = dmarc_policy.as_ref().map(|p| p.policy.as_str());
    let dmarc_domain = if !from_domain.is_empty() {
        Some(from_domain)
    } else {
        None
    };

    // Insert header_analyses
    let analysis_id = sqlx::query_scalar!(
        r#"INSERT INTO header_analyses (
             email_id, tenant_id, overall_risk_score,
             spf_result, spf_domain, spf_sender_ip,
             dkim_result, dkim_domain, dkim_selector,
             dmarc_result, dmarc_policy, dmarc_domain,
             return_path_domain, return_path_mismatch,
             reply_to_domain, reply_to_mismatch,
             from_domain,
             message_id_valid, message_id_domain,
             message_id_domain_match, message_id_fake_pattern,
             x_mailer, x_mailer_risk, x_mailer_known_kit,
             sender_spoofing_likely
           ) VALUES (
             $1, $2, $3,
             $4, $5, $6,
             $7, $8, $9,
             $10, $11, $12,
             $13, $14,
             $15, $16,
             $17,
             $18, $19,
             $20, $21,
             $22, $23, $24,
             $25
           ) ON CONFLICT (email_id) DO UPDATE SET
             overall_risk_score    = EXCLUDED.overall_risk_score,
             spf_result            = EXCLUDED.spf_result,
             spf_domain            = EXCLUDED.spf_domain,
             spf_sender_ip         = EXCLUDED.spf_sender_ip,
             dkim_result           = EXCLUDED.dkim_result,
             dkim_domain           = EXCLUDED.dkim_domain,
             dkim_selector         = EXCLUDED.dkim_selector,
             dmarc_result          = EXCLUDED.dmarc_result,
             dmarc_policy          = EXCLUDED.dmarc_policy,
             dmarc_domain          = EXCLUDED.dmarc_domain,
             return_path_domain    = EXCLUDED.return_path_domain,
             return_path_mismatch  = EXCLUDED.return_path_mismatch,
             reply_to_domain       = EXCLUDED.reply_to_domain,
             reply_to_mismatch     = EXCLUDED.reply_to_mismatch,
             from_domain           = EXCLUDED.from_domain,
             message_id_valid      = EXCLUDED.message_id_valid,
             message_id_domain     = EXCLUDED.message_id_domain,
             message_id_domain_match = EXCLUDED.message_id_domain_match,
             message_id_fake_pattern = EXCLUDED.message_id_fake_pattern,
             x_mailer              = EXCLUDED.x_mailer,
             x_mailer_risk         = EXCLUDED.x_mailer_risk,
             x_mailer_known_kit    = EXCLUDED.x_mailer_known_kit,
             sender_spoofing_likely= EXCLUDED.sender_spoofing_likely,
             analyzed_at           = now()
           RETURNING id"#,
        email_id,
        tenant_id,
        risk_score,
        spf_str,
        from_domain,
        sender_ip as Option<IpAddr>,
        dkim_str,
        dkim_check.domain.as_deref(),
        dkim_check.selector.as_deref(),
        dmarc_str,
        dmarc_policy_str,
        dmarc_domain,
        checks.return_path_domain.as_deref(),
        checks.return_path_mismatch,
        checks.reply_to_domain.as_deref(),
        checks.reply_to_mismatch,
        checks.from_domain.as_deref(),
        checks.message_id_valid,
        checks.message_id_domain.as_deref(),
        checks.message_id_domain_match,
        checks.message_id_fake_pattern,
        checks.x_mailer.as_deref(),
        checks.x_mailer_risk.as_deref(),
        checks.x_mailer_known_kit.as_deref(),
        checks.sender_spoofing_likely,
    )
    .fetch_one(pool)
    .await?;

    // Delete old findings (upsert scenario)
    sqlx::query!(
        "DELETE FROM header_findings WHERE analysis_id = $1",
        analysis_id,
    )
    .execute(pool)
    .await?;

    // Insert findings
    for finding in &checks.findings {
        sqlx::query!(
            r#"INSERT INTO header_findings (
                 email_id, analysis_id, check_name, severity,
                 passed, evidence, raw_value, risk_weight
               ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
            email_id,
            analysis_id,
            finding.check_name,
            finding.severity,
            finding.passed,
            finding.evidence,
            finding.raw_value,
            finding.risk_weight,
        )
        .execute(pool)
        .await?;
    }

    // Insert time anomaly
    let ta = &checks.time_anomaly;
    sqlx::query!(
        r#"INSERT INTO time_anomalies (
             email_id, date_header_ts, earliest_received_ts,
             latest_received_ts, future_dated, date_predates_recv,
             max_hop_delay_sec, suspicious_delay,
             total_transit_sec, hop_count
           ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
           ON CONFLICT (email_id) DO UPDATE SET
             date_header_ts        = EXCLUDED.date_header_ts,
             earliest_received_ts  = EXCLUDED.earliest_received_ts,
             latest_received_ts    = EXCLUDED.latest_received_ts,
             future_dated          = EXCLUDED.future_dated,
             date_predates_recv    = EXCLUDED.date_predates_recv,
             max_hop_delay_sec     = EXCLUDED.max_hop_delay_sec,
             suspicious_delay      = EXCLUDED.suspicious_delay,
             total_transit_sec     = EXCLUDED.total_transit_sec,
             hop_count             = EXCLUDED.hop_count"#,
        email_id,
        ta.date_header_ts,
        ta.earliest_received_ts,
        ta.latest_received_ts,
        ta.future_dated,
        ta.date_predates_received,
        ta.max_hop_delay_sec,
        ta.suspicious_delay,
        ta.total_transit_sec,
        ta.hop_count,
    )
    .execute(pool)
    .await?;

    Ok(analysis_id)
}

/// Update job_progress in deepmail_ingest DB.
/// Uses runtime sqlx::query (not macro) — cross-service.
async fn update_job_progress(
    ingest_pool: &PgPool,
    email_id: Uuid,
    duration_ms: i32,
) -> Result<(), HeaderError> {
    sqlx::query(
        r#"UPDATE job_progress
           SET status = 'completed',
               completed_at = now(),
               duration_ms = $1
           WHERE email_id = $2
             AND stage = 'header'"#,
    )
    .bind(duration_ms)
    .bind(email_id)
    .execute(ingest_pool)
    .await?;
    Ok(())
}

/// Extract domain from an email address string.
fn extract_domain(addr: &str) -> Option<String> {
    let cleaned = if addr.contains('<') && addr.contains('>') {
        let start = addr.find('<').unwrap_or(0) + 1;
        let end = addr.find('>').unwrap_or(addr.len());
        &addr[start..end]
    } else {
        addr
    };
    let cleaned = cleaned.trim();
    let at_pos = cleaned.rfind('@')?;
    let domain = &cleaned[at_pos + 1..];
    if domain.is_empty() {
        None
    } else {
        Some(domain.to_lowercase())
    }
}
