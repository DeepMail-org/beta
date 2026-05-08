/// Threat classification of sandbox-visited pages.

use crate::docker::RawPlaywrightResult;

/// Well-known brand names for impersonation detection.
const BRAND_NAMES: &[&str] = &[
    "google", "microsoft", "paypal", "apple", "amazon",
    "facebook", "instagram", "netflix", "dropbox", "linkedin",
    "github", "stripe", "coinbase", "binance", "chase",
    "wellsfargo", "bankofamerica", "citibank", "hsbc", "barclays",
];

/// Malware file extensions to watch for in network requests.
const MALWARE_EXTENSIONS: &[&str] = &[
    ".exe", ".msi", ".dll", ".bat", ".ps1", ".vbs",
];

/// C2 panel indicator phrases.
const C2_PRIMARY: &[&str] = &[
    "admin panel", "control panel", "bot", "infected", "payload", "backdoor",
];
const C2_SECONDARY: &[&str] = &[
    "c2", "command and control", "rat ",
];

/// Threat classification categories.
#[derive(Debug, Clone, PartialEq)]
pub enum ThreatClass {
    Benign,
    CredentialHarvesting,
    Phishing,
    MalwareDistribution,
    C2Panel,
    Suspicious,
}

impl ThreatClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Benign => "benign",
            Self::CredentialHarvesting => "credential_harvesting",
            Self::Phishing => "phishing",
            Self::MalwareDistribution => "malware_distribution",
            Self::C2Panel => "c2_panel",
            Self::Suspicious => "suspicious",
        }
    }
}

/// Classify a sandboxed page and return (class, score, notes).
pub fn classify_page(
    result: &RawPlaywrightResult,
    original_url: &str,
) -> (ThreatClass, f32, Vec<String>) {
    let mut score: f32 = 0.0;
    let mut notes: Vec<String> = Vec::new();

    // ── Track per-category contributions for class determination ─────────
    let mut credential_score: f32 = 0.0;
    let mut malware_score: f32 = 0.0;
    let mut c2_score: f32 = 0.0;

    // ── Score 1: Credential harvesting signals ──────────────────────────
    if result.has_login_form {
        credential_score += 0.50;
        notes.push("Login form detected".into());
    }
    if result.has_password_field {
        credential_score += 0.20;
        notes.push("Password input field detected".into());
    }
    if result.has_email_field {
        credential_score += 0.15;
        notes.push("Email input field detected".into());
    }
    score += credential_score;

    // ── Score 2: Brand impersonation ────────────────────────────────────
    if let (Some(title), Some(final_url)) = (&result.title, &result.final_url) {
        let title_lower = title.to_lowercase();
        let url_lower = final_url.to_lowercase();

        for brand in BRAND_NAMES {
            if title_lower.contains(brand) && !url_lower.contains(brand) {
                score += 0.40;
                credential_score += 0.40;
                notes.push(format!("Brand impersonation: {}", brand));
                break; // First match only
            }
        }
    }

    // ── Score 3: Malware distribution ───────────────────────────────────
    let has_malware_download = result.network_requests.iter().any(|req| {
        let url_lower = req.url.to_lowercase();
        MALWARE_EXTENSIONS.iter().any(|ext| url_lower.ends_with(ext))
    });
    if has_malware_download {
        malware_score += 0.60;
        notes.push("Malware download detected".into());
    }
    if result.has_download_trigger {
        malware_score += 0.40;
        notes.push("Download trigger detected (Content-Disposition)".into());
    }
    score += malware_score;

    // ── Score 4: C2 panel signals ───────────────────────────────────────
    if let Some(ref html) = result.page_html {
        let html_lower = html.to_lowercase();

        for phrase in C2_PRIMARY {
            if html_lower.contains(phrase) {
                c2_score += 0.30;
                notes.push(format!("C2 indicator: \"{}\"", phrase));
                break; // Count once for primary
            }
        }
        for phrase in C2_SECONDARY {
            if html_lower.contains(phrase) {
                c2_score += 0.20;
                notes.push(format!("C2 indicator: \"{}\"", phrase));
                break; // Count once for secondary
            }
        }
    }
    score += c2_score;

    // ── Score 5: Suspicious signals ─────────────────────────────────────
    if !result.js_dialogs.is_empty() {
        score += 0.15;
        notes.push(format!("JS dialogs detected: {}", result.js_dialogs.len()));
    }
    if result.redirect_chain.len() > 3 {
        score += 0.10;
        notes.push(format!("Excessive redirects: {}", result.redirect_chain.len()));
    }
    // External scripts from IP addresses
    let has_ip_scripts = result.external_scripts.iter().any(|s| {
        // Check if the host part looks like an IP
        url::Url::parse(s)
            .ok()
            .and_then(|u| u.host_str().map(|h| h.parse::<std::net::Ipv4Addr>().is_ok()))
            .unwrap_or(false)
    });
    if has_ip_scripts {
        score += 0.10;
        notes.push("External scripts loaded from IP address".into());
    }
    // Cross-origin iframes
    let has_cross_origin_iframe = if let Some(ref final_url) = result.final_url {
        let final_domain = url::Url::parse(final_url)
            .ok()
            .and_then(|u| u.host_str().map(|h| h.to_string()));
        result.iframes.iter().any(|iframe_src| {
            let iframe_domain = url::Url::parse(iframe_src)
                .ok()
                .and_then(|u| u.host_str().map(|h| h.to_string()));
            match (&final_domain, &iframe_domain) {
                (Some(fd), Some(id)) => fd != id,
                _ => false,
            }
        })
    } else {
        false
    };
    if has_cross_origin_iframe {
        score += 0.10;
        notes.push("Cross-origin iframe detected".into());
    }

    // ── Score 6: Navigation error ───────────────────────────────────────
    if let Some(ref err) = result.error {
        score += 0.10;
        notes.push(format!("Navigation error: {}", err));
    }

    // ── Final classification ────────────────────────────────────────────
    let final_score = score.clamp(0.0, 1.0);

    let threat_class = if malware_score >= 0.60 {
        ThreatClass::MalwareDistribution
    } else if credential_score >= 0.50 {
        ThreatClass::CredentialHarvesting
    } else if c2_score >= 0.30 {
        ThreatClass::C2Panel
    } else if final_score >= 0.50 {
        ThreatClass::Phishing
    } else if final_score >= 0.20 {
        ThreatClass::Suspicious
    } else {
        ThreatClass::Benign
    };

    let _ = original_url; // Used for context; kept for future use
    (threat_class, final_score, notes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::docker::RawPlaywrightResult;

    #[test]
    fn test_benign_page() {
        let result = RawPlaywrightResult {
            final_url: Some("https://example.com".into()),
            title: Some("Example".into()),
            ..Default::default()
        };
        let (class, score, _notes) = classify_page(&result, "https://example.com");
        assert_eq!(class, ThreatClass::Benign);
        assert!(score < 0.20);
    }

    #[test]
    fn test_credential_harvesting() {
        let result = RawPlaywrightResult {
            final_url: Some("https://evil.com/login".into()),
            title: Some("Google Sign In".into()),
            has_login_form: true,
            has_password_field: true,
            has_email_field: true,
            ..Default::default()
        };
        let (class, score, notes) = classify_page(&result, "https://evil.com/login");
        assert_eq!(class, ThreatClass::CredentialHarvesting);
        assert!(score >= 0.50);
        assert!(notes.iter().any(|n| n.contains("Brand impersonation")));
    }

    #[test]
    fn test_malware_distribution() {
        let result = RawPlaywrightResult {
            final_url: Some("https://bad.com/download".into()),
            title: Some("Free Software".into()),
            network_requests: vec![
                crate::docker::NetworkRequest {
                    url: "https://bad.com/payload.exe".into(),
                    method: "GET".into(),
                    status: Some(200),
                    resource_type: "document".into(),
                },
            ],
            has_download_trigger: true,
            ..Default::default()
        };
        let (class, score, _notes) = classify_page(&result, "https://bad.com/download");
        assert_eq!(class, ThreatClass::MalwareDistribution);
        assert!(score >= 0.60);
    }

    #[test]
    fn test_suspicious_redirects() {
        let result = RawPlaywrightResult {
            final_url: Some("https://final.com".into()),
            title: Some("Page".into()),
            redirect_chain: vec![
                "https://r1.com".into(),
                "https://r2.com".into(),
                "https://r3.com".into(),
                "https://r4.com".into(),
            ],
            ..Default::default()
        };
        let (class, score, notes) = classify_page(&result, "https://start.com");
        assert!(score >= 0.10);
        assert!(notes.iter().any(|n| n.contains("redirect")));
        // Score < 0.50, should be Suspicious or Benign depending on exact score
        assert!(class == ThreatClass::Benign || class == ThreatClass::Suspicious);
    }
}
