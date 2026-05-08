/// URL normalization, shortener detection, base64 URL extraction, domain extraction.

use std::collections::HashSet;

use base64::Engine;
use once_cell::sync::Lazy;
use regex::Regex;
use url::Url;

/// Known link shortener domains.
pub static SHORTENER_DOMAINS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let mut s = HashSet::new();
    for d in &[
        "bit.ly", "tinyurl.com", "t.co", "goo.gl", "ow.ly", "short.io",
        "rb.gy", "is.gd", "buff.ly", "dlvr.it", "ift.tt", "tiny.cc",
        "lnkd.in", "cutt.ly", "shorturl.at", "bl.ink", "snip.ly",
        "clck.ru", "vk.cc", "2ch.cc", "u.to", "x.co", "qr.ae",
        "sui.li", "shrtco.de", "0rz.tw", "href.li", "adf.ly", "bc.vc", "j.mp",
    ] {
        s.insert(*d);
    }
    s
});

/// Known tracking pixel / beacon domains.
pub static TRACKING_DOMAINS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let mut s = HashSet::new();
    for d in &[
        "mailchimp.com", "sendgrid.net", "constantcontact.com", "hubspot.com",
        "marketo.com", "salesforce.com", "mailtrack.io", "streak.com",
        "getnotify.com", "readnotify.com", "bananatag.com", "yesware.com",
        "mixmax.com", "boomeranggmail.com", "cirrusinsight.com", "contact.io",
        "spytrack.com", "openedornot.com", "myemailverifier.com",
    ] {
        s.insert(*d);
    }
    s
});

/// Extracted URL with metadata.
#[derive(Debug, Clone)]
pub struct ExtractedUrl {
    pub raw_url: String,
    pub normalized_url: String,
    pub url_type: String,
    pub is_shortened: bool,
    pub shortener_domain: Option<String>,
    pub is_external: bool,
    pub destination_domain: Option<String>,
    pub is_suspicious: bool,
}

/// Normalize a URL: defang, parse, remove fragment.
pub fn normalize_url(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Defanging
    let mut s = trimmed.to_string();
    s = s.replace("hxxp://", "http://");
    s = s.replace("hxxps://", "https://");
    s = s.replace("[.]", ".");
    s = s.replace("[at]", "@");

    let mut parsed = Url::parse(&s).ok()?;
    parsed.set_fragment(None);
    Some(parsed.as_str().to_string())
}

/// Extract domain from a URL string.
pub fn extract_domain(url_str: &str) -> Option<String> {
    Url::parse(url_str)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_lowercase()))
}

/// Check if a URL uses a known link shortener.
pub fn is_shortened(url_str: &str) -> bool {
    if let Some(domain) = extract_domain(url_str) {
        return SHORTENER_DOMAINS.contains(domain.as_str())
            || SHORTENER_DOMAINS.iter().any(|sd| domain.ends_with(&format!(".{}", sd)));
    }
    false
}

/// Get the shortener domain if URL is shortened.
pub fn get_shortener_domain(url_str: &str) -> Option<String> {
    let domain = extract_domain(url_str)?;
    if SHORTENER_DOMAINS.contains(domain.as_str()) {
        return Some(domain);
    }
    for sd in SHORTENER_DOMAINS.iter() {
        if domain.ends_with(&format!(".{}", sd)) {
            return Some(sd.to_string());
        }
    }
    None
}

/// Check if a URL is external relative to the sender domain.
pub fn is_external(url_str: &str, sender_domain: &str) -> bool {
    if sender_domain.is_empty() {
        return true;
    }
    match extract_domain(url_str) {
        Some(domain) => {
            let sender_lower = sender_domain.to_lowercase();
            !(domain == sender_lower || domain.ends_with(&format!(".{}", sender_lower)))
        }
        None => true,
    }
}

/// Regex for base64 strings in text.
static RE_BASE64: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[A-Za-z0-9+/]{40,}={0,2}").unwrap()
});

/// Extract URLs hidden in base64-encoded strings.
pub fn extract_base64_urls(text: &str) -> Vec<String> {
    let engine = base64::engine::general_purpose::STANDARD;
    let mut urls = Vec::new();

    for cap in RE_BASE64.find_iter(text) {
        if let Ok(decoded_bytes) = engine.decode(cap.as_str()) {
            if let Ok(decoded) = String::from_utf8(decoded_bytes) {
                if decoded.starts_with("http://") || decoded.starts_with("https://") {
                    urls.push(decoded);
                } else if decoded.contains("href=") || decoded.contains("<a ") {
                    // Extract URLs from decoded HTML fragment
                    let re = Regex::new(r#"https?://[^\s"'<>]+"#).unwrap();
                    for m in re.find_iter(&decoded) {
                        urls.push(m.as_str().to_string());
                    }
                }
            }
        }
    }

    urls
}

/// Regex for suspicious URL path patterns.
static RE_SUSPICIOUS_PATH: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)/(secure|login|verify|account|update|confirm|banking|paypal|apple|microsoft|google|amazon)/").unwrap()
});

/// Check if a URL has suspicious indicators.
pub fn is_suspicious_url(url_str: &str) -> bool {
    let parsed = match Url::parse(url_str) {
        Ok(u) => u,
        Err(_) => return false,
    };

    // IP address as host
    if let Some(host) = parsed.host_str() {
        if host.parse::<std::net::IpAddr>().is_ok() {
            return true;
        }

        // Excessive subdomains (> 4 dots)
        if host.chars().filter(|c| *c == '.').count() > 4 {
            return true;
        }

        // Very long random subdomain
        let labels: Vec<&str> = host.split('.').collect();
        if labels.len() > 2 {
            let subdomain = labels[0];
            if subdomain.len() > 30 {
                // Check for random-looking: few vowels relative to length
                let vowels = subdomain.chars().filter(|c| "aeiou".contains(*c)).count();
                if (vowels as f32) < (subdomain.len() as f32 * 0.15) {
                    return true;
                }
            }
        }
    }

    // Suspicious path patterns
    if RE_SUSPICIOUS_PATH.is_match(parsed.path()) {
        return true;
    }

    false
}

/// Classify a raw URL into a full ExtractedUrl.
pub fn classify_url(raw: &str, url_type: &str, sender_domain: &str) -> ExtractedUrl {
    let normalized = normalize_url(raw).unwrap_or_else(|| raw.to_string());
    let domain = extract_domain(&normalized);
    let shortened = is_shortened(&normalized);
    let shortener = if shortened { get_shortener_domain(&normalized) } else { None };
    let external = is_external(&normalized, sender_domain);
    let suspicious = is_suspicious_url(&normalized) || shortened;

    ExtractedUrl {
        raw_url: raw.to_string(),
        normalized_url: normalized,
        url_type: url_type.to_string(),
        is_shortened: shortened,
        shortener_domain: shortener,
        is_external: external,
        destination_domain: domain,
        is_suspicious: suspicious,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_url() {
        assert_eq!(
            normalize_url("hxxps://evil[.]com/path#frag"),
            Some("https://evil.com/path".to_string())
        );
    }

    #[test]
    fn test_shortener() {
        assert!(is_shortened("https://bit.ly/abc123"));
        assert!(!is_shortened("https://google.com/"));
    }

    #[test]
    fn test_external() {
        assert!(is_external("https://evil.com/", "google.com"));
        assert!(!is_external("https://mail.google.com/", "google.com"));
    }

    #[test]
    fn test_suspicious_ip() {
        assert!(is_suspicious_url("http://192.168.1.1/login"));
    }
}
