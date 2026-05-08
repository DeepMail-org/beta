/// Unicode script analysis, mixed-script detection, Punycode abuse detection,
/// and NFKC normalization.

use std::collections::HashSet;

use unicode_normalization::UnicodeNormalization;

/// Determine the Unicode script of a character based on code point ranges.
pub fn get_script(c: char) -> &'static str {
    let cp = c as u32;
    match cp {
        // Common: digits, hyphen, dot, underscore
        0x0030..=0x0039 => "Common",
        0x002D => "Common",
        0x002E => "Common",
        0x005F => "Common",

        // Latin (Basic Latin letters + supplements + extensions)
        0x0041..=0x005A | 0x0061..=0x007A => "Latin",
        0x00C0..=0x00FF => "Latin",   // Latin-1 Supplement (accented chars)
        0x0100..=0x024F => "Latin",   // Latin Extended-A, Extended-B
        0x0250..=0x02AF => "Latin",   // IPA Extensions
        0x1E00..=0x1EFF => "Latin",   // Latin Extended Additional
        0x2C60..=0x2C7F => "Latin",   // Latin Extended-C

        // Greek
        0x0370..=0x03FF => "Greek",
        0x1F00..=0x1FFF => "Greek",   // Greek Extended

        // Cyrillic
        0x0400..=0x04FF => "Cyrillic",
        0x0500..=0x052F => "Cyrillic", // Cyrillic Supplement

        // Arabic
        0x0600..=0x06FF => "Arabic",
        0x0750..=0x077F => "Arabic",

        // Devanagari
        0x0900..=0x097F => "Devanagari",

        // CJK
        0x4E00..=0x9FFF => "CJK",
        0x3400..=0x4DBF => "CJK",

        // Hangul
        0xAC00..=0xD7AF => "Hangul",

        // Hiragana
        0x3040..=0x309F => "Hiragana",

        // Katakana
        0x30A0..=0x30FF => "Katakana",

        // Common symbols / math
        0x2000..=0x206F => "Common",  // General punctuation
        0x2100..=0x214F => "Common",  // Letterlike symbols
        0x2150..=0x218F => "Common",  // Number forms (Roman numerals etc.)
        0xFF00..=0xFFEF => "Common",  // Fullwidth forms

        _ => "Unknown",
    }
}

/// Check if a single label has mixed Unicode scripts (excluding Common/Unknown).
pub fn is_mixed_script(label: &str) -> bool {
    let mut scripts: HashSet<&str> = HashSet::new();
    for c in label.chars() {
        let s = get_script(c);
        if s != "Common" && s != "Unknown" {
            scripts.insert(s);
        }
    }
    scripts.len() > 1
}

/// Check if any label in the domain has mixed scripts.
pub fn domain_has_mixed_script(domain: &str) -> bool {
    domain.split('.').any(|label| is_mixed_script(label))
}

/// Check for Punycode abuse: xn-- labels that decode to non-Latin chars.
pub fn has_punycode_abuse(domain: &str) -> bool {
    for label in domain.split('.') {
        if let Some(encoded) = label.strip_prefix("xn--") {
            match punycode::decode(encoded) {
                Ok(decoded_str) => {
                    for c in decoded_str.chars() {
                        if !c.is_ascii() {
                            let script = get_script(c);
                            if script != "Latin" && script != "Common" && script != "Unknown" {
                                return true;
                            }
                        }
                    }
                }
                Err(_) => {
                    // Decode failed — suspicious but not necessarily abuse
                    continue;
                }
            }
        }
    }
    false
}

/// Decode all Punycode labels in a domain.
pub fn decode_punycode_domain(domain: &str) -> String {
    domain
        .split('.')
        .map(|label| {
            if let Some(encoded) = label.strip_prefix("xn--") {
                match punycode::decode(encoded) {
                    Ok(decoded) => decoded,
                    Err(_) => label.to_string(),
                }
            } else {
                label.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(".")
}

/// NFKC normalize a string (compatibility decomposition + canonical composition).
pub fn nfkc_normalize(s: &str) -> String {
    s.nfkc().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_detection() {
        assert_eq!(get_script('a'), "Latin");
        assert_eq!(get_script('А'), "Cyrillic");  // U+0410
        assert_eq!(get_script('α'), "Greek");      // U+03B1
        assert_eq!(get_script('0'), "Common");
        assert_eq!(get_script('-'), "Common");
    }

    #[test]
    fn test_mixed_script_latin_only() {
        assert!(!is_mixed_script("google"));
        assert!(!is_mixed_script("test123")); // digits are Common
    }

    #[test]
    fn test_mixed_script_cyrillic_latin() {
        // "goоgle" with Cyrillic о (U+043E)
        assert!(is_mixed_script("go\u{043E}gle"));
    }

    #[test]
    fn test_punycode_decode() {
        // "xn--pple-43d" decodes to "аpple" (Cyrillic а + Latin pple)
        let decoded = decode_punycode_domain("xn--pple-43d.com");
        assert!(decoded.contains("pple"));
    }

    #[test]
    fn test_nfkc() {
        // Fullwidth A (U+FF21) → Latin A
        assert_eq!(nfkc_normalize("\u{FF21}"), "A");
    }
}
