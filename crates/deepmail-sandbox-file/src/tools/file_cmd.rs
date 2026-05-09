/// `file` command — MIME type and magic detection.

use std::path::Path;

use crate::error::SandboxFileError;
use super::run_tool_with_timeout;

#[derive(Debug, Clone, Default)]
pub struct FileCmdResult {
    pub mime_type: String,
    pub magic: String,
}

/// Run `file` command to detect MIME type and magic.
pub async fn run_file_command(path: &Path, timeout: u64) -> Result<FileCmdResult, SandboxFileError> {
    let path_str = path.to_string_lossy().to_string();

    // Get MIME type
    let mime_result = match run_tool_with_timeout(
        "file", &["--brief", "--mime-type", &path_str], None, timeout,
    ).await {
        Ok(r) => r.stdout.trim().to_string(),
        Err(SandboxFileError::ToolNotFound(_)) => {
            return Ok(FileCmdResult { mime_type: "unknown".into(), magic: "unknown".into() });
        }
        Err(e) => return Err(e),
    };

    // Get human-readable magic
    let magic_result = match run_tool_with_timeout(
        "file", &["--brief", &path_str], None, timeout,
    ).await {
        Ok(r) => r.stdout.trim().to_string(),
        Err(_) => "unknown".to_string(),
    };

    Ok(FileCmdResult {
        mime_type: mime_result,
        magic: magic_result,
    })
}
