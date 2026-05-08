/// Visual similarity scoring: edit distance, skeleton comparison, risk assignment.

use crate::brands::BrandRegistry;
use crate::confusables::{compute_skeleton, is_likely_idn, apply_skeleton_with_context};
use crate::unicode::{decode_punycode_domain, domain_has_mixed_script, has_punycode_abuse, nfkc_normalize};

/// Known multi-part TLDs for eTLD+1 extraction.
static MULTI_PART_TLDS: &[&str] = &[
    "co.uk", "co.in", "com.br", "co.jp", "com.au",
    "co.nz", "co.za", "com.mx", "co.kr", "net.au",
];

/// Risk level for a domain's homograph similarity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    None,
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            RiskLevel::None => "NONE",
            RiskLevel::Low => "LOW",
            RiskLevel::Medium => "MEDIUM",
            RiskLevel::High => "HIGH",
            RiskLevel::Critical => "CRITICAL",
        }
    }

    pub fn from_score(score: f32) -> Self {
        if score >= 0.90 {
            RiskLevel::Critical
        } else if score >= 0.80 {
            RiskLevel::High
        } else if score >= 0.70 {
            RiskLevel::Medium
        } else if score >= 0.60 {
            RiskLevel::Low
        } else {
            RiskLevel::None
        }
    }

    pub fn to_proto_i32(&self) -> i32 {
        match self {
            RiskLevel::None => 0,
            RiskLevel::Low => 1,
            RiskLevel::Medium => 2,
            RiskLevel::High => 3,
            RiskLevel::Critical => 4,
        }
    }
}

/// Similarity score result for one domain vs one brand.
#[derive(Debug, Clone)]
pub struct SimilarityScore {
    pub brand: String,
    pub raw_similarity: f32,
    pub skeleton_match: bool,
    pub edit_distance: usize,
    pub mixed_script: bool,
    pub punycode_abuse: bool,
    pub final_score: f32,
    pub risk_level: RiskLevel,
}

/// Extract registrable domain (eTLD+1) from a full domain.
///
/// Simple approach: check for multi-part TLDs, then take last 2 or 3 parts.
pub fn extract_registrable_domain(domain: &str) -> String {
    let d = domain
        .trim()
        .trim_start_matches("*.")
        .trim_end_matches('.');
    let parts: Vec<&str> = d.split('.').collect();

    if parts.len() <= 1 {
        return d.to_string();
    }

    // Check if last 2 parts match a multi-part TLD
    if parts.len() >= 3 {
        let last_two = format!("{}.{}", parts[parts.len() - 2], parts[parts.len() - 1]);
        if MULTI_PART_TLDS.contains(&last_two.as_str()) {
            // Take last 3 parts
            let start = parts.len().saturating_sub(3);
            return parts[start..].join(".");
        }
    }

    // Take last 2 parts
    let start = parts.len().saturating_sub(2);
    parts[start..].join(".")
}

/// Standard Wagner-Fischer edit distance on char arrays.
pub fn edit_distance(a: &str, b: &str) -> usize {
    if a == b {
        return 0;
    }

    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();

    let n = a_chars.len();
    let m = b_chars.len();

    if n == 0 {
        return m;
    }
    if m == 0 {
        return n;
    }

    let mut dp = vec![vec![0usize; m + 1]; n + 1];

    for i in 0..=n {
        dp[i][0] = i;
    }
    for j in 0..=m {
        dp[0][j] = j;
    }

    for i in 1..=n {
        for j in 1..=m {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }

    dp[n][m]
}

/// Compute full similarity between a domain and a single brand.
pub fn compute_similarity(
    domain: &str,
    brand_entry: &crate::brands::BrandEntry,
) -> SimilarityScore {
    // Step 1: Registrable domains
    let domain_registrable = extract_registrable_domain(domain);

    // Step 2: Punycode decode
    let decoded = decode_punycode_domain(&domain_registrable);

    // Step 3: NFKC normalize
    let normalized = nfkc_normalize(&decoded);

    // Step 4: Skeleton
    let is_idn = is_likely_idn(&normalized);
    let domain_skeleton = apply_skeleton_with_context(&normalized, is_idn);
    let brand_skeleton = &brand_entry.skeleton;

    // Step 5: Edit distance
    let dist = edit_distance(&domain_skeleton, brand_skeleton);

    // Step 6: Raw similarity
    let max_len = domain_skeleton.len().max(brand_skeleton.len());
    let raw_similarity = if max_len == 0 {
        0.0
    } else {
        (1.0 - (dist as f32 / max_len as f32)).clamp(0.0, 1.0)
    };

    // Step 7: Bonuses
    let mixed_script = domain_has_mixed_script(&decoded);
    let punycode_abuse_flag = has_punycode_abuse(domain);
    let skeleton_match = domain_skeleton == *brand_skeleton;

    let mut bonus: f32 = 0.0;

    if mixed_script {
        bonus += 0.20;
    }
    if punycode_abuse_flag {
        bonus += 0.15;
    }
    if skeleton_match {
        bonus += 0.10;
    }
    if dist == 1 {
        bonus += 0.10;
    }
    if domain_skeleton.contains(brand_skeleton.as_str()) {
        bonus += 0.05;
    }

    // Step 8: Final score
    let final_score = (raw_similarity + bonus).clamp(0.0, 1.0);

    // Step 9: Risk level
    let risk_level = RiskLevel::from_score(final_score);

    SimilarityScore {
        brand: brand_entry.domain.clone(),
        raw_similarity,
        skeleton_match,
        edit_distance: dist,
        mixed_script,
        punycode_abuse: punycode_abuse_flag,
        final_score,
        risk_level,
    }
}

/// Find the best brand match for a domain across all brands in the registry.
pub fn find_best_match(domain: &str, registry: &BrandRegistry) -> SimilarityScore {
    let mut best: Option<SimilarityScore> = None;

    for entry in &registry.entries {
        let score = compute_similarity(domain, entry);
        match &best {
            None => best = Some(score),
            Some(current) => {
                if score.final_score > current.final_score {
                    best = Some(score);
                }
            }
        }
    }

    best.unwrap_or(SimilarityScore {
        brand: String::new(),
        raw_similarity: 0.0,
        skeleton_match: false,
        edit_distance: usize::MAX,
        mixed_script: false,
        punycode_abuse: false,
        final_score: 0.0,
        risk_level: RiskLevel::None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_registrable_simple() {
        assert_eq!(extract_registrable_domain("mail.google.com"), "google.com");
        assert_eq!(extract_registrable_domain("google.com"), "google.com");
    }

    #[test]
    fn test_extract_registrable_multi_part_tld() {
        assert_eq!(extract_registrable_domain("sbi.co.in"), "sbi.co.in");
        assert_eq!(extract_registrable_domain("www.sbi.co.in"), "sbi.co.in");
    }

    #[test]
    fn test_edit_distance() {
        assert_eq!(edit_distance("kitten", "sitting"), 3);
        assert_eq!(edit_distance("google", "gooogle"), 1);
        assert_eq!(edit_distance("google", "google"), 0);
        assert_eq!(edit_distance("", "abc"), 3);
    }

    #[test]
    fn test_risk_levels() {
        assert_eq!(RiskLevel::from_score(0.95), RiskLevel::Critical);
        assert_eq!(RiskLevel::from_score(0.85), RiskLevel::High);
        assert_eq!(RiskLevel::from_score(0.75), RiskLevel::Medium);
        assert_eq!(RiskLevel::from_score(0.65), RiskLevel::Low);
        assert_eq!(RiskLevel::from_score(0.50), RiskLevel::None);
    }
}
