/// binwalk — embedded file detection.

use std::path::Path;

use once_cell::sync::Lazy;
use regex::Regex;

use crate::error::SandboxFileError;
use super::run_tool_with_timeout;

static RE_BINWALK_LINE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(\d+)\s+0x[0-9A-Fa-f]+\s+(.+)$").unwrap());

#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct EmbeddedFile {
    pub offset: u64,
    pub description: String,
}

#[derive(Debug, Clone, Default)]
pub struct BinwalkResult {
    pub embedded_files: Vec<EmbeddedFile>,
    pub has_embedded: bool,
}

/// Run binwalk to detect embedded files.
pub async fn run_binwalk(path: &Path, timeout: u64) -> Result<BinwalkResult, SandboxFileError> {
    let path_str = path.to_string_lossy().to_string();

    let result = match run_tool_with_timeout(
        "binwalk", &[&path_str], None, timeout,
    ).await {
        Ok(r) => r,
        Err(SandboxFileError::ToolNotFound(_)) => {
            tracing::warn!("binwalk not available, skipping");
            return Ok(BinwalkResult::default());
        }
        Err(e) => return Err(e),
    };

    let mut embedded_files = Vec::new();

    for line in result.stdout.lines() {
        if embedded_files.len() >= 50 {
            break;
        }
        if let Some(caps) = RE_BINWALK_LINE.captures(line) {
            let offset: u64 = caps[1].parse().unwrap_or(0);
            let description = caps[2].trim().to_string();
            embedded_files.push(EmbeddedFile { offset, description });
        }
    }

    let has_embedded = !embedded_files.is_empty();
    Ok(BinwalkResult {
        embedded_files,
        has_embedded,
    })
}
