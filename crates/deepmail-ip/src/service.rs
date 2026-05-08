/// gRPC IpIntelligenceService implementation.

use std::net::IpAddr;
use std::str::FromStr;

use chrono::Utc;
use redis::aio::ConnectionManager;
use sqlx::PgPool;
use tonic::{Request, Response, Status};

use deepmail_common::proto::ip::{
    ip_intelligence_server::IpIntelligence,
    IpAnalyzeRequest, IpAnalyzeResponse,
    IpBulkCheckRequest, IpBulkCheckResponse,
    IpCheckRequest, IpCheckResponse,
    IpResult,
};

use crate::db::{analyses, reputation};
use crate::pipeline;


pub struct IpService {
    pub ip_pool: PgPool,
    pub ingest_pool: PgPool,
    pub parser_pool: PgPool,
    pub redis: ConnectionManager,
    pub http_client: reqwest::Client,
    pub shodan_api_key: String,
}

fn row_to_proto(r: &reputation::IpReputationRow) -> IpResult {
    IpResult {
        ip_address: r.ip_address.clone(),
        threat_score: r.threat_score,
        threat_verdict: r.threat_verdict.clone(),
        feed_hits: r.feed_hits.clone(),
        abuse_score: r.abuse_score.unwrap_or(0),
        shodan_ports: r.shodan_ports.clone(),
        shodan_tags: r.shodan_tags.clone(),
        shodan_vulns: r.shodan_vulns.clone(),
        shodan_org: r.shodan_org.clone().unwrap_or_default(),
        pdns_hostnames: r.pdns_hostnames.clone(),
        bgp_prefix: r.bgp_prefix.clone().unwrap_or_default(),
        bgp_asn: r.bgp_asn.unwrap_or(0),
        bgp_holder: r.bgp_holder.clone().unwrap_or_default(),
        bgp_announced: r.bgp_announced,
        is_bogon: r.is_bogon,
        sighting_count: r.sighting_count,
    }
}

fn pipeline_result_to_proto(r: &pipeline::IpResult) -> IpResult {
    IpResult {
        ip_address: r.ip.clone(),
        threat_score: r.score,
        threat_verdict: r.verdict.clone(),
        feed_hits: r.feeds_hit.clone(),
        abuse_score: 0,
        shodan_ports: Vec::new(),
        shodan_tags: Vec::new(),
        shodan_vulns: Vec::new(),
        shodan_org: String::new(),
        pdns_hostnames: Vec::new(),
        bgp_prefix: String::new(),
        bgp_asn: 0,
        bgp_holder: String::new(),
        bgp_announced: true,
        is_bogon: false,
        sighting_count: 1,
    }
}

#[tonic::async_trait]
impl IpIntelligence for IpService {
    /// Idempotent email analysis — returns cached result if available.
    async fn analyze_email(
        &self,
        request: Request<IpAnalyzeRequest>,
    ) -> Result<Response<IpAnalyzeResponse>, Status> {
        let req = request.into_inner();

        let email_id: uuid::Uuid = req
            .email_id
            .parse()
            .map_err(|_| Status::invalid_argument("invalid email_id UUID"))?;
        let tenant_id: uuid::Uuid = req
            .tenant_id
            .parse()
            .map_err(|_| Status::invalid_argument("invalid tenant_id UUID"))?;

        // Idempotency check
        if let Ok(Some(existing)) = analyses::get_by_email_id(&self.ip_pool, email_id).await {
            let summary_ips = existing
                .summary_json
                .get("ips")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| serde_json::from_value::<pipeline::IpResult>(v.clone()).ok())
                        .map(|r| pipeline_result_to_proto(&r))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            return Ok(Response::new(IpAnalyzeResponse {
                email_id: email_id.to_string(),
                ips_analyzed: summary_ips.len() as i32,
                max_threat_score: existing.max_threat_score,
                max_verdict: existing.max_verdict,
                results: summary_ips,
                cached: true,
            }));
        }

        let state = pipeline::PipelineState {
            ip_pool: self.ip_pool.clone(),
            ingest_pool: self.ingest_pool.clone(),
            parser_pool: self.parser_pool.clone(),
            redis: self.redis.clone(),
            http_client: self.http_client.clone(),
            shodan_api_key: self.shodan_api_key.clone(),
        };

        let output = pipeline::analyze(&state, email_id, tenant_id)
            .await
            .map_err(|e| -> Status { e.into() })?;

        let results: Vec<IpResult> = output
            .ip_results
            .iter()
            .map(|r| pipeline_result_to_proto(r))
            .collect();

        Ok(Response::new(IpAnalyzeResponse {
            email_id: email_id.to_string(),
            ips_analyzed: output.ips_analyzed as i32,
            max_threat_score: output.max_threat_score,
            max_verdict: output.max_verdict,
            results,
            cached: false,
        }))
    }

    /// Single IP enrichment + scoring. Always runs fresh.
    async fn check_ip(
        &self,
        request: Request<IpCheckRequest>,
    ) -> Result<Response<IpCheckResponse>, Status> {
        let req = request.into_inner();
        let ip: IpAddr = IpAddr::from_str(&req.ip_address)
            .map_err(|_| Status::invalid_argument("invalid IP address"))?;

        let mut redis = self.redis.clone();
        let result = pipeline::enrich_ip_standalone(
            &self.ip_pool,
            &mut redis,
            &self.http_client,
            ip,
            &self.shodan_api_key,
        )
        .await
        .map_err(|e| -> Status { e.into() })?;

        // Re-fetch the full reputation record for the proto response
        let rep = reputation::get_by_ip(&self.ip_pool, &req.ip_address)
            .await
            .map_err(|e| -> Status { e.into() })?;

        let proto_result = match rep {
            Some(r) => row_to_proto(&r),
            None => pipeline_result_to_proto(&result),
        };

        Ok(Response::new(IpCheckResponse {
            result: Some(proto_result),
        }))
    }

    /// Bulk IP check — up to 500 IPs. Uses cached reputation if fresh.
    async fn bulk_check_ip(
        &self,
        request: Request<IpBulkCheckRequest>,
    ) -> Result<Response<IpBulkCheckResponse>, Status> {
        let req = request.into_inner();

        if req.ip_addresses.len() > 500 {
            return Err(Status::invalid_argument("maximum 500 IPs per request"));
        }

        let ip_strs = req.ip_addresses;
        let one_hour_ago = Utc::now() - chrono::Duration::hours(1);

        // Try bulk fetch from reputation table
        let cached = reputation::bulk_get(&self.ip_pool, &ip_strs)
            .await
            .map_err(|e| -> Status { e.into() })?;

        let mut results: Vec<IpResult> = Vec::new();
        let mut stale_ips: Vec<(String, IpAddr)> = Vec::new();

        for ip_str in &ip_strs {
            let cached_row = cached.iter().find(|r| r.ip_address == *ip_str);
            match cached_row {
                Some(r) if r.last_seen > one_hour_ago => {
                    results.push(row_to_proto(r));
                }
                _ => {
                    if let Ok(ip) = IpAddr::from_str(ip_str) {
                        stale_ips.push((ip_str.clone(), ip));
                    }
                }
            }
        }

        // Enrich stale IPs concurrently
        let mut handles = Vec::new();
        for (ip_str, ip) in stale_ips {
            let pool = self.ip_pool.clone();
            let mut redis = self.redis.clone();
            let http = self.http_client.clone();
            let key = self.shodan_api_key.clone();

            handles.push(tokio::spawn(async move {
                let result = pipeline::enrich_ip_standalone(
                    &pool, &mut redis, &http, ip, &key,
                )
                .await;
                (ip_str, result)
            }));
        }

        for handle in handles {
            match handle.await {
                Ok((ip_str, Ok(result))) => {
                    // Re-fetch full row
                    if let Ok(Some(r)) = reputation::get_by_ip(&self.ip_pool, &ip_str).await {
                        results.push(row_to_proto(&r));
                    } else {
                        results.push(pipeline_result_to_proto(&result));
                    }
                }
                Ok((ip_str, Err(e))) => {
                    tracing::warn!(ip = %ip_str, error = %e, "bulk check enrichment failed");
                    // Return a minimal result for failed IPs
                    results.push(IpResult {
                        ip_address: ip_str,
                        threat_score: 0.0,
                        threat_verdict: "CLEAN".to_string(),
                        ..Default::default()
                    });
                }
                Err(e) => {
                    tracing::warn!(error = %e, "bulk check task panicked");
                }
            }
        }

        Ok(Response::new(IpBulkCheckResponse { results }))
    }
}
