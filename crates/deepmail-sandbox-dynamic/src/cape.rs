/// CAPEv2 REST API client.

use std::sync::Arc;
use std::time::Duration;

use crate::error::DynamicError;

/// CAPEv2 task status.
#[derive(Debug, Clone, PartialEq)]
pub enum CapeTaskStatus {
    Pending,
    Running,
    Reported,
    Failed,
    Unknown(String),
}

/// Client for CAPEv2 REST API interactions.
#[allow(dead_code)]
pub struct CapeClient {
    client: Arc<reqwest::Client>,
    base_url: String,
    api_token: String,
}

impl CapeClient {
    /// Create a new CAPEv2 client. base_url trailing slash is stripped.
    pub fn new(base_url: String, api_token: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .pool_idle_timeout(Duration::from_secs(60))
            .build()
            .expect("failed to build reqwest client");
        Self {
            client: Arc::new(client),
            base_url: base_url.trim_end_matches('/').to_string(),
            api_token,
        }
    }

    /// Returns true if CAPEv2 is configured (URL and token are non-empty).
    pub fn is_configured(&self) -> bool {
        !self.base_url.is_empty() && !self.api_token.is_empty()
    }

    /// Submit a file to CAPEv2 for dynamic analysis.
    pub async fn submit_file(
        &self,
        filename: &str,
        data: &[u8],
    ) -> Result<u64, DynamicError> {
        let url = format!("{}/apiv2/tasks/create/file/", self.base_url);

        let file_part = reqwest::multipart::Part::bytes(data.to_vec())
            .file_name(filename.to_string())
            .mime_str("application/octet-stream")
            .map_err(|e| DynamicError::Internal(format!("mime: {}", e)))?;

        let options_part = reqwest::multipart::Part::text("analysis_timeout=120");

        let form = reqwest::multipart::Form::new()
            .part("file", file_part)
            .part("options", options_part);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Token {}", self.api_token))
            .multipart(form)
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() || e.is_timeout() {
                    DynamicError::CapeUnavailable
                } else {
                    DynamicError::CapeApiError(0, e.to_string())
                }
            })?;

        let status = response.status().as_u16();
        if status != 200 {
            let body = response.text().await.unwrap_or_default();
            return Err(DynamicError::CapeApiError(status, body));
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| DynamicError::CapeApiError(200, format!("json parse: {}", e)))?;

        // CAPEv2 returns task_id as either integer or string
        let task_id = body
            .pointer("/data/task_id")
            .or_else(|| body.pointer("/data/task_ids/0"))
            .and_then(|v| {
                v.as_u64()
                    .or_else(|| v.as_str().and_then(|s| s.parse::<u64>().ok()))
            })
            .ok_or_else(|| {
                DynamicError::CapeApiError(
                    200,
                    format!("no task_id in response: {}", body),
                )
            })?;

        Ok(task_id)
    }

    /// Poll task status from CAPEv2.
    pub async fn poll_status(&self, task_id: u64) -> Result<CapeTaskStatus, DynamicError> {
        let url = format!("{}/apiv2/tasks/view/{}/", self.base_url, task_id);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Token {}", self.api_token))
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() || e.is_timeout() {
                    DynamicError::CapeUnavailable
                } else {
                    DynamicError::CapeApiError(0, e.to_string())
                }
            })?;

        let status_code = response.status().as_u16();
        if status_code == 404 {
            return Err(DynamicError::TaskNotFound(task_id));
        }
        if status_code != 200 {
            let body = response.text().await.unwrap_or_default();
            return Err(DynamicError::CapeApiError(status_code, body));
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| DynamicError::CapeApiError(200, format!("json: {}", e)))?;

        let status_str = body
            .pointer("/data/status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        Ok(match status_str {
            "reported" => CapeTaskStatus::Reported,
            "pending" => CapeTaskStatus::Pending,
            "running" => CapeTaskStatus::Running,
            "failed_analysis" | "failed_processing" | "failed" => CapeTaskStatus::Failed,
            other => CapeTaskStatus::Unknown(other.to_string()),
        })
    }

    /// Retrieve the full CAPE report for a completed task.
    pub async fn get_report(
        &self,
        task_id: u64,
    ) -> Result<serde_json::Value, DynamicError> {
        let url = format!("{}/apiv2/tasks/report/{}/", self.base_url, task_id);

        // Use a longer timeout for large reports
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .map_err(|e| DynamicError::Internal(e.to_string()))?;

        let response = client
            .get(&url)
            .header("Authorization", format!("Token {}", self.api_token))
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() || e.is_timeout() {
                    DynamicError::CapeUnavailable
                } else {
                    DynamicError::CapeApiError(0, e.to_string())
                }
            })?;

        let status_code = response.status().as_u16();
        if status_code != 200 {
            let body = response.text().await.unwrap_or_default();
            return Err(DynamicError::CapeApiError(status_code, body));
        }

        response
            .json()
            .await
            .map_err(|e| DynamicError::CapeApiError(200, format!("json: {}", e)))
    }

    /// Delete a task from CAPEv2. Fire-and-forget: logs warning on error.
    pub async fn delete_task(&self, task_id: u64) {
        let url = format!("{}/apiv2/tasks/delete/{}/", self.base_url, task_id);

        match self
            .client
            .delete(&url)
            .header("Authorization", format!("Token {}", self.api_token))
            .send()
            .await
        {
            Ok(resp) if !resp.status().is_success() => {
                tracing::warn!(
                    task_id,
                    status = resp.status().as_u16(),
                    "CAPE delete_task non-success"
                );
            }
            Err(e) => {
                tracing::warn!(task_id, error = %e, "CAPE delete_task failed");
            }
            Ok(_) => {
                tracing::debug!(task_id, "CAPE task deleted");
            }
        }
    }
}
