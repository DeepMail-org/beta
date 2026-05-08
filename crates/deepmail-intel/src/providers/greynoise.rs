/// GreyNoise Community API client.

use std::sync::Arc;

use crate::error::IntelError;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GreyNoiseResult {
    pub classification: String,
    pub noise: bool,
    pub riot: bool,
    pub name: String,
    pub last_seen: Option<String>,
    pub raw_json: serde_json::Value,
}

impl Default for GreyNoiseResult {
    fn default() -> Self {
        Self {
            classification: "unknown".to_string(),
            noise: false,
            riot: false,
            name: String::new(),
            last_seen: None,
            raw_json: serde_json::Value::Null,
        }
    }
}

pub struct GreyNoiseClient {
    client: Arc<reqwest::Client>,
    api_key: String,
}

impl GreyNoiseClient {
    pub fn new(client: Arc<reqwest::Client>, api_key: String) -> Self {
        Self { client, api_key }
    }

    pub fn is_configured(&self) -> bool {
        !self.api_key.is_empty()
    }

    pub async fn lookup_ip(&self, ip: &str) -> Result<GreyNoiseResult, IntelError> {
        if !self.is_configured() {
            return Ok(GreyNoiseResult::default());
        }

        let url = format!("https://api.greynoise.io/v3/community/{ip}");
        let resp = self.client
            .get(&url)
            .header("key", &self.api_key)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await?;

        let status = resp.status();

        if status.as_u16() == 404 {
            return Ok(GreyNoiseResult::default());
        }

        if status.as_u16() == 429 {
            return Err(IntelError::RateLimited("GreyNoise".to_string()));
        }

        if !status.is_success() {
            return Err(IntelError::Http(format!("GreyNoise: {status}")));
        }

        let body: serde_json::Value = resp.json().await?;

        let classification = body
            .get("classification")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let noise = body.get("noise").and_then(|v| v.as_bool()).unwrap_or(false);
        let riot = body.get("riot").and_then(|v| v.as_bool()).unwrap_or(false);
        let name = body
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let last_seen = body
            .get("last_seen")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Ok(GreyNoiseResult {
            classification,
            noise,
            riot,
            name,
            last_seen,
            raw_json: body,
        })
    }
}
