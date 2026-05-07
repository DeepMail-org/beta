use chrono::{DateTime, TimeZone, Utc};

use crate::error::DkimError;

#[derive(Debug, Clone)]
pub struct DkimSignature {
    pub version: u8,
    pub algorithm: String,
    pub canonicalization: String,
    pub domain: String,
    pub selector: String,
    pub signed_headers: Vec<String>,
    pub body_hash: String,
    pub signature_data: String,
    pub timestamp: Option<DateTime<Utc>>,
    pub expiration: Option<DateTime<Utc>>,
    pub body_length: Option<usize>,
    pub raw_header: String,
}

pub fn parse_dkim_signature_header(raw: &str) -> Result<DkimSignature, DkimError> {
    let tags = parse_tags(raw)?;

    let version: u8 = tags
        .get("v")
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);

    let algorithm = tags
        .get("a")
        .cloned()
        .unwrap_or_else(|| "rsa-sha256".to_string());

    let canonicalization = tags
        .get("c")
        .cloned()
        .unwrap_or_else(|| "simple/simple".to_string());

    let domain = tags
        .get("d")
        .cloned()
        .ok_or_else(|| DkimError::Parse("missing d= tag".to_string()))?;

    let selector = tags
        .get("s")
        .cloned()
        .ok_or_else(|| DkimError::Parse("missing s= tag".to_string()))?;

    let signed_headers_str = tags
        .get("h")
        .cloned()
        .ok_or_else(|| DkimError::Parse("missing h= tag".to_string()))?;

    let signed_headers: Vec<String> = signed_headers_str
        .split(':')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();

    let body_hash = tags
        .get("bh")
        .cloned()
        .ok_or_else(|| DkimError::Parse("missing bh= tag".to_string()))?;

    let signature_data = tags
        .get("b")
        .cloned()
        .ok_or_else(|| DkimError::Parse("missing b= tag".to_string()))?;

    let timestamp = tags.get("t").and_then(|t| {
        t.trim().parse::<i64>().ok().and_then(|ts| Utc.timestamp_opt(ts, 0).single())
    });

    let expiration = tags.get("x").and_then(|x| {
        x.trim().parse::<i64>().ok().and_then(|ts| Utc.timestamp_opt(ts, 0).single())
    });

    let body_length = tags.get("l").and_then(|l| l.trim().parse::<usize>().ok());

    Ok(DkimSignature {
        version,
        algorithm,
        canonicalization,
        domain,
        selector,
        signed_headers,
        body_hash,
        signature_data,
        timestamp,
        expiration,
        body_length,
        raw_header: raw.to_string(),
    })
}

fn parse_tags(header: &str) -> Result<std::collections::HashMap<String, String>, DkimError> {
    let mut tags = std::collections::HashMap::new();

    let unfolded = header
        .lines()
        .map(|l| l.trim())
        .collect::<Vec<_>>()
        .join(" ");

    for part in unfolded.split(';') {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(eq_pos) = trimmed.find('=') {
            let key = trimmed[..eq_pos].trim().to_lowercase();
            let value = trimmed[eq_pos + 1..].trim().to_string();
            let value_no_ws: String = value.chars().filter(|c| !c.is_whitespace()).collect();
            tags.insert(key, value_no_ws);
        }
    }

    Ok(tags)
}

pub fn extract_dkim_headers_from_raw(raw_email: &[u8]) -> Vec<String> {
    let email_str = String::from_utf8_lossy(raw_email);
    let mut headers = Vec::new();
    let mut current_header = String::new();
    let mut in_dkim = false;

    for line in email_str.lines() {
        if line.is_empty() {
            if in_dkim && !current_header.is_empty() {
                headers.push(current_header.clone());
                current_header.clear();
                in_dkim = false;
            }
            break;
        }

        if line.starts_with("DKIM-Signature:") || line.starts_with("dkim-signature:") {
            if in_dkim && !current_header.is_empty() {
                headers.push(current_header.clone());
            }
            current_header = line.strip_prefix("DKIM-Signature:")
                .or_else(|| line.strip_prefix("dkim-signature:"))
                .unwrap_or("")
                .to_string();
            in_dkim = true;
        } else if in_dkim && (line.starts_with(' ') || line.starts_with('\t')) {
            current_header.push(' ');
            current_header.push_str(line.trim());
        } else {
            if in_dkim && !current_header.is_empty() {
                headers.push(current_header.clone());
                current_header.clear();
            }
            in_dkim = false;
        }
    }

    if in_dkim && !current_header.is_empty() {
        headers.push(current_header);
    }

    headers
}
