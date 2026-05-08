/// HTTP client for the external deepmail-ml DistilBERT classifier.

use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::error::BodyError;

/// Client wrapper for the ML classification service.
pub struct MlClient {
    client: Arc<reqwest::Client>,
    base_url: String,
}

#[derive(Serialize)]
struct ClassifyRequest {
    text: String,
}

#[derive(Deserialize)]
struct ClassifyResponse {
    score: f32,
    #[allow(dead_code)]
    label: String,
}

impl MlClient {
    /// Create a new ML client with the given base URL.
    pub fn new(base_url: String, timeout_secs: u64) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .pool_idle_timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        Self {
            client: Arc::new(client),
            base_url,
        }
    }

    /// Classify text for phishing using the ML service.
    ///
    /// Returns Ok(score 0.0–1.0) on success.
    /// Returns Err(MlUnavailable) on any failure (connection, timeout, non-200).
    pub async fn classify_phishing(&self, text: &str) -> Result<f32, BodyError> {
        // Truncate to 2000 chars for ML model
        let truncated: String = text.chars().take(2000).collect();

        let url = format!("{}/classify/phishing", self.base_url);

        let resp = self
            .client
            .post(&url)
            .json(&ClassifyRequest { text: truncated })
            .send()
            .await
            .map_err(|e| BodyError::MlUnavailable(format!("request failed: {e}")))?;

        if !resp.status().is_success() {
            return Err(BodyError::MlUnavailable(format!(
                "ML service returned status {}",
                resp.status()
            )));
        }

        let body: ClassifyResponse = resp
            .json()
            .await
            .map_err(|e| BodyError::MlUnavailable(format!("invalid response: {e}")))?;

        Ok(body.score.clamp(0.0, 1.0))
    }
}
