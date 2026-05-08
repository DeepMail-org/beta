/// OTX AlienVault pulse lookup client.

use std::sync::Arc;

use crate::error::IntelError;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OtxResult {
    pub pulse_count: i32,
    pub pulse_names: Vec<String>,
    pub reputation: i32,
    pub raw_json: serde_json::Value,
}

impl Default for OtxResult {
    fn default() -> Self {
        Self {
            pulse_count: 0,
            pulse_names: Vec::new(),
            reputation: 0,
            raw_json: serde_json::Value::Null,
        }
    }
}

pub struct OtxClient {
    client: Arc<reqwest::Client>,
    api_key: String,
    base_url: String,
}

impl OtxClient {
    pub fn new(client: Arc<reqwest::Client>, api_key: String) -> Self {
        Self {
            client,
            api_key,
            base_url: "https://otx.alienvault.com/api/v1".to_string(),
        }
    }

    pub fn is_configured(&self) -> bool {
        !self.api_key.is_empty()
    }

    pub async fn lookup_ip(&self, ip: &str) -> Result<OtxResult, IntelError> {
        if !self.is_configured() {
            return Ok(OtxResult::default());
        }
        let url = format!("{}/indicators/IPv4/{ip}/general", self.base_url);
        self.fetch_and_parse(&url).await
    }

    pub async fn lookup_domain(&self, domain: &str) -> Result<OtxResult, IntelError> {
        if !self.is_configured() {
            return Ok(OtxResult::default());
        }
        let url = format!("{}/indicators/domain/{domain}/general", self.base_url);
        self.fetch_and_parse(&url).await
    }

    pub async fn lookup_hash(&self, hash: &str) -> Result<OtxResult, IntelError> {
        if !self.is_configured() {
            return Ok(OtxResult::default());
        }
        let url = format!("{}/indicators/file/{hash}/general", self.base_url);
        self.fetch_and_parse(&url).await
    }

    async fn fetch_and_parse(&self, url: &str) -> Result<OtxResult, IntelError> {
        let resp = self.client
            .get(url)
            .header("X-OTX-API-KEY", &self.api_key)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await;

        let resp = match resp {
            Ok(r) => r,
            Err(e) => {
                // Retry once on connection errors
                tracing::debug!(error = %e, url = url, "OTX first attempt failed, retrying");
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                self.client
                    .get(url)
                    .header("X-OTX-API-KEY", &self.api_key)
                    .timeout(std::time::Duration::from_secs(5))
                    .send()
                    .await?
            }
        };

        let status = resp.status();

        if status.as_u16() == 404 {
            return Ok(OtxResult::default());
        }

        if status.is_server_error() {
            // Retry once on 5xx
            tracing::debug!(status = %status, url = url, "OTX 5xx, retrying");
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            let retry = self.client
                .get(url)
                .header("X-OTX-API-KEY", &self.api_key)
                .timeout(std::time::Duration::from_secs(5))
                .send()
                .await?;
            if !retry.status().is_success() {
                return Err(IntelError::Http(format!("OTX retry: {}", retry.status())));
            }
            let body: serde_json::Value = retry.json().await?;
            return parse_otx_response(body);
        }

        if !status.is_success() {
            return Err(IntelError::Http(format!("OTX: {status}")));
        }

        let body: serde_json::Value = resp.json().await?;
        parse_otx_response(body)
    }
}

fn parse_otx_response(body: serde_json::Value) -> Result<OtxResult, IntelError> {
    let pulse_info = body.get("pulse_info").cloned().unwrap_or(serde_json::Value::Null);

    let pulse_count = pulse_info
        .get("count")
        .and_then(|v| v.as_i64())
        .unwrap_or(0) as i32;

    let pulse_names = pulse_info
        .get("pulses")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|p| p.get("name").and_then(|n| n.as_str()).map(String::from))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let reputation = body
        .pointer("/general/reputation")
        .or_else(|| body.get("reputation"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0) as i32;

    Ok(OtxResult {
        pulse_count,
        pulse_names,
        reputation,
        raw_json: body,
    })
}
