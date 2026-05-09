/// Tool subprocess execution framework.

pub mod binwalk;
pub mod exiftool;
pub mod file_cmd;
pub mod oletools;
pub mod pdfid;
pub mod pefile;
pub mod strings_cmd;
pub mod yara_scan;

use std::time::Instant;

use crate::error::SandboxFileError;

/// Result from a subprocess tool execution.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ToolResult {
    pub tool_name: String,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub timed_out: bool,
    pub duration_ms: u64,
}

/// Run a tool with timeout. Returns ToolResult or error.
///
/// If the tool binary is not found (which::which fails), returns
/// `Err(SandboxFileError::ToolNotFound(...))`. The caller should
/// handle this by returning a default result, never failing the pipeline.
pub async fn run_tool_with_timeout(
    cmd: &str,
    args: &[&str],
    stdin_data: Option<&[u8]>,
    timeout_secs: u64,
) -> Result<ToolResult, SandboxFileError> {
    // Check binary availability
    if which::which(cmd).is_err() {
        return Err(SandboxFileError::ToolNotFound(cmd.to_string()));
    }

    let start = Instant::now();

    let mut command = tokio::process::Command::new(cmd);
    command
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    if stdin_data.is_some() {
        command.stdin(std::process::Stdio::piped());
    } else {
        command.stdin(std::process::Stdio::null());
    }

    let mut child = command
        .spawn()
        .map_err(|e| SandboxFileError::ToolExec(format!("{}: spawn: {}", cmd, e)))?;

    // Write stdin if provided
    if let Some(data) = stdin_data {
        if let Some(ref mut stdin) = child.stdin {
            use tokio::io::AsyncWriteExt;
            let _ = stdin.write_all(data).await;
            let _ = stdin.shutdown().await;
        }
        // Drop stdin handle to signal EOF
        drop(child.stdin.take());
    }

    // Wait with timeout
    let timeout = tokio::time::Duration::from_secs(timeout_secs);
    match tokio::time::timeout(timeout, child.wait_with_output()).await {
        Ok(Ok(output)) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            Ok(ToolResult {
                tool_name: cmd.to_string(),
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                exit_code: output.status.code().unwrap_or(-1),
                timed_out: false,
                duration_ms,
            })
        }
        Ok(Err(e)) => Err(SandboxFileError::ToolExec(format!("{}: {}", cmd, e))),
        Err(_) => {
            // Timeout — process has been consumed by wait_with_output;
            // it is already being cleaned up.
            let duration_ms = start.elapsed().as_millis() as u64;
            Ok(ToolResult {
                tool_name: cmd.to_string(),
                stdout: String::new(),
                stderr: format!("killed after {}s timeout", timeout_secs),
                exit_code: -1,
                timed_out: true,
                duration_ms,
            })
        }
    }
}
