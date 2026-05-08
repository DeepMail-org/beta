/// Shodan API enrichment client with Redis caching.

use std::collections::HashMap;
use std::net::IpAddr;

use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};

use crate::error::IpError;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ShodanResponse {
    pub ip_str: Option<String>,
    pub ports: Option<Vec<i32>>,
    pub tags: Option<Vec<String>>,
    pub vulns: Option<HashMap<String, serde_json::Value>>,
    pub org: Option<String>,
    pub isp: Option<String>,
    pub os: Option<String>,
    pub last_update: Option<String>,
    pub hostnames: Option<Vec<String>>,
    pub country_code: Option<String>,
    pub asn: Option<String>,
}

fn redis_key(ip: IpAddr) -> String {
    format!("deepmail:ip:shodan:{ip}")
}

/// Check Redis for a cached Shodan response.
pub async fn get_shodan_cached(
    redis: &mut ConnectionManager,
    ip: IpAddr,
) -> Result<Option<ShodanResponse>, IpError> {
    let key = redis_key(ip);
    let cached: Option<String> = redis.get(&key).await?;
    match cached {
        Some(json) => {
            let resp: ShodanResponse =
                serde_json::from_str(&json).map_err(|e| IpError::Parse(e.to_string()))?;
            Ok(Some(resp))
        }
        None => Ok(None),
    }
}

/// Fetch Shodan enrichment for an IP. Returns Ok(None) if API key is empty,
/// on 404 (unknown IP), or on 429 (rate limit).
pub async fn fetch_shodan(
    client: &reqwest::Client,
    redis: &mut ConnectionManager,
    ip: IpAddr,
    api_key: &str,
) -> Result<Option<ShodanResponse>, IpError> {
    if api_key.is_empty() {
        tracing::info!(ip = %ip, "Shodan API key not configured, skipping");
        return Ok(None);
    }

    // Check cache first
    if let Some(cached) = get_shodan_cached(redis, ip).await? {
        return Ok(Some(cached));
    }

    let url = format!("https://api.shodan.io/shodan/host/{ip}?key={api_key}");
    let resp = client.get(&url).send().await?;
    let status = resp.status();

    if status.as_u16() == 404 {
        // IP not in Shodan index — valid, not an error
        return Ok(None);
    }

    if status.as_u16() == 429 {
        tracing::warn!(ip = %ip, "Shodan rate limit hit");
        return Ok(None);
    }

    if status.as_u16() == 401 || status.as_u16() == 403 {
        return Err(IpError::Http(format!("Shodan auth error: {status}")));
    }

    if !status.is_success() {
        return Err(IpError::Http(format!("Shodan error: {status}")));
    }

    let body = resp.text().await?;
    let parsed: ShodanResponse =
        serde_json::from_str(&body).map_err(|e| IpError::Parse(e.to_string()))?;

    // Cache in Redis with 24h TTL
    let key = redis_key(ip);
    let _: Result<(), _> = redis.set_ex(&key, &body, 86400).await;

    Ok(Some(parsed))
}
