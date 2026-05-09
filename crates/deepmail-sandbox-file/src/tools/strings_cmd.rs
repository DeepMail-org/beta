/// `strings` command — printable string extraction.

use std::path::Path;

use once_cell::sync::Lazy;
use regex::Regex;

use crate::error::SandboxFileError;
use super::run_tool_with_timeout;

static RE_URL: Lazy<Regex> = Lazy::new(|| Regex::new(r"https?://[^\s]{10,}").unwrap());
static RE_IP: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b").unwrap());
static RE_REGKEY: Lazy<Regex> = Lazy::new(|| Regex::new(r"HKEY_[A-Z_]+\\").unwrap());
static RE_SYSPATH: Lazy<Regex> = Lazy::new(|| Regex::new(r"(C:\\Windows|C:\\System|/etc/|/tmp/)").unwrap());
static RE_SUSPICIOUS_FUNC: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(VirtualAlloc|WriteProcessMemory|CreateRemoteThread|LoadLibrary|GetProcAddress|WinExec|ShellExecute|URLDownloadToFile|RegSetValue)").unwrap()
});

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct StringsResult {
    pub all_strings: Vec<String>,
    pub suspicious: Vec<String>,
    pub total_count: usize,
}

/// Run `strings -n 6` and filter suspicious patterns.
pub async fn run_strings(path: &Path, timeout: u64) -> Result<StringsResult, SandboxFileError> {
    let path_str = path.to_string_lossy().to_string();

    let result = match run_tool_with_timeout(
        "strings", &["-n", "6", &path_str], None, timeout,
    ).await {
        Ok(r) => r,
        Err(SandboxFileError::ToolNotFound(_)) => {
            tracing::warn!("strings not available, skipping");
            return Ok(StringsResult::default());
        }
        Err(e) => return Err(e),
    };

    let lines: Vec<String> = result
        .stdout
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.to_string())
        .collect();

    let total_count = lines.len();

    // Cap all_strings at 5000
    let all_strings: Vec<String> = lines.iter().take(5000).cloned().collect();

    // Find suspicious strings
    let mut suspicious = Vec::new();
    for line in &lines {
        if suspicious.len() >= 50 {
            break;
        }
        if RE_URL.is_match(line)
            || RE_IP.is_match(line)
            || RE_REGKEY.is_match(line)
            || RE_SYSPATH.is_match(line)
            || RE_SUSPICIOUS_FUNC.is_match(line)
        {
            suspicious.push(line.clone());
        }
    }

    Ok(StringsResult {
        all_strings,
        suspicious,
        total_count,
    })
}
