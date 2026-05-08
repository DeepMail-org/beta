/// Core enrichment orchestrator: cache lookup → circuit breaker → provider call → cache store.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use redis::aio::ConnectionManager;
use sqlx::PgPool;

use crate::cache;
use crate::circuit::CircuitRegistry;
use crate::error::IntelError;
use crate::providers::abuseipdb::AbuseIpDbClient;
use crate::providers::greynoise::GreyNoiseClient;
use crate::providers::ipinfo::IpInfoClient;
use crate::providers::otx::OtxClient;
use crate::providers::shodan::ShodanClient;
use crate::providers::virustotal::VtClient;
use crate::telemetry::TelemetryAccumulator;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IocEnrichment {
    pub ioc_value: String,
    pub ioc_type: String,
    pub provider_results: HashMap<String, serde_json::Value>,
    pub max_score: f32,
    pub is_malicious: bool,
    pub fetched_at: chrono::DateTime<chrono::Utc>,
}

/// Shared context for all enrichment operations.
pub struct EnrichCtx {
    pub pool: PgPool,
    pub redis: ConnectionManager,
    pub vt: Arc<VtClient>,
    pub abuse: Arc<AbuseIpDbClient>,
    pub greynoise: Arc<GreyNoiseClient>,
    pub ipinfo: Arc<IpInfoClient>,
    pub shodan: Arc<ShodanClient>,
    pub otx: Arc<OtxClient>,
    pub circuits: Arc<CircuitRegistry>,
    pub telemetry: Arc<TelemetryAccumulator>,
}

/// Provider names applicable to each IOC type.
fn default_providers_for(ioc_type: &str) -> Vec<&'static str> {
    match ioc_type {
        "ip" => vec![
            "virustotal",
            "abuseipdb",
            "greynoise",
            "ipinfo",
            "shodan",
            "otx",
        ],
        "domain" => vec!["virustotal", "otx"],
        "url" => vec!["virustotal"],
        "hash" => vec!["virustotal", "otx"],
        _ => vec!["virustotal"],
    }
}

pub async fn enrich_ioc(
    ioc_value: &str,
    ioc_type: &str,
    providers: &[String],
    force_refresh: bool,
    ctx: &EnrichCtx,
) -> Result<IocEnrichment, IntelError> {
    let provider_list: Vec<&str> = if providers.is_empty() {
        default_providers_for(ioc_type)
    } else {
        providers.iter().map(|s| s.as_str()).collect()
    };

    let mut results: HashMap<String, serde_json::Value> = HashMap::new();
    let mut max_vt_score: f32 = 0.0;
    let mut max_abuse_score: i32 = 0;
    let mut max_pulse_count: i32 = 0;

    for &provider in &provider_list {
        // 1. Check cache (unless force_refresh)
        if !force_refresh {
            // Redis first
            let mut redis = ctx.redis.clone();
            if let Ok(Some(cached)) =
                cache::get_cached_redis(&mut redis, provider, ioc_type, ioc_value).await
            {
                ctx.telemetry
                    .record_request(provider, 0, true, true)
                    .await;
                results.insert(provider.to_string(), cached);
                continue;
            }

            // DB fallback
            if let Ok(Some(cached)) =
                cache::get_cached_db(&ctx.pool, ioc_type, ioc_value, provider).await
            {
                ctx.telemetry
                    .record_request(provider, 0, true, true)
                    .await;
                // Re-populate Redis from DB
                let ttl = cache::provider_ttl(provider, ioc_type);
                let remaining = (cached.expires_at - chrono::Utc::now()).num_seconds().max(60) as u64;
                let _ = cache::set_cached_redis(
                    &mut redis,
                    provider,
                    ioc_type,
                    ioc_value,
                    &cached.result_json,
                    remaining.min(ttl),
                )
                .await;
                results.insert(provider.to_string(), cached.result_json);
                continue;
            }
        }

        // 2. Check circuit breaker
        let cb = match ctx.circuits.get(provider) {
            Some(cb) => cb,
            None => {
                tracing::warn!(provider = provider, "no circuit breaker registered");
                continue;
            }
        };

        // 3. Call provider via circuit breaker
        let start = Instant::now();
        let api_result = match provider {
            "virustotal" => {
                cb.call(|| call_virustotal(&ctx.vt, ioc_type, ioc_value))
                    .await
            }
            "abuseipdb" => {
                cb.call(|| call_abuseipdb(&ctx.abuse, ioc_value)).await
            }
            "greynoise" => {
                cb.call(|| call_greynoise(&ctx.greynoise, ioc_value))
                    .await
            }
            "ipinfo" => {
                cb.call(|| call_ipinfo(&ctx.ipinfo, ioc_value)).await
            }
            "shodan" => {
                cb.call(|| call_shodan(&ctx.shodan, ioc_value)).await
            }
            "otx" => {
                cb.call(|| call_otx(&ctx.otx, ioc_type, ioc_value))
                    .await
            }
            _ => {
                tracing::warn!(provider = provider, "unknown provider");
                continue;
            }
        };

        let latency_ms = start.elapsed().as_millis() as u64;

        match api_result {
            Ok(pr) => {
                ctx.telemetry
                    .record_request(provider, latency_ms, true, false)
                    .await;

                // Cache in Redis + DB
                let ttl = cache::provider_ttl(provider, ioc_type);
                let mut redis = ctx.redis.clone();
                let _ = cache::set_cached_redis(
                    &mut redis,
                    provider,
                    ioc_type,
                    ioc_value,
                    &pr.value,
                    ttl,
                )
                .await;
                let _ = cache::set_cached_db(
                    &ctx.pool,
                    ioc_type,
                    ioc_value,
                    provider,
                    &pr.value,
                    pr.vt_score,
                    pr.abuse_score,
                    pr.pulse_count,
                    ttl as i64,
                )
                .await;

                if let Some(vts) = pr.vt_score {
                    max_vt_score = max_vt_score.max(vts);
                }
                if let Some(abs) = pr.abuse_score {
                    max_abuse_score = max_abuse_score.max(abs);
                }
                if let Some(pc) = pr.pulse_count {
                    max_pulse_count = max_pulse_count.max(pc);
                }

                results.insert(provider.to_string(), pr.value);
            }
            Err(e) => {
                ctx.telemetry
                    .record_request(provider, latency_ms, false, false)
                    .await;
                tracing::warn!(
                    provider = provider,
                    ioc = ioc_value,
                    error = %e,
                    "provider enrichment failed, continuing"
                );
                // Continue with other providers (partial result)
            }
        }
    }

    let max_score = max_vt_score;
    let is_malicious =
        max_vt_score > 0.5 || max_abuse_score > 80 || max_pulse_count > 5;

    Ok(IocEnrichment {
        ioc_value: ioc_value.to_string(),
        ioc_type: ioc_type.to_string(),
        provider_results: results,
        max_score,
        is_malicious,
        fetched_at: chrono::Utc::now(),
    })
}

/// Internal result wrapper for provider calls.
struct ProviderCallResult {
    value: serde_json::Value,
    vt_score: Option<f32>,
    abuse_score: Option<i32>,
    pulse_count: Option<i32>,
}

async fn call_virustotal(
    vt: &Arc<VtClient>,
    ioc_type: &str,
    ioc_value: &str,
) -> Result<ProviderCallResult, IntelError> {
    let result = match ioc_type {
        "ip" => vt.lookup_ip(ioc_value).await?,
        "domain" => vt.lookup_domain(ioc_value).await?,
        "url" => vt.lookup_url(ioc_value).await?,
        "hash" => vt.lookup_hash(ioc_value).await?,
        _ => return Err(IntelError::Parse(format!("VT unsupported ioc_type: {ioc_type}"))),
    };

    let value = serde_json::to_value(&result).map_err(|e| IntelError::Parse(e.to_string()))?;
    Ok(ProviderCallResult {
        value,
        vt_score: Some(result.vt_score),
        abuse_score: None,
        pulse_count: None,
    })
}

async fn call_abuseipdb(
    abuse: &Arc<AbuseIpDbClient>,
    ip: &str,
) -> Result<ProviderCallResult, IntelError> {
    let result = abuse.lookup_ip(ip).await?;
    let value = serde_json::to_value(&result).map_err(|e| IntelError::Parse(e.to_string()))?;
    Ok(ProviderCallResult {
        value,
        vt_score: None,
        abuse_score: Some(result.abuse_score),
        pulse_count: None,
    })
}

async fn call_greynoise(
    gn: &Arc<GreyNoiseClient>,
    ip: &str,
) -> Result<ProviderCallResult, IntelError> {
    let result = gn.lookup_ip(ip).await?;
    let value = serde_json::to_value(&result).map_err(|e| IntelError::Parse(e.to_string()))?;
    Ok(ProviderCallResult {
        value,
        vt_score: None,
        abuse_score: None,
        pulse_count: None,
    })
}

async fn call_ipinfo(
    client: &Arc<IpInfoClient>,
    ip: &str,
) -> Result<ProviderCallResult, IntelError> {
    let result = client.lookup_ip(ip).await?;
    let value = serde_json::to_value(&result).map_err(|e| IntelError::Parse(e.to_string()))?;
    Ok(ProviderCallResult {
        value,
        vt_score: None,
        abuse_score: None,
        pulse_count: None,
    })
}

async fn call_shodan(
    client: &Arc<ShodanClient>,
    ip: &str,
) -> Result<ProviderCallResult, IntelError> {
    let result = client.lookup_ip(ip).await?;
    let value = serde_json::to_value(&result).map_err(|e| IntelError::Parse(e.to_string()))?;
    Ok(ProviderCallResult {
        value,
        vt_score: None,
        abuse_score: None,
        pulse_count: None,
    })
}

async fn call_otx(
    otx: &Arc<OtxClient>,
    ioc_type: &str,
    ioc_value: &str,
) -> Result<ProviderCallResult, IntelError> {
    let result = match ioc_type {
        "ip" => otx.lookup_ip(ioc_value).await?,
        "domain" => otx.lookup_domain(ioc_value).await?,
        "hash" => otx.lookup_hash(ioc_value).await?,
        _ => return Err(IntelError::Parse(format!("OTX unsupported ioc_type: {ioc_type}"))),
    };

    let value = serde_json::to_value(&result).map_err(|e| IntelError::Parse(e.to_string()))?;
    Ok(ProviderCallResult {
        value,
        vt_score: None,
        abuse_score: None,
        pulse_count: Some(result.pulse_count),
    })
}
