use std::net::IpAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;

use chrono::Utc;
use redis::aio::ConnectionManager;
use sqlx::PgPool;
use uuid::Uuid;

use crate::blocklist;
use crate::classifier;
use crate::db::{analyses, hops};
use crate::enrichment::{self, EnrichmentConfig};
use crate::error::GeoError;
use crate::geoip::GeoIpReader;
use crate::risk;
use crate::tor;
use crate::vpn;

pub struct PipelineState {
    pub geo_pool: PgPool,
    pub ingest_pool: PgPool,
    pub parser_pool: PgPool,
    pub redis: ConnectionManager,
    pub geoip: Arc<GeoIpReader>,
    pub http_client: reqwest::Client,
    pub enrichment_config: Arc<EnrichmentConfig>,
}

pub struct AnalysisOutput {
    pub email_id: Uuid,
    pub hop_count: usize,
    pub origin_ip: Option<String>,
    pub origin_country: Option<String>,
    pub overall_risk: String,
    pub risk_score: f32,
    pub duration_ms: i32,
}

pub async fn analyze(
    state: &PipelineState,
    email_id: Uuid,
    tenant_id: Uuid,
) -> Result<AnalysisOutput, GeoError> {
    let start = Instant::now();

    let hop_ips = fetch_hop_ips(&state.parser_pool, email_id).await?;

    let public_hops: Vec<(i32, IpAddr)> = hop_ips
        .into_iter()
        .filter(|(_, ip)| !classifier::is_private(*ip))
        .collect();

    if public_hops.is_empty() {
        let analysis_id = analyses::upsert_analysis(
            &state.geo_pool, email_id, tenant_id, 0,
            None, None, None, "NONE", 0.0,
        ).await?;
        let _ = analysis_id;

        let duration_ms = start.elapsed().as_millis() as i32;
        update_job_progress(&state.ingest_pool, email_id, duration_ms).await?;

        return Ok(AnalysisOutput {
            email_id,
            hop_count: 0,
            origin_ip: None,
            origin_country: None,
            overall_risk: "NONE".to_string(),
            risk_score: 0.0,
            duration_ms,
        });
    }

    let mut hop_results: Vec<HopResult> = Vec::new();

    for (hop_index, ip) in &public_hops {
        let geo = state.geoip.lookup_city(*ip);
        let asn_data = state.geoip.lookup_asn(*ip);

        let as_org_str = asn_data.as_org.clone().unwrap_or_default();
        let ip_type = classifier::classify(&as_org_str);

        let mut redis = state.redis.clone();

        let mut redis_tor = redis.clone();
        let mut redis_vpn = redis.clone();
        let (is_tor, is_vpn_flag, is_botnet) = tokio::join!(
            tor::is_tor_exit(&mut redis_tor, *ip),
            vpn::is_vpn_ip(&mut redis_vpn, *ip, &as_org_str),
            blocklist::check_ip_in_blocklist(&state.geo_pool, *ip),
        );
        let is_tor = is_tor.unwrap_or(false);
        let is_vpn_flag = is_vpn_flag.unwrap_or(false);
        let is_botnet = is_botnet.unwrap_or(false);

        let enrichment = enrichment::enrich_ip(
            &state.http_client,
            &mut redis,
            *ip,
            &state.enrichment_config,
        ).await;

        let abuse_score = enrichment.abuseipdb
            .as_ref()
            .map(|a| a.data.abuse_confidence_score);
        let greynoise_class = enrichment.greynoise
            .as_ref()
            .map(|g| g.classification.clone());

        let (risk_score, _verdict) = risk::compute_risk(&risk::HopRiskInput {
            is_tor,
            is_vpn: is_vpn_flag,
            is_botnet,
            abuse_score,
            greynoise_class: greynoise_class.clone(),
            ip_type,
        });

        let ipinfo_raw = enrichment.ipinfo
            .as_ref()
            .and_then(|v| serde_json::to_value(v).ok());
        let abuseipdb_raw = enrichment.abuseipdb
            .as_ref()
            .and_then(|v| serde_json::to_value(v).ok());
        let greynoise_raw = enrichment.greynoise
            .as_ref()
            .and_then(|v| serde_json::to_value(v).ok());

        hop_results.push(HopResult {
            hop_index: *hop_index,
            ip: *ip,
            ip_type,
            geo,
            asn_data,
            is_tor,
            is_vpn: is_vpn_flag,
            is_botnet,
            abuse_score,
            greynoise_class,
            ipinfo_raw,
            abuseipdb_raw,
            greynoise_raw,
            risk_score,
        });
    }

    let origin = hop_results.first();
    let origin_ip = origin.map(|h| h.ip.to_string());
    let origin_country = origin.and_then(|h| h.geo.country_code.clone());
    let origin_asn = origin.and_then(|h| h.asn_data.asn.map(|a| a as i32));

    let max_risk = hop_results
        .iter()
        .map(|h| h.risk_score)
        .fold(0.0f32, f32::max);
    let overall_risk_str = if max_risk < 0.15 { "NONE" }
        else if max_risk < 0.35 { "LOW" }
        else if max_risk < 0.55 { "MEDIUM" }
        else if max_risk < 0.75 { "HIGH" }
        else { "CRITICAL" };

    let analysis_id = analyses::upsert_analysis(
        &state.geo_pool,
        email_id,
        tenant_id,
        hop_results.len() as i32,
        origin_ip.as_deref(),
        origin_country.as_deref(),
        origin_asn,
        overall_risk_str,
        max_risk,
    ).await?;

    for hr in &hop_results {
        hops::insert_hop(
            &state.geo_pool,
            analysis_id,
            hr.hop_index,
            &hr.ip.to_string(),
            hr.ip_type.as_str(),
            hr.geo.country_code.as_deref(),
            hr.geo.country_name.as_deref(),
            hr.geo.city.as_deref(),
            hr.geo.latitude,
            hr.geo.longitude,
            hr.geo.postal_code.as_deref(),
            hr.geo.timezone.as_deref(),
            hr.asn_data.asn.map(|a| a as i32),
            hr.asn_data.as_org.as_deref(),
            hr.asn_data.network.as_deref(),
            hr.is_tor,
            hr.is_vpn,
            hr.is_botnet,
            hr.abuse_score,
            hr.greynoise_class.as_deref(),
            hr.ipinfo_raw.as_ref(),
            hr.abuseipdb_raw.as_ref(),
            hr.greynoise_raw.as_ref(),
            Some(Utc::now()),
        ).await?;
    }

    let duration_ms = start.elapsed().as_millis() as i32;
    update_job_progress(&state.ingest_pool, email_id, duration_ms).await?;

    Ok(AnalysisOutput {
        email_id,
        hop_count: hop_results.len(),
        origin_ip,
        origin_country,
        overall_risk: overall_risk_str.to_string(),
        risk_score: max_risk,
        duration_ms,
    })
}

struct HopResult {
    hop_index: i32,
    ip: IpAddr,
    ip_type: classifier::IpType,
    geo: crate::geoip::GeoData,
    asn_data: crate::geoip::AsnData,
    is_tor: bool,
    is_vpn: bool,
    is_botnet: bool,
    abuse_score: Option<i32>,
    greynoise_class: Option<String>,
    ipinfo_raw: Option<serde_json::Value>,
    abuseipdb_raw: Option<serde_json::Value>,
    greynoise_raw: Option<serde_json::Value>,
    risk_score: f32,
}

fn extract_ip_from_host(host: &str) -> Option<IpAddr> {
    let trimmed = host.trim();
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        return IpAddr::from_str(&trimmed[1..trimmed.len()-1]).ok();
    }
    IpAddr::from_str(trimmed).ok()
}

async fn fetch_hop_ips(
    parser_pool: &PgPool,
    email_id: Uuid,
) -> Result<Vec<(i32, IpAddr)>, GeoError> {
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
        r#"SELECT hop_index, from_host, by_host
           FROM received_hops
           WHERE parsed_email_id = $1
           ORDER BY hop_index"#,
    )
    .bind(parsed_id)
    .fetch_all(parser_pool)
    .await?;

    let mut result = Vec::new();
    use sqlx::Row;
    for row in &rows {
        let hop_index: i32 = row.get("hop_index");
        let from_host: Option<String> = row.get("from_host");
        let by_host: Option<String> = row.get("by_host");

        if let Some(ref host) = from_host {
            if let Some(ip) = extract_ip_from_host(host) {
                result.push((hop_index, ip));
                continue;
            }
        }
        if let Some(ref host) = by_host {
            if let Some(ip) = extract_ip_from_host(host) {
                result.push((hop_index, ip));
            }
        }
    }

    Ok(result)
}

async fn update_job_progress(
    ingest_pool: &PgPool,
    email_id: Uuid,
    duration_ms: i32,
) -> Result<(), GeoError> {
    sqlx::query(
        r#"UPDATE job_progress
           SET status = 'completed',
               completed_at = now(),
               duration_ms = $1
           WHERE email_id = $2
             AND stage = 'geo'"#,
    )
    .bind(duration_ms)
    .bind(email_id)
    .execute(ingest_pool)
    .await?;
    Ok(())
}
