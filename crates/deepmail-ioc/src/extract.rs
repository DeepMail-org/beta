/// IOC extraction from parsed email data (cross-service queries to parser DB).

use std::collections::HashSet;
use std::net::IpAddr;

use once_cell::sync::Lazy;
use regex::Regex;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::IocError;

/// A raw extracted IOC before normalization.
#[derive(Debug, Clone)]
pub struct RawIoc {
    pub ioc_type: String,
    pub value: String,
    pub raw_value: String,
    pub source: String, // "header", "body", "attachment", "subject", "url"
}

// ─── Regex patterns (compiled once) ────────────────────────────────────────

static RE_IPV4: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b(?:(?:25[0-5]|2[0-4]\d|[01]?\d\d?)\.){3}(?:25[0-5]|2[0-4]\d|[01]?\d\d?)\b")
        .unwrap()
});

static RE_IPV6: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)\b(?:[0-9a-f]{1,4}:){7}[0-9a-f]{1,4}\b|\b(?:[0-9a-f]{1,4}:)*::(?:[0-9a-f]{1,4}:)*[0-9a-f]{1,4}\b"
    ).unwrap()
});

static RE_DOMAIN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)\b(?:[a-z0-9](?:[a-z0-9\-]{0,61}[a-z0-9])?\.)+(?:com|net|org|io|co|uk|de|ru|cn|info|biz|xyz|top|site|online|click|tk|ml|ga|cf|gq|icu|live|shop|app|dev|ai|cc|me|tv|us|ca|au|br|jp|kr|fr|it|es|nl|se|no|fi|pl|cz|ro|hu|bg|hr|sk|si|edu|gov|mil|int|arpa)\b"
    ).unwrap()
});

static RE_URL: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(?:hxxps?|https?)://[^\s"'<>]+"#).unwrap()
});

static RE_EMAIL: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b[a-z0-9._%+\-]+@[a-z0-9.\-]+\.[a-z]{2,}\b").unwrap()
});

static RE_SHA256: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b[0-9a-fA-F]{64}\b").unwrap()
});

static RE_MD5: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b[0-9a-fA-F]{32}\b").unwrap()
});

static RE_SHA1: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b[0-9a-fA-F]{40}\b").unwrap()
});

static RE_CVE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\bCVE-\d{4}-\d{4,7}\b").unwrap()
});

static RE_BITCOIN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b[13][a-km-zA-HJ-NP-Z1-9]{25,34}\b|\bbc1[a-z0-9]{39,59}\b").unwrap()
});

static RE_ETHEREUM: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b0x[0-9a-fA-F]{40}\b").unwrap()
});

/// Hash indicator pattern — MD5/SHA1 only extracted if preceded by these
static RE_HASH_INDICATOR: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(?:md5|sha1|sha256|hash|checksum)\s*[=:]\s*").unwrap()
});

/// Check if an IP is in a private/reserved range.
fn is_private_ip(ip_str: &str) -> bool {
    match ip_str.parse::<IpAddr>() {
        Ok(IpAddr::V4(v4)) => {
            v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || v4.is_broadcast()
                || v4.is_unspecified()
                || v4.octets()[0] == 0
        }
        Ok(IpAddr::V6(v6)) => v6.is_loopback() || v6.is_unspecified(),
        Err(_) => false,
    }
}

/// Check if a hash-like match has a hash indicator nearby (within 20 chars before).
fn has_hash_indicator(text: &str, match_start: usize) -> bool {
    let search_start = match_start.saturating_sub(30);
    let prefix = &text[search_start..match_start];
    RE_HASH_INDICATOR.is_match(prefix)
}

/// Extract IOCs from all text fields.
fn extract_from_text(text: &str, source: &str, iocs: &mut Vec<RawIoc>) {
    // IPv4
    for m in RE_IPV4.find_iter(text) {
        let ip = m.as_str();
        if !is_private_ip(ip) {
            iocs.push(RawIoc {
                ioc_type: "ip".into(),
                value: ip.into(),
                raw_value: ip.into(),
                source: source.into(),
            });
        }
    }

    // IPv6
    for m in RE_IPV6.find_iter(text) {
        let ip = m.as_str();
        if !is_private_ip(ip) {
            iocs.push(RawIoc {
                ioc_type: "ip".into(),
                value: ip.into(),
                raw_value: ip.into(),
                source: source.into(),
            });
        }
    }

    // Domains
    for m in RE_DOMAIN.find_iter(text) {
        let domain = m.as_str();
        // Skip single-label domains (already filtered by regex requiring TLD)
        if domain.contains('.') {
            iocs.push(RawIoc {
                ioc_type: "domain".into(),
                value: domain.into(),
                raw_value: domain.into(),
                source: source.into(),
            });
        }
    }

    // URLs
    for m in RE_URL.find_iter(text) {
        iocs.push(RawIoc {
            ioc_type: "url".into(),
            value: m.as_str().into(),
            raw_value: m.as_str().into(),
            source: source.into(),
        });
    }

    // Email addresses
    for m in RE_EMAIL.find_iter(text) {
        iocs.push(RawIoc {
            ioc_type: "email".into(),
            value: m.as_str().into(),
            raw_value: m.as_str().into(),
            source: source.into(),
        });
    }

    // SHA-256 (always extract)
    for m in RE_SHA256.find_iter(text) {
        iocs.push(RawIoc {
            ioc_type: "hash".into(),
            value: m.as_str().into(),
            raw_value: m.as_str().into(),
            source: source.into(),
        });
    }

    // MD5 — only if near a hash indicator
    for m in RE_MD5.find_iter(text) {
        // Skip if it's actually a SHA-256 or SHA-1 substring
        if m.as_str().len() == 32 && has_hash_indicator(text, m.start()) {
            iocs.push(RawIoc {
                ioc_type: "hash".into(),
                value: m.as_str().into(),
                raw_value: m.as_str().into(),
                source: source.into(),
            });
        }
    }

    // SHA-1 — only if near a hash indicator
    for m in RE_SHA1.find_iter(text) {
        if m.as_str().len() == 40 && has_hash_indicator(text, m.start()) {
            iocs.push(RawIoc {
                ioc_type: "hash".into(),
                value: m.as_str().into(),
                raw_value: m.as_str().into(),
                source: source.into(),
            });
        }
    }

    // CVE
    for m in RE_CVE.find_iter(text) {
        iocs.push(RawIoc {
            ioc_type: "cve".into(),
            value: m.as_str().into(),
            raw_value: m.as_str().into(),
            source: source.into(),
        });
    }

    // Bitcoin
    for m in RE_BITCOIN.find_iter(text) {
        iocs.push(RawIoc {
            ioc_type: "wallet".into(),
            value: m.as_str().into(),
            raw_value: m.as_str().into(),
            source: source.into(),
        });
    }

    // Ethereum
    for m in RE_ETHEREUM.find_iter(text) {
        iocs.push(RawIoc {
            ioc_type: "wallet".into(),
            value: m.as_str().into(),
            raw_value: m.as_str().into(),
            source: source.into(),
        });
    }
}

/// Extract email address from a header value like "Name <user@example.com>".
fn extract_email_from_header(value: &str) -> Option<String> {
    if let Some(start) = value.rfind('<') {
        if let Some(end) = value.rfind('>') {
            if end > start {
                return Some(value[start + 1..end].to_string());
            }
        }
    }
    // Fallback: if it looks like an email directly
    if value.contains('@') && value.contains('.') {
        return Some(value.trim().to_string());
    }
    None
}

/// Extract domain from an email address.
fn domain_from_email(email: &str) -> Option<String> {
    email.rfind('@').map(|pos| email[pos + 1..].to_lowercase())
}

/// Main extraction entry point: fetch from parser DB and extract all IOCs.
pub async fn extract_email_iocs(
    parser_pool: &PgPool,
    email_id: Uuid,
) -> Result<Vec<RawIoc>, IocError> {
    let mut iocs: Vec<RawIoc> = Vec::new();

    // 1. Get parsed_email_id
    let parsed_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM parsed_emails WHERE email_id = $1 LIMIT 1",
    )
    .bind(email_id)
    .fetch_optional(parser_pool)
    .await?;

    let parsed_id = match parsed_id {
        Some(id) => id,
        None => {
            tracing::warn!(%email_id, "no parsed email found for IOC extraction");
            return Ok(iocs);
        }
    };

    // 2. Headers (for IP/domain/email extraction)
    let header_rows: Vec<(String, String)> = sqlx::query_as(
        r#"SELECT name, value FROM email_headers
           WHERE parsed_email_id = $1
             AND name IN ('from','reply-to','return-path','x-originating-ip',
                          'x-sender-ip','cc','to')"#,
    )
    .bind(parsed_id)
    .fetch_all(parser_pool)
    .await
    .unwrap_or_default();

    for (name, value) in &header_rows {
        // Extract emails from from/reply-to/return-path/cc/to headers
        if matches!(name.as_str(), "from" | "reply-to" | "return-path" | "cc" | "to") {
            if let Some(email_addr) = extract_email_from_header(value) {
                iocs.push(RawIoc {
                    ioc_type: "email".into(),
                    value: email_addr.clone(),
                    raw_value: email_addr.clone(),
                    source: "header".into(),
                });
                // Also extract domain from email
                if let Some(domain) = domain_from_email(&email_addr) {
                    if domain.contains('.') {
                        iocs.push(RawIoc {
                            ioc_type: "domain".into(),
                            value: domain.clone(),
                            raw_value: domain,
                            source: "header".into(),
                        });
                    }
                }
            }
        }

        // Extract IPs from x-originating-ip, x-sender-ip
        if matches!(name.as_str(), "x-originating-ip" | "x-sender-ip") {
            let ip_str = value.trim().trim_matches(|c| c == '[' || c == ']');
            if !ip_str.is_empty() && !is_private_ip(ip_str) {
                iocs.push(RawIoc {
                    ioc_type: "ip".into(),
                    value: ip_str.into(),
                    raw_value: value.clone(),
                    source: "header".into(),
                });
            }
        }

        // Extract from all header values using regex
        extract_from_text(value, "header", &mut iocs);
    }

    // 3. Received hops (IPs)
    let hop_rows: Vec<(Option<String>, Option<String>)> = sqlx::query_as(
        r#"SELECT CAST(from_ip AS TEXT), from_host
           FROM received_hops
           WHERE parsed_email_id = $1
             AND from_ip IS NOT NULL"#,
    )
    .bind(parsed_id)
    .fetch_all(parser_pool)
    .await
    .unwrap_or_default();

    for (from_ip, from_host) in &hop_rows {
        if let Some(ip) = from_ip {
            let ip = ip.trim();
            if !ip.is_empty() && !is_private_ip(ip) {
                iocs.push(RawIoc {
                    ioc_type: "ip".into(),
                    value: ip.into(),
                    raw_value: ip.into(),
                    source: "header".into(),
                });
            }
        }
        if let Some(host) = from_host {
            let host = host.trim();
            if !host.is_empty() && host.contains('.') {
                iocs.push(RawIoc {
                    ioc_type: "domain".into(),
                    value: host.into(),
                    raw_value: host.into(),
                    source: "header".into(),
                });
            }
        }
    }

    // 4. Attachments (hashes)
    let attach_rows: Vec<(Option<String>, Option<String>, Option<String>)> = sqlx::query_as(
        r#"SELECT sha256_hash, md5_hash, filename
           FROM attachments
           WHERE parsed_email_id = $1"#,
    )
    .bind(parsed_id)
    .fetch_all(parser_pool)
    .await
    .unwrap_or_default();

    for (sha256, md5, _filename) in &attach_rows {
        if let Some(h) = sha256 {
            let h = h.trim();
            if !h.is_empty() {
                iocs.push(RawIoc {
                    ioc_type: "hash".into(),
                    value: h.into(),
                    raw_value: h.into(),
                    source: "attachment".into(),
                });
            }
        }
        if let Some(h) = md5 {
            let h = h.trim();
            if !h.is_empty() {
                iocs.push(RawIoc {
                    ioc_type: "hash".into(),
                    value: h.into(),
                    raw_value: h.into(),
                    source: "attachment".into(),
                });
            }
        }
    }

    // 5. Subject + body text for regex extraction
    let body_row: Option<(Option<String>, Option<String>, Option<String>)> = sqlx::query_as(
        r#"SELECT subject, body_text, body_html
           FROM parsed_emails WHERE email_id = $1"#,
    )
    .bind(email_id)
    .fetch_optional(parser_pool)
    .await?;

    if let Some((subject, body_text, body_html)) = body_row {
        if let Some(subj) = &subject {
            if !subj.is_empty() {
                extract_from_text(subj, "subject", &mut iocs);
            }
        }
        if let Some(bt) = &body_text {
            if !bt.is_empty() {
                extract_from_text(bt, "body", &mut iocs);
            }
        }
        if let Some(bh) = &body_html {
            if !bh.is_empty() {
                extract_from_text(bh, "body", &mut iocs);
            }
        }
    }

    tracing::info!(%email_id, raw_ioc_count = iocs.len(), "IOC extraction complete");
    Ok(iocs)
}
