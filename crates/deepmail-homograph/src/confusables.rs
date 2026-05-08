/// Unicode confusable character mappings and skeleton computation.

use std::collections::HashMap;

use once_cell::sync::Lazy;

/// Curated confusable character mappings: Unicode char → ASCII skeleton char.
/// Covers highest-risk phishing substitutions across Cyrillic, Greek, Roman
/// numeral, fullwidth digit, and IPA blocks.
pub static CONFUSABLE_MAP: Lazy<HashMap<char, char>> = Lazy::new(|| {
    let mut m = HashMap::with_capacity(60);

    // Cyrillic → Latin
    m.insert('\u{0430}', 'a'); // а → a
    m.insert('\u{0435}', 'e'); // е → e
    m.insert('\u{043E}', 'o'); // о → o
    m.insert('\u{0440}', 'p'); // р → p
    m.insert('\u{0441}', 'c'); // с → c
    m.insert('\u{0443}', 'y'); // у → y
    m.insert('\u{0445}', 'x'); // х → x
    m.insert('\u{0455}', 's'); // ѕ → s
    m.insert('\u{0456}', 'i'); // і → i
    m.insert('\u{0458}', 'j'); // ј → j  (note: spec says U+0457 but standard is U+0458)
    m.insert('\u{0501}', 'd'); // ԁ → d

    // Latin IPA
    m.insert('\u{0251}', 'a'); // ɑ → a
    m.insert('\u{0261}', 'g'); // ɡ → g

    // Script letters
    m.insert('\u{212F}', 'e'); // ℯ → e
    m.insert('\u{2134}', 'o'); // ℴ → o

    // Fullwidth digits
    m.insert('\u{FF10}', '0'); // ０ → 0
    m.insert('\u{FF11}', '1'); // １ → 1
    m.insert('\u{FF12}', '2'); // ２ → 2
    m.insert('\u{FF13}', '3'); // ３ → 3
    m.insert('\u{FF14}', '4'); // ４ → 4
    m.insert('\u{FF15}', '5'); // ５ → 5
    m.insert('\u{FF16}', '6'); // ６ → 6
    m.insert('\u{FF17}', '7'); // ７ → 7
    m.insert('\u{FF18}', '8'); // ８ → 8
    m.insert('\u{FF19}', '9'); // ９ → 9

    // Greek capitals → Latin
    m.insert('\u{0391}', 'A'); // Α → A
    m.insert('\u{0392}', 'B'); // Β → B
    m.insert('\u{0395}', 'E'); // Ε → E
    m.insert('\u{0396}', 'Z'); // Ζ → Z
    m.insert('\u{0397}', 'H'); // Η → H
    m.insert('\u{0399}', 'I'); // Ι → I
    m.insert('\u{039A}', 'K'); // Κ → K
    m.insert('\u{039C}', 'M'); // Μ → M
    m.insert('\u{039D}', 'N'); // Ν → N
    m.insert('\u{039F}', 'O'); // Ο → O
    m.insert('\u{03A1}', 'R'); // Ρ → R
    m.insert('\u{03A4}', 'T'); // Τ → T
    m.insert('\u{03A5}', 'Y'); // Υ → Y
    m.insert('\u{03A7}', 'X'); // Χ → X

    // Greek lowercase → Latin
    m.insert('\u{03B1}', 'a'); // α → a
    m.insert('\u{03B2}', 'b'); // β → b
    m.insert('\u{03BD}', 'v'); // ν → v
    m.insert('\u{03BF}', 'o'); // ο → o
    m.insert('\u{03C5}', 'u'); // υ → u
    m.insert('\u{03F2}', 'c'); // ϲ → c

    // Roman numerals
    m.insert('\u{2170}', 'i'); // ⅰ → i
    m.insert('\u{217C}', 'l'); // ⅼ → l
    m.insert('\u{217D}', 'c'); // ⅽ → c
    m.insert('\u{217E}', 'd'); // ⅾ → d
    m.insert('\u{217F}', 'm'); // ⅿ → m

    // Misc
    m.insert('|', 'l');        // | → l

    m
});

/// Check if a domain contains any non-ASCII characters (i.e., is likely IDN).
pub fn is_likely_idn(domain: &str) -> bool {
    domain.chars().any(|c| !c.is_ascii())
}

/// Compute the skeleton of a string with context-aware '1'→'l' mapping.
///
/// Steps:
/// 1. Lowercase
/// 2. Apply multi-char substitutions: "rn" → "m", "vv" → "w"
/// 3. Char-by-char confusable mapping
/// 4. If `is_idn` is true, also map '1' → 'l'
pub fn apply_skeleton_with_context(input: &str, is_idn: bool) -> String {
    let s = input.to_lowercase();
    // Multi-char substitutions first
    let s = s.replace("rn", "m").replace("vv", "w");

    s.chars()
        .map(|c| {
            if is_idn && c == '1' {
                return 'l';
            }
            CONFUSABLE_MAP.get(&c).copied().unwrap_or(c)
        })
        .collect()
}

/// Compute skeleton with automatic IDN detection.
pub fn compute_skeleton(input: &str) -> String {
    let is_idn = is_likely_idn(input);
    apply_skeleton_with_context(input, is_idn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skeleton_basic() {
        assert_eq!(compute_skeleton("google"), "google");
        assert_eq!(compute_skeleton("GOOGLE"), "google");
    }

    #[test]
    fn test_skeleton_cyrillic() {
        // "gооgle" with Cyrillic о (U+043E)
        assert_eq!(compute_skeleton("g\u{043E}\u{043E}gle"), "google");
    }

    #[test]
    fn test_skeleton_multi_char() {
        assert_eq!(compute_skeleton("rnicrosoft"), "microsoft");
        assert_eq!(compute_skeleton("vvindovvs"), "windows");
    }

    #[test]
    fn test_idn_digit_mapping() {
        // Only map '1' → 'l' if IDN context
        assert_eq!(apply_skeleton_with_context("g1obal", false), "g1obal");
        assert_eq!(apply_skeleton_with_context("g1obal", true), "global");
    }
}
