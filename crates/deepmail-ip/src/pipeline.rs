/// Per-email IP analysis orchestration pipeline.

use std::net::IpAddr;
use std::str::FromStr;

use std::time::Instant;

use redis::aio::ConnectionManager;
use sqlx::PgPool;
use uuid::Uuid;

use crate::bgp;
use crate::db::{analyses, entries, reputation};
use crate::error::IpError;

use crate::pdns;
use crate::scorer::{self, SignalSet, ThreatVerdict};
use crate::shodan;

/// Shared state for pipeline operations.
pub struct PipelineState {
    pub ip_pool: PgPool,
    pub ingest_pool: PgPool,
    pub parser_pool: PgPool,
    pub redis: ConnectionManager,
    pub http_client: reqwest::Client,
    pub shodan_api_key: String,
}

pub struct AnalysisOutput {
    pub email_id: Uuid,
    pub ips_analyzed: usize,
    pub max_threat_score: f32,
    pub max_verdict: String,
    pub ip_results: Vec<IpResult>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IpResult {
    pub ip: String,
    pub score: f32,
    pub verdict: String,
    pub feeds_hit: Vec<String>,
}

/// Full per-email IP analysis.
pub async fn analyze(
    state: &PipelineState,
    email_id: Uuid,
    tenant_id: Uuid,
) -> Result<AnalysisOutput, IpError> {
    let start = Instant::now();

    // a. Fetch received hop IPs from parser DB
    let hop_ips = fetch_hop_ips(&state.parser_pool, email_id).await?;

    // b. Filter private IPs
    let public_ips: Vec<IpAddr> = hop_ips
        .into_iter()
        .filter(|ip| !bgp::is_private(*ip))
        .collect();

    if public_ips.is_empty() {
        let summary = serde_json::json!({ "ips": [] });
        analyses::upsert_email_analysis(
            &state.ip_pool,
            email_id,
            tenant_id,
            &[],
            0.0,
            "CLEAN",
            &summary,
        )
        .await?;

        let duration_ms = start.elapsed().as_millis() as i32;
        update_job_progress(&state.ingest_pool, email_id, duration_ms).await?;

        return Ok(AnalysisOutput {
            email_id,
            ips_analyzed: 0,
            max_threat_score: 0.0,
            max_verdict: "CLEAN".to_string(),
            ip_results: Vec::new(),
        });
    }

    // c. Enrich each public IP concurrently
    let mut handles = Vec::new();

    for ip in &public_ips {
        let ip = *ip;
        let ip_pool = state.ip_pool.clone();
        let mut redis = state.redis.clone();
        let http_client = state.http_client.clone();
        let shodan_key = state.shodan_api_key.clone();

        handles.push(tokio::spawn(async move {
            enrich_single_ip(&ip_pool, &mut redis, &http_client, ip, &shodan_key).await
        }));
    }

    let mut ip_results: Vec<IpResult> = Vec::new();
    let mut max_score: f32 = 0.0;

    for handle in handles {
        match handle.await {
            Ok(Ok(result)) => {
                if result.score > max_score {
                    max_score = result.score;
                }
                ip_results.push(result);
            }
            Ok(Err(e)) => {
                tracing::warn!(error = %e, "single IP enrichment failed");
            }
            Err(e) => {
                tracing::warn!(error = %e, "IP enrichment task panicked");
            }
        }
    }

    let max_verdict = ThreatVerdict::from_score(max_score);

    // d. Persist email_ip_analyses
    let analyzed_ip_strs: Vec<String> = public_ips.iter().map(|ip| ip.to_string()).collect();
    let summary = serde_json::json!({ "ips": ip_results });

    analyses::upsert_email_analysis(
        &state.ip_pool,
        email_id,
        tenant_id,
        &analyzed_ip_strs,
        max_score,
        max_verdict.as_str(),
        &summary,
    )
    .await?;

    // e. Update ingest job_progress
    let duration_ms = start.elapsed().as_millis() as i32;
    update_job_progress(&state.ingest_pool, email_id, duration_ms).await?;

    Ok(AnalysisOutput {
        email_id,
        ips_analyzed: ip_results.len(),
        max_threat_score: max_score,
        max_verdict: max_verdict.as_str().to_string(),
        ip_results,
    })
}

/// Enrich a single IP: check feeds, Shodan, pDNS, BGP, compute score, upsert reputation.
async fn enrich_single_ip(
    pool: &PgPool,
    redis: &mut ConnectionManager,
    http_client: &reqwest::Client,
    ip: IpAddr,
    shodan_api_key: &str,
) -> Result<IpResult, IpError> {
    let ip_str = ip.to_string();

    // Check feeds via DB (covers both INET and CIDR containment)
    let feed_hits = entries::check_ip(pool, &ip_str).await?;

    // Shodan enrichment
    let shodan_data = shodan::fetch_shodan(http_client, redis, ip, shodan_api_key).await?;

    // pDNS enrichment
    let pdns_hostnames = pdns::fetch_pdns(http_client, pool, redis, ip).await?;

    // BGP enrichment
    let bgp_info = bgp::fetch_bgp(http_client, redis, ip).await?;

    // Build signal set
    let bogon = bgp::is_bogon(ip);

    let shodan_tags = shodan_data
        .as_ref()
        .and_then(|s| s.tags.clone())
        .unwrap_or_default();
    let shodan_has_vulns = shodan_data
        .as_ref()
        .and_then(|s| s.vulns.as_ref())
        .map(|v| !v.is_empty())
        .unwrap_or(false);
    let shodan_ports = shodan_data
        .as_ref()
        .and_then(|s| s.ports.clone())
        .unwrap_or_default();
    let shodan_vulns: Vec<String> = shodan_data
        .as_ref()
        .and_then(|s| s.vulns.as_ref())
        .map(|v| v.keys().cloned().collect())
        .unwrap_or_default();
    let shodan_org = shodan_data.as_ref().and_then(|s| s.org.clone());

    let signals = SignalSet {
        in_feodo: feed_hits.contains(&"feodo_tracker".to_string()),
        in_spamhaus_drop: feed_hits.contains(&"spamhaus_drop".to_string()),
        in_spamhaus_edrop: feed_hits.contains(&"spamhaus_edrop".to_string()),
        in_emerging_threats: feed_hits.contains(&"emerging_threats".to_string()),
        in_cins_army: feed_hits.contains(&"cins_army".to_string()),
        in_blocklist_de: feed_hits.contains(&"blocklist_de".to_string()),
        in_tor: feed_hits.contains(&"tor_exits".to_string()),
        in_brute_force: feed_hits.contains(&"brute_force_logins".to_string()),
        in_alienvault: feed_hits.contains(&"alienvault_otx".to_string()),
        abuse_score: None, // AbuseIPDB is handled by deepmail-geo
        shodan_tags: shodan_tags.clone(),
        shodan_has_vulns,
        is_bogon: bogon,
        pdns_hostname_count: pdns_hostnames.len(),
    };

    let (score, verdict) = scorer::compute_threat_score(&signals);

    // Persist Shodan cache in DB
    if let Some(ref shodan_resp) = shodan_data {
        let _ = crate::db::shodan::upsert_shodan_cache(pool, &ip_str, shodan_resp).await;
    }

    // Upsert ip_reputation
    let pdns_first: Option<chrono::DateTime<chrono::Utc>> = None; // aggregated from pdns records
    let pdns_last: Option<chrono::DateTime<chrono::Utc>> = None;

    reputation::upsert_reputation(
        pool,
        &ip_str,
        score,
        verdict.as_str(),
        &feed_hits,
        None,
        &shodan_ports,
        &shodan_tags,
        &shodan_vulns,
        shodan_org.as_deref(),
        &pdns_hostnames,
        pdns_first,
        pdns_last,
        bgp_info.prefix.as_deref(),
        bgp_info.asn,
        bgp_info.holder.as_deref(),
        bgp_info.announced,
        bogon,
    )
    .await?;

    Ok(IpResult {
        ip: ip_str,
        score,
        verdict: verdict.as_str().to_string(),
        feeds_hit: feed_hits,
    })
}

/// Enrich a single IP for the CheckIp RPC (no email context required).
pub async fn enrich_ip_standalone(
    pool: &PgPool,
    redis: &mut ConnectionManager,
    http_client: &reqwest::Client,
    ip: IpAddr,
    shodan_api_key: &str,
) -> Result<IpResult, IpError> {
    enrich_single_ip(pool, redis, http_client, ip, shodan_api_key).await
}

async fn fetch_hop_ips(
    parser_pool: &PgPool,
    email_id: Uuid,
) -> Result<Vec<IpAddr>, IpError> {
    let parsed_id_row = sqlx::query(
        r#"SELECT id FROM parsed_emails WHERE email_id = $1 LIMIT 1"#,
    )
    .bind(email_id)
    .fetch_optional(parser_pool)
    .await?;

    let parsed_id: Uuid = match parsed_id_row {
        Some(row) => {
            use sqlx::Row;
            row.get("id")
        }
        None => return Ok(Vec::new()),
    };

    let rows = sqlx::query(
        r#"SELECT CAST(from_ip AS TEXT) AS from_ip
           FROM received_hops
           WHERE parsed_email_id = $1
             AND from_ip IS NOT NULL"#,
    )
    .bind(parsed_id)
    .fetch_all(parser_pool)
    .await?;

    use sqlx::Row;
    let mut ips = Vec::new();
    for row in &rows {
        let ip_str: Option<String> = row.get("from_ip");
        if let Some(ref s) = ip_str {
            // CAST may include /32 suffix, strip it
            let clean = s.split('/').next().unwrap_or(s).trim();
            if let Ok(ip) = IpAddr::from_str(clean) {
                if !ips.contains(&ip) {
                    ips.push(ip);
                }
            }
        }
    }

    Ok(ips)
}

async fn update_job_progress(
    ingest_pool: &PgPool,
    email_id: Uuid,
    duration_ms: i32,
) -> Result<(), IpError> {
    sqlx::query(
        r#"UPDATE job_progress
           SET status = 'completed',
               completed_at = now(),
               duration_ms = $1
           WHERE email_id = $2
             AND stage = 'ip'"#,
    )
    .bind(duration_ms)
    .bind(email_id)
    .execute(ingest_pool)
    .await?;
    Ok(())
}
