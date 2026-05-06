//! DKIM header-level check.
//!
//! This module checks for the presence and content of the
//! Authentication-Results and DKIM-Signature headers.
//! Full cryptographic DKIM verification lives in deepmail-dkim;
//! this module checks header-level indicators only.

/// DKIM check result parsed from Authentication-Results or DKIM-Signature header.
#[derive(Debug, Clone)]
pub struct DkimCheckResult {
    pub result: DkimVerdict,
    pub domain: Option<String>,
    pub selector: Option<String>,
}

/// Outcome of the header-level DKIM check.
#[derive(Debug, Clone, PartialEq)]
pub enum DkimVerdict {
    Pass,
    Fail,
    None,
    Neutral,
    Policy,
    TempError,
    PermError,
}

impl DkimVerdict {
    pub fn as_str(&self) -> &'static str {
        match self {
            DkimVerdict::Pass      => "pass",
            DkimVerdict::Fail      => "fail",
            DkimVerdict::None      => "none",
            DkimVerdict::Neutral   => "neutral",
            DkimVerdict::Policy    => "policy",
            DkimVerdict::TempError => "temperror",
            DkimVerdict::PermError => "permerror",
        }
    }
}

/// Check DKIM result from Authentication-Results header content.
///
/// Parses a string like: `dkim=pass header.d=example.com header.s=selector`
pub fn check_authentication_results(auth_results: &str) -> DkimCheckResult {
    let lower = auth_results.to_lowercase();

    // Find "dkim=" in the Authentication-Results header
    let result_str = if let Some(idx) = lower.find("dkim=") {
        let rest = &lower[idx + 5..];
        // Extract result token (up to whitespace or semicolon)
        rest.split(|c: char| c.is_whitespace() || c == ';')
            .next()
            .unwrap_or("")
    } else {
        return DkimCheckResult {
            result: DkimVerdict::None,
            domain: None,
            selector: None,
        };
    };

    let verdict = match result_str {
        "pass"      => DkimVerdict::Pass,
        "fail"      => DkimVerdict::Fail,
        "neutral"   => DkimVerdict::Neutral,
        "policy"    => DkimVerdict::Policy,
        "temperror" => DkimVerdict::TempError,
        "permerror" => DkimVerdict::PermError,
        _           => DkimVerdict::None,
    };

    let domain = extract_kv(&lower, "header.d=");
    let selector = extract_kv(&lower, "header.s=");

    DkimCheckResult {
        result: verdict,
        domain,
        selector,
    }
}

/// Parse a raw DKIM-Signature header to extract d= (domain) and s= (selector).
///
/// DKIM-Signature has key=value pairs separated by semicolons.
pub fn parse_dkim_signature(dkim_sig: &str) -> DkimCheckResult {
    let lower = dkim_sig.to_lowercase();

    let domain = extract_tag_value(&lower, "d");
    let selector = extract_tag_value(&lower, "s");

    DkimCheckResult {
        result: if domain.is_some() { DkimVerdict::None } else { DkimVerdict::PermError },
        domain,
        selector,
    }
}

/// Extract "key=value" from a whitespace/semicolon-delimited string.
fn extract_kv(haystack: &str, key: &str) -> Option<String> {
    let idx = haystack.find(key)?;
    let rest = &haystack[idx + key.len()..];
    let token: &str = rest
        .split(|c: char| c.is_whitespace() || c == ';')
        .next()
        .unwrap_or("");
    if token.is_empty() {
        None
    } else {
        Some(token.to_string())
    }
}

/// Extract a DKIM-Signature tag value (e.g. "d=example.com").
fn extract_tag_value(header: &str, tag: &str) -> Option<String> {
    // Look for the tag followed by '=' in a semicolon-delimited list
    for part in header.split(';') {
        let trimmed = part.trim();
        let prefix = format!("{tag}=");
        if let Some(rest) = trimmed.strip_prefix(&prefix) {
            let val = rest.trim().trim_end_matches(';');
            if val.is_empty() {
                return None;
            }
            return Some(val.to_string());
        }
    }
    None
}
