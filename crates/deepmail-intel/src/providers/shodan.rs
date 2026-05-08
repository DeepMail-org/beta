/// Shodan API client (centralised version of deepmail-ip shodan.rs).


use std::sync::Arc;

use crate::error::IntelError;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct ShodanResult {
    pub ports: Vec<i32>,
    pub tags: Vec<String>,
    pub vulns: Vec<String>,
    pub org: Option<String>,
    pub isp: Option<String>,
    pub os: Option<String>,
    pub hostnames: Vec<String>,
    pub country_code: Option<String>,
    pub asn: Option<String>,
    pub raw_json: serde_json::Value,
}

pub struct ShodanClient {
    client: Arc<reqwest::Client>,
    api_key: String,
}

impl ShodanClient {
    pub fn new(client: Arc<reqwest::Client>, api_key: String) -> Self {
        Self { client, api_key }
    }

    pub fn is_configured(&self) -> bool {
        !self.api_key.is_empty()
    }

    pub async fn lookup_ip(&self, ip: &str) -> Result<ShodanResult, IntelError> {
        if !self.is_configured() {
            return Ok(ShodanResult::default());
        }

        let url = format!("https://api.shodan.io/shodan/host/{ip}?key={}", self.api_key);
        let resp = self.client
            .get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await?;

        let status = resp.status();

        if status.as_u16() == 404 {
            return Ok(ShodanResult::default());
        }

        if status.as_u16() == 429 {
            return Err(IntelError::RateLimited("Shodan".to_string()));
        }

        if status.as_u16() == 401 || status.as_u16() == 403 {
            return Err(IntelError::Http(format!("Shodan auth error: {status}")));
        }

        if !status.is_success() {
            return Err(IntelError::Http(format!("Shodan: {status}")));
        }

        let body: serde_json::Value = resp.json().await?;

        let ports = body
            .get("ports")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_i64().map(|n| n as i32)).collect())
            .unwrap_or_default();
        let tags = body
            .get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();
        let vulns_map: Option<&serde_json::Map<String, serde_json::Value>> =
            body.get("vulns").and_then(|v| v.as_object());
        let vulns: Vec<String> = vulns_map
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default();
        let hostnames = body
            .get("hostnames")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();
        let org = body.get("org").and_then(|v| v.as_str()).map(String::from);
        let isp = body.get("isp").and_then(|v| v.as_str()).map(String::from);
        let os = body.get("os").and_then(|v| v.as_str()).map(String::from);
        let country_code = body.get("country_code").and_then(|v| v.as_str()).map(String::from);
        let asn = body.get("asn").and_then(|v| v.as_str()).map(String::from);

        Ok(ShodanResult {
            ports, tags, vulns, org, isp, os, hostnames, country_code, asn,
            raw_json: body,
        })
    }
}
