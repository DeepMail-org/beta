/// Docker container management for URL sandboxing via bollard.

use std::collections::HashMap;
use std::path::Path;

use bollard::container::{
    Config as ContainerConfig, CreateContainerOptions, RemoveContainerOptions,
    StartContainerOptions, WaitContainerOptions,
};
use bollard::models::HostConfig;
use bollard::Docker;
use futures::StreamExt;
use serde::Deserialize;

use crate::config::SandboxUrlConfig;
use crate::error::SandboxUrlError;
use crate::playwright_script::PLAYWRIGHT_SCRIPT;

// ─── Result types parsed from container output ──────────────────────────────

/// Raw result JSON written by the Playwright script.
#[derive(Debug, Clone, Deserialize)]
pub struct RawPlaywrightResult {
    pub final_url: Option<String>,
    pub title: Option<String>,
    #[serde(default)]
    pub redirect_chain: Vec<String>,
    #[serde(default)]
    pub network_requests: Vec<NetworkRequest>,
    #[serde(default)]
    pub cookies: Vec<serde_json::Value>,
    #[serde(default)]
    pub has_password_field: bool,
    #[serde(default)]
    pub has_email_field: bool,
    #[serde(default)]
    pub has_login_form: bool,
    #[serde(default)]
    pub has_download_trigger: bool,
    #[serde(default)]
    pub external_scripts: Vec<String>,
    #[serde(default)]
    pub iframes: Vec<String>,
    pub page_html: Option<String>,
    #[serde(default)]
    pub js_dialogs: Vec<String>,
    pub meta_description: Option<String>,
    pub error: Option<String>,
}

/// A network request captured during page load.
#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct NetworkRequest {
    pub url: String,
    #[serde(default)]
    pub method: String,
    pub status: Option<i32>,
    #[serde(default)]
    pub resource_type: String,
}

impl Default for RawPlaywrightResult {
    fn default() -> Self {
        Self {
            final_url: None,
            title: None,
            redirect_chain: Vec::new(),
            network_requests: Vec::new(),
            cookies: Vec::new(),
            has_password_field: false,
            has_email_field: false,
            has_login_form: false,
            has_download_trigger: false,
            external_scripts: Vec::new(),
            iframes: Vec::new(),
            page_html: None,
            js_dialogs: Vec::new(),
            meta_description: None,
            error: None,
        }
    }
}

// ─── Container operations ───────────────────────────────────────────────────

/// Create and start an analysis container. Returns the container ID.
pub async fn spawn_analysis_container(
    docker: &Docker,
    config: &SandboxUrlConfig,
    url: &str,
    results_dir: &Path,
) -> Result<String, SandboxUrlError> {
    let results_path = results_dir
        .to_str()
        .ok_or_else(|| SandboxUrlError::Docker("invalid results dir path".into()))?;

    let env_vars = vec![format!("URL_TO_ANALYZE={}", url)];
    let binds = vec![format!("{}:/results", results_path)];

    let host_config = HostConfig {
        binds: Some(binds),
        memory: Some(536_870_912),       // 512 MB
        nano_cpus: Some(500_000_000),    // 0.5 CPUs
        pids_limit: Some(100),
        network_mode: Some("bridge".to_string()),
        auto_remove: Some(false),
        ..Default::default()
    };

    let container_config = ContainerConfig {
        image: Some(config.playwright_image.clone()),
        cmd: Some(vec![
            "node".to_string(),
            "-e".to_string(),
            PLAYWRIGHT_SCRIPT.to_string(),
        ]),
        env: Some(env_vars),
        host_config: Some(host_config),
        ..Default::default()
    };

    let create_opts = CreateContainerOptions::<String> {
        name: String::new(),
        platform: None,
    };

    let response = docker
        .create_container(Some(create_opts), container_config)
        .await
        .map_err(|e| SandboxUrlError::Docker(format!("create container: {}", e)))?;

    let container_id = response.id;

    docker
        .start_container(&container_id, None::<StartContainerOptions<String>>)
        .await
        .map_err(|e| SandboxUrlError::Docker(format!("start container: {}", e)))?;

    tracing::info!(container_id = %container_id, url = %url, "sandbox container started");
    Ok(container_id)
}

/// Wait for the container to finish, with a timeout.
/// Returns the exit code on success, or Err(Timeout) if the container
/// exceeds `timeout_secs`.
pub async fn wait_for_container(
    docker: &Docker,
    container_id: &str,
    timeout_secs: u64,
) -> Result<i64, SandboxUrlError> {
    let wait_opts = WaitContainerOptions {
        condition: "not-running".to_string(),
    };

    let mut stream = docker.wait_container(container_id, Some(wait_opts));

    tokio::select! {
        result = async {
            while let Some(item) = stream.next().await {
                match item {
                    Ok(resp) => return Ok(resp.status_code),
                    Err(e) => return Err(SandboxUrlError::Docker(
                        format!("wait container: {}", e)
                    )),
                }
            }
            // Stream ended without result — treat as 0
            Ok(0i64)
        } => result,
        _ = tokio::time::sleep(std::time::Duration::from_secs(timeout_secs)) => {
            tracing::warn!(container_id = %container_id, "container timed out after {}s", timeout_secs);
            // Try to stop the container
            let _ = docker.stop_container(container_id, None).await;
            Err(SandboxUrlError::Timeout(timeout_secs))
        }
    }
}

/// Read the Playwright result JSON from the results directory.
pub fn read_results(results_dir: &Path) -> Result<RawPlaywrightResult, SandboxUrlError> {
    let result_path = results_dir.join("result.json");
    let content = std::fs::read_to_string(&result_path)
        .map_err(|_| SandboxUrlError::NoResults)?;
    let result: RawPlaywrightResult = serde_json::from_str(&content)
        .map_err(|e| SandboxUrlError::Docker(format!("parse result.json: {}", e)))?;
    Ok(result)
}

/// Read the screenshot PNG from the results directory. Returns None if missing.
pub fn read_screenshot(results_dir: &Path) -> Option<Vec<u8>> {
    let path = results_dir.join("screenshot.png");
    std::fs::read(&path).ok()
}

/// Remove a container (force). Logs but never propagates errors.
pub async fn cleanup_container(docker: &Docker, container_id: &str) {
    let opts = RemoveContainerOptions {
        force: true,
        ..Default::default()
    };
    if let Err(e) = docker.remove_container(container_id, Some(opts)).await {
        tracing::warn!(container_id = %container_id, "failed to remove container: {}", e);
    } else {
        tracing::debug!(container_id = %container_id, "container removed");
    }
}
