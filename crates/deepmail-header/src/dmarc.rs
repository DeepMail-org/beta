//! DMARC record lookup and alignment evaluation.
//!
//! Implements DMARC policy parsing and domain alignment checks
//! against SPF and DKIM results.

use crate::dns::Resolver;

/// DMARC evaluation result.
#[derive(Debug, Clone, PartialEq)]
pub enum DmarcResult {
    Pass,
    Fail,
    None,
    TempError,
    PermError,
}

impl DmarcResult {
    pub fn as_str(&self) -> &'static str {
        match self {
            DmarcResult::Pass      => "pass",
            DmarcResult::Fail      => "fail",
            DmarcResult::None      => "none",
            DmarcResult::TempError => "temperror",
            DmarcResult::PermError => "permerror",
        }
    }
}

/// DMARC policy details parsed from a _dmarc TXT record.
#[derive(Debug, Clone)]
pub struct DmarcPolicy {
    pub version: String,
    pub policy: String,
    pub subdomain_policy: Option<String>,
    pub pct: u8,
    pub alignment_spf: AlignmentMode,
    pub alignment_dkim: AlignmentMode,
    pub raw_record: String,
}

/// DMARC alignment mode.
#[derive(Debug, Clone, PartialEq)]
pub enum AlignmentMode {
    Relaxed,
    Strict,
}

/// Evaluate DMARC for a From domain using SPF and DKIM results.
///
/// Arguments:
/// - `resolver`: DNS resolver
/// - `from_domain`: the domain in the From: header
/// - `spf_passed`: whether SPF passed
/// - `spf_domain`: the domain evaluated by SPF (envelope sender)
/// - `dkim_passed`: whether DKIM passed
/// - `dkim_domain`: the d= domain from the DKIM signature
pub async fn evaluate(
    resolver: &Resolver,
    from_domain: &str,
    spf_passed: bool,
    spf_domain: &str,
    dkim_passed: bool,
    dkim_domain: &str,
) -> (DmarcResult, Option<DmarcPolicy>) {
    // Step 1: Look up _dmarc.<from_domain> TXT record
    let query_name = format!("_dmarc.{from_domain}");
    let dns_result = resolver.lookup_txt(&query_name).await;

    if dns_result.nxdomain || dns_result.answers.is_empty() {
        return (DmarcResult::None, None);
    }

    // Step 2: Find the DMARC record
    let raw = match dns_result
        .answers
        .iter()
        .find(|a| a.to_lowercase().starts_with("v=dmarc1"))
    {
        Some(r) => r.clone(),
        None => return (DmarcResult::None, None),
    };

    // Step 3: Parse DMARC tags
    let policy = parse_dmarc_record(&raw);

    // Step 4: Check alignment
    let spf_aligned = if spf_passed {
        domain_aligned(from_domain, spf_domain, &policy.alignment_spf)
    } else {
        false
    };

    let dkim_aligned = if dkim_passed {
        domain_aligned(from_domain, dkim_domain, &policy.alignment_dkim)
    } else {
        false
    };

    let result = if spf_aligned || dkim_aligned {
        DmarcResult::Pass
    } else {
        DmarcResult::Fail
    };

    (result, Some(policy))
}

/// Parse a DMARC TXT record into structured policy fields.
fn parse_dmarc_record(raw: &str) -> DmarcPolicy {
    let tags = parse_tags(raw);

    let version = tags.get("v").cloned().unwrap_or_default();
    let policy = tags.get("p").cloned().unwrap_or_else(|| "none".to_string());
    let subdomain_policy = tags.get("sp").cloned();
    let pct: u8 = tags
        .get("pct")
        .and_then(|v| v.parse().ok())
        .unwrap_or(100);
    let alignment_spf = match tags.get("aspf").map(String::as_str) {
        Some("s") => AlignmentMode::Strict,
        _         => AlignmentMode::Relaxed,
    };
    let alignment_dkim = match tags.get("adkim").map(String::as_str) {
        Some("s") => AlignmentMode::Strict,
        _         => AlignmentMode::Relaxed,
    };

    DmarcPolicy {
        version,
        policy,
        subdomain_policy,
        pct,
        alignment_spf,
        alignment_dkim,
        raw_record: raw.to_string(),
    }
}

/// Parse semicolon-delimited "key=value" tags from a DMARC record.
fn parse_tags(raw: &str) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    for part in raw.split(';') {
        let trimmed = part.trim();
        if let Some((k, v)) = trimmed.split_once('=') {
            map.insert(k.trim().to_lowercase(), v.trim().to_string());
        }
    }
    map
}

/// Check if `from_domain` aligns with `auth_domain` per the given mode.
///
/// - **Relaxed**: auth_domain must equal or be a subdomain of from_domain
///   (i.e. they share the organizational domain).
/// - **Strict**: exact match.
fn domain_aligned(from_domain: &str, auth_domain: &str, mode: &AlignmentMode) -> bool {
    let from_lower = from_domain.to_lowercase();
    let auth_lower = auth_domain.to_lowercase();

    match mode {
        AlignmentMode::Strict => from_lower == auth_lower,
        AlignmentMode::Relaxed => {
            if from_lower == auth_lower {
                return true;
            }
            // Relaxed: match organizational domain (base domain).
            // Simplified: check if one is a subdomain of the other.
            let from_org = organizational_domain(&from_lower);
            let auth_org = organizational_domain(&auth_lower);
            from_org == auth_org
        }
    }
}

/// Extract organizational domain (simplistic: last two labels).
///
/// This is a best-effort approach; a full implementation would use
/// the Public Suffix List.
fn organizational_domain(domain: &str) -> String {
    let labels: Vec<&str> = domain.rsplitn(3, '.').collect();
    if labels.len() >= 2 {
        format!("{}.{}", labels[1], labels[0])
    } else {
        domain.to_string()
    }
}
