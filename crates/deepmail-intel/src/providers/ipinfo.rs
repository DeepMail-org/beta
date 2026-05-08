/// IPInfo geolocation/ASN client.

use std::sync::Arc;

use crate::error::IntelError;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IpInfoResult {
    pub ip: String,
    pub hostname: Option<String>,
    pub city: Option<String>,
    pub region: Option<String>,
    pub country: Option<String>,
    pub org: Option<String>,
    pub postal: Option<String>,
    pub timezone: Option<String>,
    pub loc: Option<String>,
    pub raw_json: serde_json::Value,
}

impl Default for IpInfoResult {
    fn default() -> Self {
        Self {
            ip: String::new(), hostname: None, city: None, region: None,
            country: None, org: None, postal: None, timezone: None,
            loc: None, raw_json: serde_json::Value::Null,
        }
    }
}

pub struct IpInfoClient {
    client: Arc<reqwest::Client>,
    token: String,
}

impl IpInfoClient {
    pub fn new(client: Arc<reqwest::Client>, token: String) -> Self {
        Self { client, token }
    }

    pub async fn lookup_ip(&self, ip: &str) -> Result<IpInfoResult, IntelError> {
        let url = if self.token.is_empty() {
            format!("https://ipinfo.io/{ip}/json")
        } else {
            format!("https://ipinfo.io/{ip}/json?token={}", self.token)
        };

        let resp = self.client
            .get(&url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(IntelError::Http(format!("IPInfo: {}", resp.status())));
        }

        let body: serde_json::Value = resp.json().await?;

        Ok(IpInfoResult {
            ip: body.get("ip").and_then(|v| v.as_str()).unwrap_or(ip).to_string(),
            hostname: body.get("hostname").and_then(|v| v.as_str()).map(|s| s.to_string()),
            city: body.get("city").and_then(|v| v.as_str()).map(|s| s.to_string()),
            region: body.get("region").and_then(|v| v.as_str()).map(|s| s.to_string()),
            country: body.get("country").and_then(|v| v.as_str()).map(|s| s.to_string()),
            org: body.get("org").and_then(|v| v.as_str()).map(|s| s.to_string()),
            postal: body.get("postal").and_then(|v| v.as_str()).map(|s| s.to_string()),
            timezone: body.get("timezone").and_then(|v| v.as_str()).map(|s| s.to_string()),
            loc: body.get("loc").and_then(|v| v.as_str()).map(|s| s.to_string()),
            raw_json: body,
        })
    }
}
