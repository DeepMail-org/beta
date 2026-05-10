/// CAPEv2 report parser — extract structured findings from raw JSON.

use once_cell::sync::Lazy;
use regex::Regex;

/// Parsed HTTP request from CAPE report.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[allow(dead_code)]
pub struct HttpRequest {
    pub uri: String,
    pub method: String,
    pub status: Option<i32>,
}

/// Dropped file from dynamic analysis.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[allow(dead_code)]
pub struct DroppedFile {
    pub name: String,
    pub sha256: String,
    pub file_type: String,
}

/// CAPE behavioral signature.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[allow(dead_code)]
pub struct CapeSignature {
    pub name: String,
    pub description: String,
    pub severity: i32,
    pub families: Vec<String>,
}

/// Aggregated findings from dynamic analysis.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct DynamicFindings {
    pub malscore: f32,
    pub network_hosts: Vec<String>,
    pub dns_requests: Vec<String>,
    pub http_requests: Vec<HttpRequest>,
    pub smtp_activity: bool,
    pub processes_spawned: Vec<String>,
    pub files_dropped: Vec<DroppedFile>,
    pub registry_modifications: Vec<String>,
    pub persistence_indicators: Vec<String>,
    pub c2_indicators: Vec<String>,
    pub cape_signatures: Vec<CapeSignature>,
    pub cape_unavailable: bool,
}

static C2_PATH_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"/(gate|panel|tasks|upload|bot|check.?in|report|cmd|config)\.php")
        .expect("c2 regex")
});

static C2_PATH_KEYWORDS: &[&str] = &[
    "/gate", "/panel", "/tasks", "/upload", "/report",
];

static PERSIST_API_PATTERNS: &[&str] = &[
    "CreateServiceW",
    "CreateServiceA",
    "SetWindowsHookEx",
];

static RUN_KEY_PATH: &str =
    r"Software\Microsoft\Windows\CurrentVersion\Run";
static STARTUP_FOLDER: &str = r"Startup";

/// Parse a raw CAPEv2 report JSON into structured DynamicFindings.
pub fn parse_cape_report(report: &serde_json::Value) -> DynamicFindings {
    let mut findings = DynamicFindings::default();

    // ── malscore ────────────────────────────────────────────────────────
    findings.malscore = report
        .get("malscore")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0) as f32
        / 10.0;

    // ── network.hosts ──────────────────────────────────────────────────
    if let Some(hosts) = report.pointer("/network/hosts").and_then(|v| v.as_array()) {
        let mut seen = std::collections::HashSet::new();
        for h in hosts.iter().take(100) {
            if let Some(ip) = h.get("ip").and_then(|v| v.as_str()) {
                if seen.insert(ip.to_string()) {
                    findings.network_hosts.push(ip.to_string());
                }
            }
        }
    }

    // ── network.dns ────────────────────────────────────────────────────
    if let Some(dns) = report.pointer("/network/dns").and_then(|v| v.as_array()) {
        let mut seen = std::collections::HashSet::new();
        for d in dns.iter().take(200) {
            if let Some(req) = d.get("request").and_then(|v| v.as_str()) {
                if seen.insert(req.to_string()) {
                    findings.dns_requests.push(req.to_string());
                }
            }
        }
    }

    // ── network.http ───────────────────────────────────────────────────
    if let Some(http) = report.pointer("/network/http").and_then(|v| v.as_array()) {
        for h in http.iter().take(100) {
            findings.http_requests.push(HttpRequest {
                uri: h.get("uri").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                method: h.get("method").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                status: h.get("status").and_then(|v| v.as_i64()).map(|s| s as i32),
            });
        }
    }

    // ── network.smtp ───────────────────────────────────────────────────
    findings.smtp_activity = report
        .pointer("/network/smtp")
        .and_then(|v| v.as_array())
        .map(|a| !a.is_empty())
        .unwrap_or(false);

    // ── behavior.processes ─────────────────────────────────────────────
    if let Some(procs) = report.pointer("/behavior/processes").and_then(|v| v.as_array()) {
        let mut proc_names = std::collections::HashSet::new();

        for proc in procs.iter().take(50) {
            if let Some(name) = proc.get("process_name").and_then(|v| v.as_str()) {
                if proc_names.insert(name.to_lowercase()) {
                    findings.processes_spawned.push(name.to_string());
                }
            }

            // ── registry_modifications & persistence from calls ────────
            if let Some(calls) = proc.get("calls").and_then(|v| v.as_array()) {
                let mut reg_seen = std::collections::HashSet::new();
                for call in calls {
                    let api = call.get("api").and_then(|v| v.as_str()).unwrap_or("");

                    // Registry modifications
                    if api.contains("RegSetValue") || api.contains("RegCreateKey") {
                        if let Some(args) = call.get("arguments").and_then(|v| v.as_array()) {
                            for arg in args {
                                let arg_name = arg.get("name").and_then(|v| v.as_str()).unwrap_or("");
                                if arg_name == "reg_key" || arg_name == "regkey" || arg_name == "FullName" {
                                    if let Some(val) = arg.get("value").and_then(|v| v.as_str()) {
                                        if reg_seen.insert(val.to_string()) && findings.registry_modifications.len() < 100 {
                                            findings.registry_modifications.push(val.to_string());
                                        }
                                        // Check for RunKey persistence
                                        if val.contains(RUN_KEY_PATH) {
                                            let desc = format!("RunKey persistence via {} → {}", api, val);
                                            if !findings.persistence_indicators.contains(&desc) {
                                                findings.persistence_indicators.push(desc);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Persistence: service creation, hooks
                    for pattern in PERSIST_API_PATTERNS {
                        if api.contains(pattern) {
                            let desc = format!("{} detected", pattern);
                            if !findings.persistence_indicators.contains(&desc) {
                                findings.persistence_indicators.push(desc);
                            }
                        }
                    }

                    // Persistence: copy to Startup folder
                    if api.contains("CopyFile") || api.contains("MoveFile") {
                        if let Some(args) = call.get("arguments").and_then(|v| v.as_array()) {
                            for arg in args {
                                if let Some(val) = arg.get("value").and_then(|v| v.as_str()) {
                                    if val.contains(STARTUP_FOLDER) {
                                        let desc = format!("File copied to Startup folder via {}", api);
                                        if !findings.persistence_indicators.contains(&desc) {
                                            findings.persistence_indicators.push(desc);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // ── dropped files ──────────────────────────────────────────────────
    if let Some(dropped) = report.get("dropped").and_then(|v| v.as_array()) {
        for d in dropped.iter().take(50) {
            findings.files_dropped.push(DroppedFile {
                name: d.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                sha256: d.get("sha256").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                file_type: d.get("type").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            });
        }
    }

    // ── cape_signatures ────────────────────────────────────────────────
    if let Some(sigs) = report.get("signatures").and_then(|v| v.as_array()) {
        for sig in sigs.iter().take(50) {
            let name = sig.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let desc = sig.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let severity = sig.get("severity").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let families = sig
                .get("families")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            findings.cape_signatures.push(CapeSignature {
                name,
                description: desc,
                severity,
                families,
            });
        }
    }

    // ── c2_indicators ──────────────────────────────────────────────────
    let mut c2_set = std::collections::HashSet::new();

    // From HTTP requests
    for req in &findings.http_requests {
        if C2_PATH_RE.is_match(&req.uri)
            || C2_PATH_KEYWORDS.iter().any(|kw| req.uri.contains(kw))
        {
            if c2_set.insert(req.uri.clone()) && findings.c2_indicators.len() < 20 {
                findings.c2_indicators.push(format!("HTTP C2: {}", req.uri));
            }
        }
    }

    // From signatures
    for sig in &findings.cape_signatures {
        let name_lower = sig.name.to_lowercase();
        if name_lower.contains("network")
            || name_lower.contains("c2")
            || name_lower.contains("backdoor")
        {
            let label = format!("Signature: {}", sig.name);
            if c2_set.insert(label.clone()) && findings.c2_indicators.len() < 20 {
                findings.c2_indicators.push(label);
            }
        }
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_report() {
        let report = serde_json::json!({});
        let findings = parse_cape_report(&report);
        assert_eq!(findings.malscore, 0.0);
        assert!(findings.network_hosts.is_empty());
        assert!(findings.cape_signatures.is_empty());
    }

    #[test]
    fn test_malscore_normalization() {
        let report = serde_json::json!({ "malscore": 7.5 });
        let findings = parse_cape_report(&report);
        assert!((findings.malscore - 0.75).abs() < 0.01);
    }

    #[test]
    fn test_network_parsing() {
        let report = serde_json::json!({
            "network": {
                "hosts": [
                    {"ip": "10.0.0.1", "country_name": "US"},
                    {"ip": "10.0.0.2"},
                    {"ip": "10.0.0.1"}  // duplicate
                ],
                "dns": [
                    {"request": "evil.com"},
                    {"request": "bad.org"}
                ],
                "http": [
                    {"uri": "http://evil.com/gate.php", "method": "POST", "status": 200}
                ],
                "smtp": [{"raw": "mail data"}]
            }
        });
        let findings = parse_cape_report(&report);
        assert_eq!(findings.network_hosts.len(), 2); // deduped
        assert_eq!(findings.dns_requests.len(), 2);
        assert!(findings.smtp_activity);
        assert!(!findings.c2_indicators.is_empty()); // gate.php
    }

    #[test]
    fn test_signatures_and_dropped() {
        let report = serde_json::json!({
            "signatures": [
                {"name": "network_c2", "description": "C2 detected", "severity": 3, "families": ["emotet"]},
                {"name": "file_ops", "description": "File created", "severity": 1}
            ],
            "dropped": [
                {"name": "payload.exe", "sha256": "abc123", "type": "PE32"}
            ]
        });
        let findings = parse_cape_report(&report);
        assert_eq!(findings.cape_signatures.len(), 2);
        assert_eq!(findings.cape_signatures[0].severity, 3);
        assert_eq!(findings.files_dropped.len(), 1);
        assert!(!findings.c2_indicators.is_empty()); // network_c2 signature
    }
}
