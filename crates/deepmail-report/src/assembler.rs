use crate::error::ReportError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use std::sync::Arc;
use uuid::Uuid;

pub struct ServicePools {
    pub parser: Arc<PgPool>,
    pub header: Arc<PgPool>,
    pub dkim: Arc<PgPool>,
    pub geo: Arc<PgPool>,
    pub ip: Arc<PgPool>,
    pub intel: Arc<PgPool>,
    pub ioc: Arc<PgPool>,
    pub homograph: Arc<PgPool>,
    pub body: Arc<PgPool>,
    pub sandbox_url: Arc<PgPool>,
    pub sandbox_file: Arc<PgPool>,
    pub sandbox_dynamic: Arc<PgPool>,
    pub scoring: Arc<PgPool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportData {
    pub report_id: Uuid,
    pub email_id: Uuid,
    pub tenant_id: Uuid,
    pub generated_at: DateTime<Utc>,
    pub final_score: f32,
    pub final_verdict: String,
    pub signals_available: i32,
    pub email_metadata: EmailMetadata,
    pub header_analysis: Option<HeaderAnalysisData>,
    pub dkim_analysis: Option<DkimAnalysisData>,
    pub geo_analysis: Option<GeoAnalysisData>,
    pub ip_analysis: Option<IpAnalysisData>,
    pub intel_enrichment: Option<IntelEnrichmentData>,
    pub ioc_analysis: Option<IocAnalysisData>,
    pub homograph_analysis: Option<HomographAnalysisData>,
    pub body_analysis: Option<BodyAnalysisData>,
    pub sandbox_url: Option<SandboxUrlData>,
    pub sandbox_file: Option<SandboxFileData>,
    pub sandbox_dynamic: Option<SandboxDynamicData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailMetadata {
    pub subject: Option<String>,
    pub from: Option<String>,
    pub date_sent: Option<DateTime<Utc>>,
    pub message_id: Option<String>,
    pub reply_to: Option<String>,
    pub return_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderAnalysisData {
    pub spf_result: Option<String>,
    pub dkim_result: Option<String>,
    pub dmarc_result: Option<String>,
    pub return_path_mismatch: bool,
    pub reply_to_mismatch: bool,
    pub findings: Vec<String>,
    pub hop_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DkimSignatureData {
    pub selector: String,
    pub domain: String,
    pub body_hash_match: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DkimAnalysisData {
    pub replay_confidence: f32,
    pub verdict: String,
    pub signatures: Vec<DkimSignatureData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoHopData {
    pub ip: String,
    pub country: Option<String>,
    pub is_tor: bool,
    pub is_vpn: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoAnalysisData {
    pub origin_ip: Option<String>,
    pub origin_country: Option<String>,
    pub overall_risk: String,
    pub hops: Vec<GeoHopData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpAnalysisData {
    pub max_threat_score: f32,
    pub max_verdict: String,
    pub analyzed_ips: Vec<String>,
    pub feed_hits: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelEnrichmentData {
    pub iocs_analyzed: i32,
    pub max_vt_score: f32,
    pub malicious_iocs: i32,
    pub provider_hits: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopIoc {
    #[serde(rename = "type")]
    pub ioc_type: String,
    pub value: String,
    pub threat_level: String,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IocAnalysisData {
    pub total_iocs: i64,
    pub malicious_count: i64,
    pub suspicious_count: i64,
    pub campaign_id: Option<String>,
    pub campaign_name: Option<String>,
    pub campaign_status: Option<String>,
    pub top_iocs: Vec<TopIoc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspiciousDomain {
    pub domain: String,
    pub best_match: String,
    pub score: f32,
    pub risk: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomographAnalysisData {
    pub domains_checked: i32,
    pub high_risk_count: i32,
    pub overall_risk: String,
    pub suspicious_domains: Vec<SuspiciousDomain>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyAnalysisData {
    pub verdict: String,
    pub final_phishing_score: f32,
    pub url_count: i32,
    pub shortener_url_count: i32,
    pub qr_code_count: i32,
    pub has_obfuscation: bool,
    pub has_tracking_pixels: bool,
    pub suspicious_urls: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlSandboxReport {
    pub url: String,
    pub final_url: Option<String>,
    pub threat_class: String,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxUrlData {
    pub jobs_run: i64,
    pub malicious_count: i64,
    pub reports: Vec<UrlSandboxReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSandboxReport {
    pub filename: String,
    pub verdict: String,
    pub has_macros: bool,
    pub yara_matches: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxFileData {
    pub attachments_analyzed: i64,
    pub malicious_count: i64,
    pub reports: Vec<FileSandboxReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicSandboxReport {
    pub filename: String,
    pub verdict: String,
    pub malscore: f32,
    pub network_hosts: Vec<String>,
    pub persistence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxDynamicData {
    pub attachments_analyzed: i64,
    pub malicious_count: i64,
    pub reports: Vec<DynamicSandboxReport>,
}

pub async fn assemble_report(
    pools: &Arc<ServicePools>,
    email_id: Uuid,
    tenant_id: Uuid,
) -> Result<ReportData, ReportError> {
    let (
        scoring,
        metadata,
        header,
        dkim,
        geo,
        ip,
        intel,
        ioc,
        homograph,
        body,
        s_url,
        s_file,
        s_dynamic,
    ) = tokio::join!(
        fetch_scoring(&pools.scoring, email_id),
        fetch_email_metadata(&pools.parser, email_id),
        fetch_header_analysis(&pools.header, email_id),
        fetch_dkim_analysis(&pools.dkim, email_id),
        fetch_geo_analysis(&pools.geo, email_id),
        fetch_ip_analysis(&pools.ip, email_id),
        fetch_intel_enrichment(&pools.intel, email_id),
        fetch_ioc_analysis(&pools.ioc, email_id),
        fetch_homograph_analysis(&pools.homograph, email_id),
        fetch_body_analysis(&pools.body, email_id),
        fetch_sandbox_url(&pools.sandbox_url, email_id),
        fetch_sandbox_file(&pools.sandbox_file, email_id),
        fetch_sandbox_dynamic(&pools.sandbox_dynamic, email_id),
    );

    let (final_score, final_verdict, signals_available) = scoring;
    let email_metadata = metadata.unwrap_or_else(|| EmailMetadata {
        subject: None,
        from: None,
        date_sent: None,
        message_id: None,
        reply_to: None,
        return_path: None,
    });

    Ok(ReportData {
        report_id: Uuid::new_v4(),
        email_id,
        tenant_id,
        generated_at: Utc::now(),
        final_score,
        final_verdict,
        signals_available,
        email_metadata,
        header_analysis: header,
        dkim_analysis: dkim,
        geo_analysis: geo,
        ip_analysis: ip,
        intel_enrichment: intel,
        ioc_analysis: ioc,
        homograph_analysis: homograph,
        body_analysis: body,
        sandbox_url: s_url,
        sandbox_file: s_file,
        sandbox_dynamic: s_dynamic,
    })
}

async fn fetch_scoring(pool: &PgPool, email_id: Uuid) -> (f32, String, i32) {
    let row: Option<(f32, String, i32)> = sqlx::query_as(
        "SELECT final_score, final_verdict, signals_available \
         FROM threat_scores WHERE email_id = $1 LIMIT 1",
    )
    .bind(email_id)
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    row.unwrap_or((0.0, "CLEAN".to_string(), 0))
}

async fn fetch_email_metadata(pool: &PgPool, email_id: Uuid) -> Option<EmailMetadata> {
    let row: Option<(Option<String>, Option<String>, Option<DateTime<Utc>>, Option<String>, Option<String>)> =
        sqlx::query_as(
            "SELECT subject, from_address, date_sent, message_id, reply_to \
             FROM parsed_emails WHERE email_id = $1 LIMIT 1",
        )
        .bind(email_id)
        .fetch_optional(pool)
        .await
        .ok()?;

    let (subject, from, date_sent, message_id, reply_to) = row?;

    let parsed_email_id: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM parsed_emails WHERE email_id = $1 LIMIT 1",
    )
    .bind(email_id)
    .fetch_optional(pool)
    .await
    .ok()?;

    let return_path = if let Some((pe_id,)) = parsed_email_id {
        let rp: Option<(String,)> = sqlx::query_as(
            "SELECT header_value FROM email_headers \
             WHERE parsed_email_id = $1 AND lower(header_name) = 'return-path' \
             LIMIT 1",
        )
        .bind(pe_id)
        .fetch_optional(pool)
        .await
        .ok()?;
        rp.map(|(v,)| v)
    } else {
        None
    };

    Some(EmailMetadata {
        subject,
        from,
        date_sent,
        message_id,
        reply_to,
        return_path,
    })
}

async fn fetch_header_analysis(pool: &PgPool, email_id: Uuid) -> Option<HeaderAnalysisData> {
    let row: Option<(Option<String>, Option<String>, Option<String>, bool, bool, Option<i32>)> =
        sqlx::query_as(
            "SELECT spf_result, dkim_result, dmarc_result, \
                    return_path_mismatch, reply_to_mismatch, hop_count \
             FROM header_analyses WHERE email_id = $1 LIMIT 1",
        )
        .bind(email_id)
        .fetch_optional(pool)
        .await
        .ok()?;

    let (spf_result, dkim_result, dmarc_result, return_path_mismatch, reply_to_mismatch, hop_count) =
        row?;

    let findings_rows: Vec<(String,)> = sqlx::query_as(
        "SELECT evidence FROM header_findings \
         WHERE analysis_id = (SELECT id FROM header_analyses WHERE email_id = $1 LIMIT 1) \
         LIMIT 20",
    )
    .bind(email_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    Some(HeaderAnalysisData {
        spf_result,
        dkim_result,
        dmarc_result,
        return_path_mismatch,
        reply_to_mismatch,
        findings: findings_rows.into_iter().map(|(f,)| f).collect(),
        hop_count: hop_count.unwrap_or(0),
    })
}

async fn fetch_dkim_analysis(pool: &PgPool, email_id: Uuid) -> Option<DkimAnalysisData> {
    let row: Option<(f32, String)> = sqlx::query_as(
        "SELECT replay_confidence, overall_verdict \
         FROM dkim_analyses WHERE email_id = $1 LIMIT 1",
    )
    .bind(email_id)
    .fetch_optional(pool)
    .await
    .ok()?;

    let (replay_confidence, verdict) = row?;

    let sig_rows: Vec<(String, String, bool)> = sqlx::query_as(
        "SELECT selector, domain, body_hash_match \
         FROM dkim_signatures \
         WHERE analysis_id = (SELECT id FROM dkim_analyses WHERE email_id = $1 LIMIT 1)",
    )
    .bind(email_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    Some(DkimAnalysisData {
        replay_confidence,
        verdict,
        signatures: sig_rows
            .into_iter()
            .map(|(selector, domain, body_hash_match)| DkimSignatureData {
                selector,
                domain,
                body_hash_match,
            })
            .collect(),
    })
}

async fn fetch_geo_analysis(pool: &PgPool, email_id: Uuid) -> Option<GeoAnalysisData> {
    let row: Option<(Option<String>, Option<String>, String)> = sqlx::query_as(
        "SELECT CAST(origin_ip AS TEXT), origin_country, overall_risk \
         FROM email_geo_analyses WHERE email_id = $1 LIMIT 1",
    )
    .bind(email_id)
    .fetch_optional(pool)
    .await
    .ok()?;

    let (origin_ip, origin_country, overall_risk) = row?;

    let hop_rows: Vec<(String, Option<String>, bool, bool)> = sqlx::query_as(
        "SELECT CAST(ip_address AS TEXT), country_code, is_tor, is_vpn \
         FROM hop_geo_details \
         WHERE analysis_id = (SELECT id FROM email_geo_analyses WHERE email_id = $1 LIMIT 1) \
         LIMIT 20",
    )
    .bind(email_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    Some(GeoAnalysisData {
        origin_ip,
        origin_country,
        overall_risk,
        hops: hop_rows
            .into_iter()
            .map(|(ip, country, is_tor, is_vpn)| GeoHopData {
                ip,
                country,
                is_tor,
                is_vpn,
            })
            .collect(),
    })
}

async fn fetch_ip_analysis(pool: &PgPool, email_id: Uuid) -> Option<IpAnalysisData> {
    let row: Option<(f32, String, serde_json::Value)> = sqlx::query_as(
        "SELECT max_threat_score, max_verdict, summary_json \
         FROM email_ip_analyses WHERE email_id = $1 LIMIT 1",
    )
    .bind(email_id)
    .fetch_optional(pool)
    .await
    .ok()?;

    let (max_threat_score, max_verdict, summary_json) = row?;

    let ip_rows: Vec<(String,)> = sqlx::query_as(
        "SELECT CAST(unnest(analyzed_ips) AS TEXT) \
         FROM email_ip_analyses WHERE email_id = $1",
    )
    .bind(email_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let mut feed_hits = Vec::new();
    if let Some(ips) = summary_json.get("ips").and_then(|v| v.as_array()) {
        for ip_obj in ips {
            if let Some(feeds) = ip_obj.get("feeds_hit").and_then(|v| v.as_array()) {
                for feed in feeds {
                    if let Some(name) = feed.as_str() {
                        let name = name.to_string();
                        if !feed_hits.contains(&name) {
                            feed_hits.push(name);
                        }
                    }
                }
            }
        }
    }

    Some(IpAnalysisData {
        max_threat_score,
        max_verdict,
        analyzed_ips: ip_rows.into_iter().map(|(ip,)| ip).collect(),
        feed_hits,
    })
}

async fn fetch_intel_enrichment(pool: &PgPool, email_id: Uuid) -> Option<IntelEnrichmentData> {
    let row: Option<(i32, f32, i32, Vec<String>)> = sqlx::query_as(
        "SELECT iocs_analyzed, max_vt_score, malicious_iocs, provider_hits \
         FROM email_intel_results WHERE email_id = $1 LIMIT 1",
    )
    .bind(email_id)
    .fetch_optional(pool)
    .await
    .ok()?;

    let (iocs_analyzed, max_vt_score, malicious_iocs, provider_hits) = row?;

    Some(IntelEnrichmentData {
        iocs_analyzed,
        max_vt_score,
        malicious_iocs,
        provider_hits,
    })
}

async fn fetch_ioc_analysis(pool: &PgPool, email_id: Uuid) -> Option<IocAnalysisData> {
    let counts: Option<(i64, i64, i64)> = sqlx::query_as(
        "SELECT COUNT(*), \
                COUNT(*) FILTER (WHERE n.threat_level = 'MALICIOUS'), \
                COUNT(*) FILTER (WHERE n.threat_level = 'SUSPICIOUS') \
         FROM email_ioc_occurrences o \
         JOIN ioc_nodes n ON n.id = o.ioc_node_id \
         WHERE o.email_id = $1",
    )
    .bind(email_id)
    .fetch_optional(pool)
    .await
    .ok()?;

    let (total_iocs, malicious_count, suspicious_count) =
        counts.unwrap_or((0, 0, 0));

    let campaign: Option<(Uuid, String, String)> = sqlx::query_as(
        "SELECT c.id, c.campaign_name, c.status \
         FROM campaign_clusters c \
         JOIN campaign_members m ON m.campaign_id = c.id \
         WHERE m.email_id = $1 LIMIT 1",
    )
    .bind(email_id)
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    let (campaign_id, campaign_name, campaign_status) = match campaign {
        Some((id, name, status)) => (Some(id.to_string()), Some(name), Some(status)),
        None => (None, None, None),
    };

    let top_rows: Vec<(String, String, String, f32)> = sqlx::query_as(
        "SELECT n.ioc_type, n.ioc_value, n.threat_level, n.intel_score \
         FROM ioc_nodes n \
         JOIN email_ioc_occurrences o ON o.ioc_node_id = n.id \
         WHERE o.email_id = $1 \
         ORDER BY n.intel_score DESC LIMIT 10",
    )
    .bind(email_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    Some(IocAnalysisData {
        total_iocs,
        malicious_count,
        suspicious_count,
        campaign_id,
        campaign_name,
        campaign_status,
        top_iocs: top_rows
            .into_iter()
            .map(|(ioc_type, value, threat_level, score)| TopIoc {
                ioc_type,
                value,
                threat_level,
                score,
            })
            .collect(),
    })
}

async fn fetch_homograph_analysis(
    pool: &PgPool,
    email_id: Uuid,
) -> Option<HomographAnalysisData> {
    let row: Option<(i32, i32, String)> = sqlx::query_as(
        "SELECT domains_checked, high_risk_count, overall_risk \
         FROM homograph_analyses WHERE email_id = $1 LIMIT 1",
    )
    .bind(email_id)
    .fetch_optional(pool)
    .await
    .ok()?;

    let (domains_checked, high_risk_count, overall_risk) = row?;

    let dom_rows: Vec<(String, String, f32, String)> = sqlx::query_as(
        "SELECT domain, best_brand_match, final_score, risk_level \
         FROM domain_scores \
         WHERE analysis_id = (SELECT id FROM homograph_analyses WHERE email_id = $1 LIMIT 1) \
         AND risk_level != 'NONE' \
         ORDER BY final_score DESC LIMIT 10",
    )
    .bind(email_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    Some(HomographAnalysisData {
        domains_checked,
        high_risk_count,
        overall_risk,
        suspicious_domains: dom_rows
            .into_iter()
            .map(|(domain, best_match, score, risk)| SuspiciousDomain {
                domain,
                best_match,
                score,
                risk,
            })
            .collect(),
    })
}

async fn fetch_body_analysis(pool: &PgPool, email_id: Uuid) -> Option<BodyAnalysisData> {
    let row: Option<(String, f32, i32, i32, i32, bool, bool)> = sqlx::query_as(
        "SELECT verdict, final_phishing_score, url_count, shortener_url_count, \
                qr_code_count, has_obfuscation, has_tracking_pixels \
         FROM body_analyses WHERE email_id = $1 LIMIT 1",
    )
    .bind(email_id)
    .fetch_optional(pool)
    .await
    .ok()?;

    let (verdict, final_phishing_score, url_count, shortener_url_count, qr_code_count, has_obfuscation, has_tracking_pixels) =
        row?;

    let url_rows: Vec<(String,)> = sqlx::query_as(
        "SELECT normalized_url FROM extracted_urls \
         WHERE analysis_id = (SELECT id FROM body_analyses WHERE email_id = $1 LIMIT 1) \
         AND is_suspicious = true \
         ORDER BY created_at DESC LIMIT 20",
    )
    .bind(email_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    Some(BodyAnalysisData {
        verdict,
        final_phishing_score,
        url_count,
        shortener_url_count,
        qr_code_count,
        has_obfuscation,
        has_tracking_pixels,
        suspicious_urls: url_rows.into_iter().map(|(u,)| u).collect(),
    })
}

async fn fetch_sandbox_url(pool: &PgPool, email_id: Uuid) -> Option<SandboxUrlData> {
    let counts: Option<(i64, i64)> = sqlx::query_as(
        "SELECT COUNT(*), \
                COUNT(*) FILTER (WHERE threat_class != 'benign') \
         FROM url_sandbox_reports WHERE email_id = $1",
    )
    .bind(email_id)
    .fetch_optional(pool)
    .await
    .ok()?;

    let (jobs_run, malicious_count) = counts.unwrap_or((0, 0));
    if jobs_run == 0 {
        return None;
    }

    let rows: Vec<(String, Option<String>, String, f32)> = sqlx::query_as(
        "SELECT original_url, final_url, threat_class, threat_score \
         FROM url_sandbox_reports WHERE email_id = $1 \
         ORDER BY threat_score DESC LIMIT 10",
    )
    .bind(email_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    Some(SandboxUrlData {
        jobs_run,
        malicious_count,
        reports: rows
            .into_iter()
            .map(|(url, final_url, threat_class, score)| UrlSandboxReport {
                url,
                final_url,
                threat_class,
                score,
            })
            .collect(),
    })
}

async fn fetch_sandbox_file(pool: &PgPool, email_id: Uuid) -> Option<SandboxFileData> {
    let counts: Option<(i64, i64)> = sqlx::query_as(
        "SELECT COUNT(*), \
                COUNT(*) FILTER (WHERE threat_verdict = 'MALICIOUS') \
         FROM file_static_reports WHERE email_id = $1",
    )
    .bind(email_id)
    .fetch_optional(pool)
    .await
    .ok()?;

    let (total, malicious_count) = counts.unwrap_or((0, 0));
    if total == 0 {
        return None;
    }

    let rows: Vec<(String, String, bool, Vec<String>)> = sqlx::query_as(
        "SELECT filename, threat_verdict, has_macros, yara_matches \
         FROM file_static_reports WHERE email_id = $1 \
         ORDER BY threat_score DESC LIMIT 10",
    )
    .bind(email_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    Some(SandboxFileData {
        attachments_analyzed: total,
        malicious_count,
        reports: rows
            .into_iter()
            .map(|(filename, verdict, has_macros, yara_matches)| FileSandboxReport {
                filename,
                verdict,
                has_macros,
                yara_matches,
            })
            .collect(),
    })
}

async fn fetch_sandbox_dynamic(pool: &PgPool, email_id: Uuid) -> Option<SandboxDynamicData> {
    let counts: Option<(i64, i64)> = sqlx::query_as(
        "SELECT COUNT(*), \
                COUNT(*) FILTER (WHERE r.dynamic_verdict = 'MALWARE') \
         FROM dynamic_analysis_reports r \
         JOIN dynamic_analysis_jobs j ON j.id = r.job_id \
         WHERE j.email_id = $1",
    )
    .bind(email_id)
    .fetch_optional(pool)
    .await
    .ok()?;

    let (total, malicious_count) = counts.unwrap_or((0, 0));
    if total == 0 {
        return None;
    }

    let rows: Vec<(String, f32, Vec<String>, Vec<String>, String)> = sqlx::query_as(
        "SELECT j.filename, r.malscore, r.network_hosts, r.persistence_indicators, \
                r.dynamic_verdict \
         FROM dynamic_analysis_reports r \
         JOIN dynamic_analysis_jobs j ON j.id = r.job_id \
         WHERE j.email_id = $1 \
         ORDER BY r.dynamic_score DESC LIMIT 5",
    )
    .bind(email_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    Some(SandboxDynamicData {
        attachments_analyzed: total,
        malicious_count,
        reports: rows
            .into_iter()
            .map(
                |(filename, malscore, network_hosts, persistence, verdict)| DynamicSandboxReport {
                    filename,
                    verdict,
                    malscore,
                    network_hosts,
                    persistence,
                },
            )
            .collect(),
    })
}
