/// IOC normalization, defanging, and deduplication.

use std::collections::HashMap;
use std::net::IpAddr;

use crate::extract::RawIoc;

/// Defang common obfuscation patterns.
pub fn defang(value: &str) -> String {
    let mut s = value.to_string();
    s = s.replace("[.]", ".");
    s = s.replace("(.)",".");
    s = s.replace("[:]", ":");
    // Case-insensitive [at] → @
    let lower = s.to_lowercase();
    if lower.contains("[at]") {
        let mut result = String::with_capacity(s.len());
        let mut i = 0;
        let bytes = s.as_bytes();
        while i < bytes.len() {
            if i + 4 <= bytes.len() && s[i..i+4].eq_ignore_ascii_case("[at]") {
                result.push('@');
                i += 4;
            } else {
                result.push(bytes[i] as char);
                i += 1;
            }
        }
        s = result;
    }
    // hxxp:// → http://  hxxps:// → https://
    if s.starts_with("hxxp://") {
        s = format!("http://{}", &s[7..]);
    } else if s.starts_with("hxxps://") {
        s = format!("https://{}", &s[8..]);
    } else if s.starts_with("hXXp://") || s.starts_with("HXXP://") {
        s = format!("http://{}", &s[7..]);
    } else if s.starts_with("hXXps://") || s.starts_with("HXXPS://") {
        s = format!("https://{}", &s[8..]);
    }
    s
}

/// Normalize a domain value.
pub fn normalize_domain(domain: &str) -> String {
    let s = defang(domain);
    let s = s.to_lowercase();
    let s = s.trim_end_matches('.').to_string();
    // Strip leading wildcard
    if s.starts_with("*.") {
        s[2..].to_string()
    } else {
        s
    }
}

/// Normalize a URL: defang, parse, sort query params, remove fragment.
pub fn normalize_url(raw_url: &str) -> String {
    let defanged = defang(raw_url);
    match url::Url::parse(&defanged) {
        Ok(mut parsed) => {
            // Remove fragment
            parsed.set_fragment(None);
            // Sort query params
            let has_query = parsed.query().is_some();
            if has_query {
                let mut params: Vec<(String, String)> = parsed
                    .query_pairs()
                    .map(|(k, v)| (k.into_owned(), v.into_owned()))
                    .collect();
                params.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
                let sorted: Vec<String> = params
                    .iter()
                    .map(|(k, v)| {
                        if v.is_empty() {
                            k.clone()
                        } else {
                            format!("{k}={v}")
                        }
                    })
                    .collect();
                if sorted.is_empty() {
                    parsed.set_query(None);
                } else {
                    parsed.set_query(Some(&sorted.join("&")));
                }
            }
            // Strip trailing slash if path is "/"
            let result = parsed.to_string();
            result
        }
        Err(_) => defanged,
    }
}

/// Normalize an IP address. Returns None if unparseable.
pub fn normalize_ip(ip: &str) -> Option<String> {
    let defanged = defang(ip);
    let trimmed = defanged.trim();
    trimmed.parse::<IpAddr>().ok().map(|addr| addr.to_string())
}

/// Normalize an email address.
pub fn normalize_email_addr(addr: &str) -> String {
    let s = defang(addr);
    s.to_lowercase().trim().to_string()
}

/// Normalize a hash value.
pub fn normalize_hash(hash: &str) -> String {
    hash.trim().to_lowercase()
}

/// Normalize any IOC by type. Returns None if normalization fails.
pub fn normalize_ioc(ioc_type: &str, value: &str) -> Option<String> {
    match ioc_type {
        "ip" => normalize_ip(value),
        "domain" => {
            let d = normalize_domain(value);
            if d.is_empty() { None } else { Some(d) }
        }
        "url" => {
            let u = normalize_url(value);
            if u.is_empty() { None } else { Some(u) }
        }
        "hash" => {
            let h = normalize_hash(value);
            if h.is_empty() { None } else { Some(h) }
        }
        "email" => {
            let e = normalize_email_addr(value);
            if e.is_empty() { None } else { Some(e) }
        }
        "cve" | "wallet" | "mutex" | "registry" => {
            let t = value.trim().to_string();
            if t.is_empty() { None } else { Some(t) }
        }
        _ => None,
    }
}

/// Deduplicate IOCs: group by (ioc_type, normalized_value), prefer "header" source.
pub fn deduplicate(iocs: Vec<RawIoc>) -> Vec<RawIoc> {
    let mut best: HashMap<(String, String), RawIoc> = HashMap::new();

    for ioc in iocs {
        let normalized = match normalize_ioc(&ioc.ioc_type, &ioc.value) {
            Some(n) => n,
            None => continue,
        };

        let key = (ioc.ioc_type.clone(), normalized.clone());
        let entry = best.entry(key);
        match entry {
            std::collections::hash_map::Entry::Vacant(v) => {
                v.insert(RawIoc {
                    ioc_type: ioc.ioc_type,
                    value: normalized,
                    raw_value: ioc.raw_value,
                    source: ioc.source,
                });
            }
            std::collections::hash_map::Entry::Occupied(mut o) => {
                // Prefer header source over body
                if ioc.source == "header" && o.get().source != "header" {
                    o.insert(RawIoc {
                        ioc_type: ioc.ioc_type,
                        value: normalized,
                        raw_value: ioc.raw_value,
                        source: ioc.source,
                    });
                }
            }
        }
    }

    best.into_values().collect()
}
