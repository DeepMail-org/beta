use std::net::IpAddr;
use std::time::Duration;

use redis::AsyncCommands;
use serde::{Deserialize, Serialize};

use crate::error::GeoError;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnrichmentResult {
    pub ipinfo: Option<IpInfoResponse>,
    pub abuseipdb: Option<AbuseIpDbResponse>,
    pub greynoise: Option<GreyNoiseResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpInfoResponse {
    pub ip: String,
    #[serde(default)]
    pub hostname: String,
    #[serde(default)]
    pub city: String,
    #[serde(default)]
    pub region: String,
    #[serde(default)]
    pub country: String,
    #[serde(default)]
    pub loc: String,
    #[serde(default)]
    pub org: String,
    #[serde(default)]
    pub postal: String,
    #[serde(default)]
    pub timezone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbuseIpDbResponse {
    pub data: AbuseIpDbData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbuseIpDbData {
    #[serde(rename = "abuseConfidenceScore")]
    pub abuse_confidence_score: i32,
    #[serde(rename = "countryCode", default)]
    pub country_code: String,
    #[serde(default)]
    pub isp: String,
    #[serde(rename = "totalReports", default)]
    pub total_reports: i32,
    #[serde(rename = "lastReportedAt", default)]
    pub last_reported_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GreyNoiseResponse {
    pub ip: String,
    #[serde(default)]
    pub noise: bool,
    #[serde(default)]
    pub riot: bool,
    #[serde(default)]
    pub classification: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub link: String,
    #[serde(default)]
    pub last_seen: String,
    #[serde(default)]
    pub message: String,
}

pub async fn fetch_ipinfo(
    client: &reqwest::Client,
    ip: IpAddr,
    token: &str,
) -> Result<IpInfoResponse, GeoError> {
    let url = format!("https://ipinfo.io/{ip}/json?token={token}");
    let resp = client
        .get(&url)
        .timeout(Duration::from_secs(3))
        .send()
        .await
        .map_err(|e| GeoError::Http(format!("ipinfo: {e}")))?;

    if resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS
        || resp.status().is_server_error()
    {
        tokio::time::sleep(Duration::from_millis(500)).await;
        let retry = client
            .get(&url)
            .timeout(Duration::from_secs(3))
            .send()
            .await
            .map_err(|e| GeoError::Http(format!("ipinfo retry: {e}")))?;
        return retry
            .json::<IpInfoResponse>()
            .await
            .map_err(|e| GeoError::Http(format!("ipinfo parse: {e}")));
    }

    resp.json::<IpInfoResponse>()
        .await
        .map_err(|e| GeoError::Http(format!("ipinfo parse: {e}")))
}

pub async fn fetch_abuseipdb(
    client: &reqwest::Client,
    ip: IpAddr,
    api_key: &str,
) -> Result<AbuseIpDbResponse, GeoError> {
    let resp = client
        .get("https://api.abuseipdb.com/api/v2/check")
        .header("Key", api_key)
        .header("Accept", "application/json")
        .query(&[("ipAddress", ip.to_string()), ("maxAgeInDays", "90".to_string())])
        .timeout(Duration::from_secs(3))
        .send()
        .await
        .map_err(|e| GeoError::Http(format!("abuseipdb: {e}")))?;

    if resp.status().is_server_error() {
        tokio::time::sleep(Duration::from_millis(500)).await;
        let retry = client
            .get("https://api.abuseipdb.com/api/v2/check")
            .header("Key", api_key)
            .header("Accept", "application/json")
            .query(&[("ipAddress", ip.to_string()), ("maxAgeInDays", "90".to_string())])
            .timeout(Duration::from_secs(3))
            .send()
            .await
            .map_err(|e| GeoError::Http(format!("abuseipdb retry: {e}")))?;
        return retry
            .json::<AbuseIpDbResponse>()
            .await
            .map_err(|e| GeoError::Http(format!("abuseipdb parse: {e}")));
    }

    resp.json::<AbuseIpDbResponse>()
        .await
        .map_err(|e| GeoError::Http(format!("abuseipdb parse: {e}")))
}

pub async fn fetch_greynoise(
    client: &reqwest::Client,
    ip: IpAddr,
    api_key: &str,
) -> Result<GreyNoiseResponse, GeoError> {
    let url = format!("https://api.greynoise.io/v3/community/{ip}");
    let resp = client
        .get(&url)
        .header("key", api_key)
        .timeout(Duration::from_secs(3))
        .send()
        .await
        .map_err(|e| GeoError::Http(format!("greynoise: {e}")))?;

    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(GreyNoiseResponse {
            ip: ip.to_string(),
            noise: false,
            riot: false,
            classification: "unknown".to_string(),
            name: String::new(),
            link: String::new(),
            last_seen: String::new(),
            message: "IP not observed".to_string(),
        });
    }

    if resp.status().is_server_error() {
        tokio::time::sleep(Duration::from_millis(500)).await;
        let retry = client
            .get(&url)
            .header("key", api_key)
            .timeout(Duration::from_secs(3))
            .send()
            .await
            .map_err(|e| GeoError::Http(format!("greynoise retry: {e}")))?;
        return retry
            .json::<GreyNoiseResponse>()
            .await
            .map_err(|e| GeoError::Http(format!("greynoise parse: {e}")));
    }

    resp.json::<GreyNoiseResponse>()
        .await
        .map_err(|e| GeoError::Http(format!("greynoise parse: {e}")))
}

pub struct EnrichmentConfig {
    pub ipinfo_token: String,
    pub abuseipdb_api_key: String,
    pub greynoise_api_key: String,
    pub cache_ttl: u64,
}

pub async fn enrich_ip(
    client: &reqwest::Client,
    redis: &mut redis::aio::ConnectionManager,
    ip: IpAddr,
    config: &EnrichmentConfig,
) -> EnrichmentResult {
    let cache_key = format!("deepmail:geo:enrich:{ip}");

    if let Ok(cached) = redis.get::<_, Option<String>>(&cache_key).await {
        if let Some(json_str) = cached {
            if let Ok(result) = serde_json::from_str::<EnrichmentResult>(&json_str) {
                return result;
            }
        }
    }

    let ipinfo_fut = async {
        if config.ipinfo_token.is_empty() {
            return None;
        }
        match fetch_ipinfo(client, ip, &config.ipinfo_token).await {
            Ok(r) => Some(r),
            Err(e) => {
                tracing::warn!(ip = %ip, error = %e, "ipinfo enrichment failed");
                None
            }
        }
    };

    let abuse_fut = async {
        if config.abuseipdb_api_key.is_empty() {
            return None;
        }
        match fetch_abuseipdb(client, ip, &config.abuseipdb_api_key).await {
            Ok(r) => Some(r),
            Err(e) => {
                tracing::warn!(ip = %ip, error = %e, "abuseipdb enrichment failed");
                None
            }
        }
    };

    let greynoise_fut = async {
        if config.greynoise_api_key.is_empty() {
            return None;
        }
        match fetch_greynoise(client, ip, &config.greynoise_api_key).await {
            Ok(r) => Some(r),
            Err(e) => {
                tracing::warn!(ip = %ip, error = %e, "greynoise enrichment failed");
                None
            }
        }
    };

    let (ipinfo, abuseipdb, greynoise) = tokio::join!(ipinfo_fut, abuse_fut, greynoise_fut);

    let result = EnrichmentResult {
        ipinfo,
        abuseipdb,
        greynoise,
    };

    if let Ok(json_str) = serde_json::to_string(&result) {
        let _: Result<(), _> = redis
            .set_ex::<_, _, ()>(&cache_key, &json_str, config.cache_ttl)
            .await;
    }

    result
}
