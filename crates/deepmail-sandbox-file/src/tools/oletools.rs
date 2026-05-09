/// oletools — OLE/VBA macro analysis.

use std::path::Path;

use crate::error::SandboxFileError;
use super::run_tool_with_timeout;

#[derive(Debug, Clone, Default)]
pub struct OleResult {
    pub has_macros: bool,
    pub macro_count: i32,
    pub has_vba: bool,
    pub has_autoexec: bool,
    pub suspicious_keywords: Vec<String>,
    pub iocs: Vec<String>,
    pub is_suspicious: bool,
}

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct OleIdResult {
    pub has_macros: bool,
    pub is_encrypted: bool,
    pub has_flash: bool,
}

/// Run olevba to analyze VBA macros.
pub async fn run_olevba(path: &Path, timeout: u64) -> Result<OleResult, SandboxFileError> {
    let path_str = path.to_string_lossy().to_string();

    let result = match run_tool_with_timeout(
        "olevba", &["--json", &path_str], None, timeout,
    ).await {
        Ok(r) => r,
        Err(SandboxFileError::ToolNotFound(_)) => {
            tracing::warn!("olevba not available, skipping");
            return Ok(OleResult::default());
        }
        Err(e) => return Err(e),
    };

    if result.timed_out {
        return Ok(OleResult::default());
    }

    // Try JSON parsing first
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&result.stdout) {
        return Ok(parse_olevba_json(&json));
    }

    // Fall back to text parsing
    Ok(parse_olevba_text(&result.stdout))
}

fn parse_olevba_json(json: &serde_json::Value) -> OleResult {
    let mut res = OleResult::default();

    if let Some(macros) = json.get("macros").and_then(|m| m.as_array()) {
        res.macro_count = macros.len() as i32;
        res.has_macros = !macros.is_empty();
        res.has_vba = !macros.is_empty();
    }

    if let Some(analysis) = json.get("analysis") {
        if let Some(suspicious) = analysis.get("suspicious").and_then(|s| s.as_array()) {
            for item in suspicious {
                if let Some(s) = item.as_str() {
                    res.suspicious_keywords.push(s.to_string());
                } else if let Some(kw) = item.get("keyword").and_then(|k| k.as_str()) {
                    res.suspicious_keywords.push(kw.to_string());
                }
            }
        }
        if let Some(iocs) = analysis.get("iocs").and_then(|i| i.as_array()) {
            for ioc in iocs {
                if let Some(s) = ioc.as_str() {
                    res.iocs.push(s.to_string());
                }
            }
        }
        if let Some(autoexec) = analysis.get("autoexec").and_then(|a| a.as_array()) {
            res.has_autoexec = !autoexec.is_empty();
        }
    }

    res.is_suspicious = !res.suspicious_keywords.is_empty() || res.has_autoexec;
    res
}

fn parse_olevba_text(stdout: &str) -> OleResult {
    let mut res = OleResult::default();
    let lower = stdout.to_lowercase();

    res.has_macros = lower.contains("vba macro") || lower.contains("has macro");
    res.has_vba = res.has_macros;

    res.has_autoexec = lower.contains("autoopen")
        || lower.contains("autoexec")
        || lower.contains("document_open");

    for line in stdout.lines() {
        let line_lower = line.to_lowercase();
        if line_lower.contains("suspicious") || line_lower.contains("high risk") {
            res.suspicious_keywords.push(line.trim().to_string());
        }
    }

    // Cap suspicious keywords
    res.suspicious_keywords.truncate(20);
    res.is_suspicious = !res.suspicious_keywords.is_empty() || res.has_autoexec;

    if res.has_macros {
        res.macro_count = 1; // At least one macro
    }

    res
}

/// Run oleid to detect OLE indicators.
pub async fn run_oleid(path: &Path, timeout: u64) -> Result<OleIdResult, SandboxFileError> {
    let path_str = path.to_string_lossy().to_string();

    let result = match run_tool_with_timeout(
        "oleid", &[&path_str], None, timeout,
    ).await {
        Ok(r) => r,
        Err(SandboxFileError::ToolNotFound(_)) => {
            tracing::warn!("oleid not available, skipping");
            return Ok(OleIdResult::default());
        }
        Err(e) => return Err(e),
    };

    if result.timed_out {
        return Ok(OleIdResult::default());
    }

    let lower = result.stdout.to_lowercase();

    Ok(OleIdResult {
        has_macros: lower.contains("macros: true")
            || lower.contains("vba macros: yes")
            || lower.contains("macros              | true"),
        is_encrypted: lower.contains("encrypted: yes")
            || lower.contains("encrypted: true")
            || lower.contains("encrypted           | true"),
        has_flash: lower.contains("flash: true")
            || lower.contains("flash objects: yes"),
    })
}
