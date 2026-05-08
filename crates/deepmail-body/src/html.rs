/// HTML parsing: text extraction, URL extraction, obfuscation detection,
/// tracking pixel detection.

use once_cell::sync::Lazy;
use regex::Regex;
use scraper::{Html, Selector};

use crate::urls::{self, ExtractedUrl, TRACKING_DOMAINS};

// ─── Selectors (compiled once) ─────────────────────────────────────────────

static SEL_A: Lazy<Selector> = Lazy::new(|| Selector::parse("a[href]").unwrap());
static SEL_IMG: Lazy<Selector> = Lazy::new(|| Selector::parse("img[src]").unwrap());
static SEL_FORM: Lazy<Selector> = Lazy::new(|| Selector::parse("form[action]").unwrap());
static SEL_META_REFRESH: Lazy<Selector> = Lazy::new(|| {
    Selector::parse("meta[http-equiv='refresh']").unwrap()
});
static SEL_LINK: Lazy<Selector> = Lazy::new(|| Selector::parse("link[href]").unwrap());
static SEL_SCRIPT: Lazy<Selector> = Lazy::new(|| Selector::parse("script[src]").unwrap());
static SEL_STYLE_ATTR: Lazy<Selector> = Lazy::new(|| Selector::parse("[style]").unwrap());

// ─── Regex patterns ───────────────────────────────────────────────────────

static RE_CSS_URL: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"url\(['"]?([^'")\s]+)['"]?\)"#).unwrap()
});
static RE_META_URL: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)url\s*=\s*['"]?([^'";\s]+)"#).unwrap()
});
static RE_DISPLAY_NONE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)display\s*:\s*none").unwrap()
});
static RE_FONT_ZERO: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)font-size\s*:\s*0").unwrap()
});
static RE_WHITE_TEXT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)color\s*:\s*(#fff|#ffffff|white|rgb\(\s*255\s*,\s*255\s*,\s*255\s*\))").unwrap()
});
static RE_VISIBILITY_HIDDEN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)visibility\s*:\s*hidden").unwrap()
});
static RE_OPACITY_ZERO: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)opacity\s*:\s*0([^.0-9]|$)").unwrap()
});
static RE_OFFSCREEN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)position\s*:\s*absolute.*left\s*:\s*-\d{4,}").unwrap()
});
static RE_COMMENT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"<!--[\s\S]*?-->").unwrap()
});

/// Zero-width Unicode characters to detect.
const ZERO_WIDTH_CHARS: &[char] = &[
    '\u{200B}', // zero width space
    '\u{200C}', // zero width non-joiner
    '\u{200D}', // zero width joiner
    '\u{FEFF}', // byte order mark / zero-width no-break space
];

// ─── Plain text extraction ────────────────────────────────────────────────

/// Extract plain text from HTML, excluding script/style content.
pub fn extract_plain_text(html: &str) -> String {
    // First, remove script/style tags and their content via regex
    let re_script = Regex::new(r"(?is)<script[^>]*>.*?</script>").unwrap();
    let re_style = Regex::new(r"(?is)<style[^>]*>.*?</style>").unwrap();
    let re_noscript = Regex::new(r"(?is)<noscript[^>]*>.*?</noscript>").unwrap();

    let cleaned = re_script.replace_all(html, "");
    let cleaned = re_style.replace_all(&cleaned, "");
    let cleaned = re_noscript.replace_all(&cleaned, "");

    let document = Html::parse_document(&cleaned);

    let mut text_parts: Vec<String> = Vec::new();
    for text_node in document.root_element().text() {
        text_parts.push(text_node.to_string());
    }

    let combined = text_parts.join(" ");
    // Collapse whitespace
    let re_ws = Regex::new(r"\s+").unwrap();
    re_ws.replace_all(&combined, " ").trim().to_string()
}

// ─── URL extraction ───────────────────────────────────────────────────────

/// Extract all URLs from HTML with their types.
pub fn extract_urls_from_html(html: &str, sender_domain: &str) -> Vec<ExtractedUrl> {
    let document = Html::parse_document(html);
    let mut raw_urls: Vec<(String, String)> = Vec::new(); // (url, type)

    // <a href="...">
    for el in document.select(&SEL_A) {
        if let Some(href) = el.value().attr("href") {
            if is_valid_url_candidate(href) {
                let url_type = if href.starts_with("data:") { "data_uri" } else { "href" };
                raw_urls.push((href.to_string(), url_type.to_string()));
            }
        }
    }

    // <img src="...">
    for el in document.select(&SEL_IMG) {
        if let Some(src) = el.value().attr("src") {
            if is_valid_url_candidate(src) {
                let url_type = if src.starts_with("data:") { "data_uri" } else { "src" };
                raw_urls.push((src.to_string(), url_type.to_string()));
            }
        }
    }

    // <form action="...">
    for el in document.select(&SEL_FORM) {
        if let Some(action) = el.value().attr("action") {
            if is_valid_url_candidate(action) {
                raw_urls.push((action.to_string(), "action".to_string()));
            }
        }
    }

    // <meta http-equiv="refresh" content="...url=...">
    for el in document.select(&SEL_META_REFRESH) {
        if let Some(content) = el.value().attr("content") {
            if let Some(cap) = RE_META_URL.captures(content) {
                if let Some(m) = cap.get(1) {
                    raw_urls.push((m.as_str().to_string(), "meta_refresh".to_string()));
                }
            }
        }
    }

    // <link href="...">
    for el in document.select(&SEL_LINK) {
        if let Some(href) = el.value().attr("href") {
            if is_valid_url_candidate(href) {
                raw_urls.push((href.to_string(), "src".to_string()));
            }
        }
    }

    // <script src="...">
    for el in document.select(&SEL_SCRIPT) {
        if let Some(src) = el.value().attr("src") {
            if is_valid_url_candidate(src) {
                raw_urls.push((src.to_string(), "src".to_string()));
            }
        }
    }

    // style="...url('...')..."
    for el in document.select(&SEL_STYLE_ATTR) {
        if let Some(style) = el.value().attr("style") {
            for cap in RE_CSS_URL.captures_iter(style) {
                if let Some(m) = cap.get(1) {
                    raw_urls.push((m.as_str().to_string(), "css_url".to_string()));
                }
            }
        }
    }

    // Classify all extracted URLs
    raw_urls
        .into_iter()
        .map(|(raw, url_type)| urls::classify_url(&raw, &url_type, sender_domain))
        .collect()
}

/// Check if a candidate string looks like a URL worth extracting.
fn is_valid_url_candidate(s: &str) -> bool {
    let trimmed = s.trim();
    if trimmed.is_empty() || trimmed == "#" || trimmed == "/" {
        return false;
    }
    // Must start with a scheme or be absolute/relative path
    trimmed.starts_with("http://")
        || trimmed.starts_with("https://")
        || trimmed.starts_with("data:")
        || trimmed.starts_with("//")
        || trimmed.starts_with("hxxp")
}

// ─── Obfuscation detection ───────────────────────────────────────────────

/// Report of HTML obfuscation techniques found.
#[derive(Debug, Clone)]
pub struct ObfuscationReport {
    pub has_obfuscation: bool,
    pub techniques: Vec<String>,
}

/// Detect HTML obfuscation techniques.
pub fn detect_obfuscation(html: &str) -> ObfuscationReport {
    let mut techniques: Vec<String> = Vec::new();

    // Scan style attributes
    let document = Html::parse_document(html);
    for el in document.select(&SEL_STYLE_ATTR) {
        if let Some(style) = el.value().attr("style") {
            if RE_DISPLAY_NONE.is_match(style) && !techniques.contains(&"display_none".to_string()) {
                techniques.push("display_none".to_string());
            }
            if RE_FONT_ZERO.is_match(style) && !techniques.contains(&"zero_font_size".to_string()) {
                techniques.push("zero_font_size".to_string());
            }
            if RE_WHITE_TEXT.is_match(style) && !techniques.contains(&"white_text".to_string()) {
                techniques.push("white_text".to_string());
            }
            if RE_VISIBILITY_HIDDEN.is_match(style) && !techniques.contains(&"visibility_hidden".to_string()) {
                techniques.push("visibility_hidden".to_string());
            }
            if RE_OPACITY_ZERO.is_match(style) && !techniques.contains(&"opacity_zero".to_string()) {
                techniques.push("opacity_zero".to_string());
            }
            if RE_OFFSCREEN.is_match(style) && !techniques.contains(&"offscreen_content".to_string()) {
                techniques.push("offscreen_content".to_string());
            }
        }
    }

    // Excessive HTML comments
    let comment_count = RE_COMMENT.find_iter(html).count();
    if comment_count > 5 {
        techniques.push("excessive_comments".to_string());
    }

    // Zero-width characters
    let plain_text = extract_plain_text(html);
    if plain_text.chars().any(|c| ZERO_WIDTH_CHARS.contains(&c)) {
        techniques.push("zero_width_characters".to_string());
    }

    ObfuscationReport {
        has_obfuscation: !techniques.is_empty(),
        techniques,
    }
}

// ─── Tracking pixel detection ─────────────────────────────────────────────

/// A tracking pixel / beacon hit.
#[derive(Debug, Clone)]
pub struct TrackingHit {
    pub url: String,
    pub hit_type: String,
    pub domain: String,
}

/// Common tracking URL patterns.
static RE_TRACKING_PARAMS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(\?|&)(open|track|pixel)=|/track/open|/o\.gif|/pixel\.gif").unwrap()
});

/// Detect tracking pixels and beacons in HTML.
pub fn detect_tracking_pixels(html: &str) -> Vec<TrackingHit> {
    let document = Html::parse_document(html);
    let mut hits: Vec<TrackingHit> = Vec::new();

    // 1x1 pixel images
    for el in document.select(&SEL_IMG) {
        let src = el.value().attr("src").unwrap_or_default();
        if src.is_empty() || src.starts_with("data:") {
            continue;
        }

        let w = el.value().attr("width").and_then(|v| v.parse::<u32>().ok());
        let h = el.value().attr("height").and_then(|v| v.parse::<u32>().ok());

        let is_1x1 = matches!((w, h), (Some(1), Some(1)) | (Some(0), Some(0)));

        // Check style for width:1px
        let style = el.value().attr("style").unwrap_or_default();
        let style_1px = style.contains("width:1px") || style.contains("width: 1px");

        if is_1x1 || style_1px {
            if let Some(domain) = urls::extract_domain(src) {
                hits.push(TrackingHit {
                    url: src.to_string(),
                    hit_type: "tracking_pixel".to_string(),
                    domain,
                });
            }
        }

        // Check domain against tracking list
        if let Some(domain) = urls::extract_domain(src) {
            if is_tracking_domain(&domain) {
                if !hits.iter().any(|h| h.url == src) {
                    hits.push(TrackingHit {
                        url: src.to_string(),
                        hit_type: "tracking_pixel".to_string(),
                        domain,
                    });
                }
            }
        }

        // Check for tracking params in URL
        if RE_TRACKING_PARAMS.is_match(src) {
            if let Some(domain) = urls::extract_domain(src) {
                if !hits.iter().any(|h| h.url == src) {
                    hits.push(TrackingHit {
                        url: src.to_string(),
                        hit_type: "tracking_param".to_string(),
                        domain,
                    });
                }
            }
        }
    }

    // Tracking stylesheets
    for el in document.select(&SEL_LINK) {
        if let Some(href) = el.value().attr("href") {
            if let Some(domain) = urls::extract_domain(href) {
                if is_tracking_domain(&domain) {
                    hits.push(TrackingHit {
                        url: href.to_string(),
                        hit_type: "tracking_stylesheet".to_string(),
                        domain,
                    });
                }
            }
        }
    }

    hits
}

/// Check if a domain (or parent domain) is a known tracking domain.
fn is_tracking_domain(domain: &str) -> bool {
    if TRACKING_DOMAINS.contains(domain) {
        return true;
    }
    // Check if it's a subdomain of a tracking domain
    for td in TRACKING_DOMAINS.iter() {
        if domain.ends_with(&format!(".{}", td)) {
            return true;
        }
    }
    false
}

// ─── QR code detection ───────────────────────────────────────────────────

/// QR code candidate finding.
#[derive(Debug, Clone)]
pub struct QrFinding {
    pub image_src: String,
    pub image_type: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub alt_text: Option<String>,
}

/// Detect potential QR code images in HTML.
pub fn detect_qr_candidates(html: &str) -> Vec<QrFinding> {
    let document = Html::parse_document(html);
    let mut findings: Vec<QrFinding> = Vec::new();

    for el in document.select(&SEL_IMG) {
        let src = el.value().attr("src").unwrap_or_default();
        let alt = el.value().attr("alt").unwrap_or_default();
        let w = el.value().attr("width").and_then(|v| v.parse::<i32>().ok());
        let h = el.value().attr("height").and_then(|v| v.parse::<i32>().ok());

        let alt_lower = alt.to_lowercase();
        let src_lower = src.to_lowercase();

        let is_qr_candidate =
            // Alt contains "qr" or "scan"
            alt_lower.contains("qr") || alt_lower.contains("scan") ||
            // Data URI image
            src_lower.starts_with("data:image/") ||
            // Square image >= 100px (typical QR)
            matches!((w, h), (Some(wv), Some(hv)) if wv == hv && wv >= 100) ||
            // URL path contains "qr"
            src_lower.contains("/qr") || src_lower.contains("qr=") || src_lower.contains("qrcode");

        if is_qr_candidate && !src.is_empty() {
            let image_type = if src.starts_with("data:") {
                "data_uri"
            } else if src.starts_with("cid:") {
                "attachment"
            } else {
                "external"
            };

            findings.push(QrFinding {
                image_src: src.to_string(),
                image_type: image_type.to_string(),
                width: w,
                height: h,
                alt_text: if alt.is_empty() { None } else { Some(alt.to_string()) },
            });
        }
    }

    findings
}

// ─── Plain text URL extraction ────────────────────────────────────────────

static RE_PLAIN_URL: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"https?://[^\s"'<>]+"#).unwrap()
});

/// Extract URLs from plain text content.
pub fn extract_urls_from_text(text: &str, sender_domain: &str) -> Vec<ExtractedUrl> {
    RE_PLAIN_URL
        .find_iter(text)
        .map(|m| urls::classify_url(m.as_str(), "plain_text", sender_domain))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_plain_text() {
        let html = "<html><body><p>Hello <b>world</b></p><script>var x=1;</script></body></html>";
        let text = extract_plain_text(html);
        assert!(text.contains("Hello"));
        assert!(text.contains("world"));
    }

    #[test]
    fn test_extract_urls() {
        let html = r#"<html><body><a href="https://evil.com/phish">click</a></body></html>"#;
        let urls = extract_urls_from_html(html, "good.com");
        assert!(!urls.is_empty());
        assert_eq!(urls[0].url_type, "href");
    }

    #[test]
    fn test_obfuscation_display_none() {
        let html = r#"<div style="display:none">hidden content</div>"#;
        let report = detect_obfuscation(html);
        assert!(report.has_obfuscation);
        assert!(report.techniques.contains(&"display_none".to_string()));
    }

    #[test]
    fn test_qr_detection() {
        let html = r#"<img src="https://example.com/qrcode.png" alt="Scan QR" width="200" height="200">"#;
        let findings = detect_qr_candidates(html);
        assert!(!findings.is_empty());
    }
}
