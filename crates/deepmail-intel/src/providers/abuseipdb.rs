/// AbuseIPDB API client.

use std::sync::Arc;

use crate::error::IntelError;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AbuseResult {
    pub abuse_score: i32,
    pub country_code: String,
    pub isp: String,
    pub total_reports: i32,
    pub last_reported_at: Option<String>,
    pub raw_json: serde_json::Value,
}

impl Default for AbuseResult {
    fn default() -> Self {
        Self {
            abuse_score: 0,
            country_code: String::new(),
            isp: String::new(),
            total_reports: 0,
            last_reported_at: None,
            raw_json: serde_json::Value::Null,
        }
    }
}

pub struct AbuseIpDbClient {
    client: Arc<reqwest::Client>,
    api_key: String,
}

impl AbuseIpDbClient {
    pub fn new(client: Arc<reqwest::Client>, api_key: String) -> Self {
        Self { client, api_key }
    }

    pub fn is_configured(&self) -> bool {
        !self.api_key.is_empty()
    }

    pub async fn lookup_ip(&self, ip: &str) -> Result<AbuseResult, IntelError> {
        if !self.is_configured() {
            return Ok(AbuseResult::default());
        }

        let resp = self.client
            .get("https://api.abuseipdb.com/api/v2/check")
            .header("Key", &self.api_key)
            .header("Accept", "application/json")
            .query(&[
                ("ipAddress", ip),
                ("maxAgeInDays", "90"),
                ("verbose", "false"),
            ])
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(IntelError::Http(format!(
                "AbuseIPDB: {}",
                resp.status()
            )));
        }

        let body: serde_json::Value = resp.json().await?;
        let data = body.get("data").cloned().unwrap_or(serde_json::Value::Null);

        let abuse_score = data
            .get("abuseConfidenceScore")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32;
        let country_code = data
            .get("countryCode")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let isp = data
            .get("isp")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let total_reports = data
            .get("totalReports")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32;
        let last_reported_at = data
            .get("lastReportedAt")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Ok(AbuseResult {
            abuse_score,
            country_code,
            isp,
            total_reports,
            last_reported_at,
            raw_json: body,
        })
    }
}
