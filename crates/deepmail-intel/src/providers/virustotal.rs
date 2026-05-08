/// VirusTotal API client with rate limiting and 4 entity type lookups.

use std::sync::Arc;
use std::time::{Duration, Instant};

use base64::Engine;
use tokio::sync::Mutex;

use crate::error::IntelError;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VtResult {
    pub malicious: u32,
    pub suspicious: u32,
    pub harmless: u32,
    pub undetected: u32,
    pub total_engines: u32,
    pub reputation: i32,
    pub tags: Vec<String>,
    pub vt_score: f32,
    pub raw_json: serde_json::Value,
}

impl Default for VtResult {
    fn default() -> Self {
        Self {
            malicious: 0, suspicious: 0, harmless: 0, undetected: 0,
            total_engines: 0, reputation: 0, tags: Vec::new(),
            vt_score: 0.0, raw_json: serde_json::Value::Null,
        }
    }
}

struct VtRateLimiter {
    tokens: u32,
    capacity: u32,
    last_refill: Instant,
    refill_interval: Duration,
}

impl VtRateLimiter {
    fn new(capacity: u32) -> Self {
        Self {
            tokens: capacity,
            capacity,
            last_refill: Instant::now(),
            refill_interval: Duration::from_secs(60),
        }
    }

    fn acquire(&mut self) -> Option<Duration> {
        self.refill();
        if self.tokens > 0 {
            self.tokens -= 1;
            None // no wait needed
        } else {
            let elapsed = self.last_refill.elapsed();
            let wait = if elapsed < self.refill_interval {
                self.refill_interval - elapsed
            } else {
                Duration::from_millis(100)
            };
            Some(wait)
        }
    }

    fn refill(&mut self) {
        let elapsed = self.last_refill.elapsed();
        if elapsed >= self.refill_interval {
            let periods = (elapsed.as_secs_f64() / self.refill_interval.as_secs_f64()) as u32;
            self.tokens = (self.tokens + periods * self.capacity).min(self.capacity);
            self.last_refill = Instant::now();
        }
    }
}

pub struct VtClient {
    client: Arc<reqwest::Client>,
    api_key: String,
    base_url: String,
    rate_limiter: Arc<Mutex<VtRateLimiter>>,
}

impl VtClient {
    pub fn new(client: Arc<reqwest::Client>, api_key: String, rate_limit_per_min: u32) -> Self {
        Self {
            client,
            api_key,
            base_url: "https://www.virustotal.com/api/v3".to_string(),
            rate_limiter: Arc::new(Mutex::new(VtRateLimiter::new(rate_limit_per_min))),
        }
    }

    pub fn is_configured(&self) -> bool {
        !self.api_key.is_empty()
    }

    async fn rate_limit(&self) {
        loop {
            let wait = {
                let mut rl = self.rate_limiter.lock().await;
                rl.acquire()
            };
            match wait {
                None => return,
                Some(dur) => tokio::time::sleep(dur).await,
            }
        }
    }

    pub async fn lookup_url(&self, url: &str) -> Result<VtResult, IntelError> {
        if !self.is_configured() {
            return Ok(VtResult::default());
        }
        self.rate_limit().await;

        let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(url);
        let api_url = format!("{}/urls/{encoded}", self.base_url);

        let resp = self.client
            .get(&api_url)
            .header("x-apikey", &self.api_key)
            .send()
            .await?;

        let status = resp.status();

        if status.as_u16() == 404 {
            // Try submitting the URL for analysis
            return self.submit_url(url).await;
        }

        if !status.is_success() {
            return Err(IntelError::Http(format!("VT URL lookup: {status}")));
        }

        let body: serde_json::Value = resp.json().await?;
        parse_vt_response(body)
    }

    async fn submit_url(&self, url: &str) -> Result<VtResult, IntelError> {
        self.rate_limit().await;

        let api_url = format!("{}/urls", self.base_url);
        let resp = self.client
            .post(&api_url)
            .header("x-apikey", &self.api_key)
            .form(&[("url", url)])
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(IntelError::Http(format!(
                "VT URL submit: {}",
                resp.status()
            )));
        }

        let body: serde_json::Value = resp.json().await?;
        let analysis_id = body
            .pointer("/data/id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| IntelError::Parse("missing analysis ID".to_string()))?
            .to_string();

        // Poll for results up to 3 times with 2s sleep
        for attempt in 0..3 {
            tokio::time::sleep(Duration::from_secs(2)).await;
            self.rate_limit().await;

            let poll_url = format!("{}/analyses/{analysis_id}", self.base_url);
            let poll_resp = self.client
                .get(&poll_url)
                .header("x-apikey", &self.api_key)
                .send()
                .await?;

            if !poll_resp.status().is_success() {
                if attempt == 2 {
                    return Err(IntelError::Http(format!(
                        "VT analysis poll: {}",
                        poll_resp.status()
                    )));
                }
                continue;
            }

            let poll_body: serde_json::Value = poll_resp.json().await?;
            let status_val = poll_body
                .pointer("/data/attributes/status")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            if status_val == "completed" {
                return parse_vt_response(poll_body);
            }
        }

        // Return empty result if analysis didn't complete in time
        Ok(VtResult::default())
    }

    pub async fn lookup_hash(&self, hash: &str) -> Result<VtResult, IntelError> {
        if !self.is_configured() {
            return Ok(VtResult::default());
        }
        self.rate_limit().await;

        let api_url = format!("{}/files/{hash}", self.base_url);
        let resp = self.client
            .get(&api_url)
            .header("x-apikey", &self.api_key)
            .send()
            .await?;

        if resp.status().as_u16() == 404 {
            return Ok(VtResult::default());
        }

        if !resp.status().is_success() {
            return Err(IntelError::Http(format!("VT hash lookup: {}", resp.status())));
        }

        let body: serde_json::Value = resp.json().await?;
        parse_vt_response(body)
    }

    pub async fn lookup_ip(&self, ip: &str) -> Result<VtResult, IntelError> {
        if !self.is_configured() {
            return Ok(VtResult::default());
        }
        self.rate_limit().await;

        let api_url = format!("{}/ip_addresses/{ip}", self.base_url);
        let resp = self.client
            .get(&api_url)
            .header("x-apikey", &self.api_key)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(IntelError::Http(format!("VT IP lookup: {}", resp.status())));
        }

        let body: serde_json::Value = resp.json().await?;
        parse_vt_response(body)
    }

    pub async fn lookup_domain(&self, domain: &str) -> Result<VtResult, IntelError> {
        if !self.is_configured() {
            return Ok(VtResult::default());
        }
        self.rate_limit().await;

        let api_url = format!("{}/domains/{domain}", self.base_url);
        let resp = self.client
            .get(&api_url)
            .header("x-apikey", &self.api_key)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(IntelError::Http(format!("VT domain lookup: {}", resp.status())));
        }

        let body: serde_json::Value = resp.json().await?;
        parse_vt_response(body)
    }
}

fn parse_vt_response(json: serde_json::Value) -> Result<VtResult, IntelError> {
    let attrs = json
        .pointer("/data/attributes")
        .cloned()
        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

    let stats = attrs
        .get("last_analysis_stats")
        .cloned()
        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

    let malicious = stats.get("malicious").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    let suspicious = stats.get("suspicious").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    let harmless = stats.get("harmless").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    let undetected = stats.get("undetected").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    let timeout = stats.get("timeout").and_then(|v| v.as_u64()).unwrap_or(0) as u32;

    let total_engines = malicious + suspicious + harmless + undetected + timeout;

    let reputation = attrs.get("reputation").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

    let tags = attrs
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let vt_score = if total_engines > 0 {
        ((malicious as f32) + (suspicious as f32) * 0.5) / (total_engines as f32)
    } else {
        0.0
    }
    .clamp(0.0, 1.0);

    Ok(VtResult {
        malicious,
        suspicious,
        harmless,
        undetected,
        total_engines,
        reputation,
        tags,
        vt_score,
        raw_json: json,
    })
}
