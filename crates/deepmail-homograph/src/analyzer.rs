/// Domain analysis orchestration: decode, normalize, skeleton, match.

use std::sync::Arc;

use crate::brands::BrandRegistry;
use crate::confusables::compute_skeleton;
use crate::similarity::{find_best_match, SimilarityScore};
use crate::unicode::{decode_punycode_domain, nfkc_normalize};

/// Result of analyzing one domain.
#[derive(Debug, Clone)]
pub struct DomainAnalysis {
    pub original_domain: String,
    pub decoded_domain: String,
    pub skeleton: String,
    pub best_match: SimilarityScore,
}

/// Analyze a single domain against the brand registry.
pub fn analyze_domain(domain: &str, brand_registry: &Arc<BrandRegistry>) -> DomainAnalysis {
    // Punycode decode
    let decoded = decode_punycode_domain(domain);

    // NFKC normalize
    let normalized = nfkc_normalize(&decoded);

    // Compute skeleton
    let skeleton = compute_skeleton(&normalized);

    // Find best brand match
    let best_match = find_best_match(domain, brand_registry);

    DomainAnalysis {
        original_domain: domain.to_string(),
        decoded_domain: normalized,
        skeleton,
        best_match,
    }
}

/// Check if a string is an IP address.
fn is_ip_address(s: &str) -> bool {
    s.parse::<std::net::IpAddr>().is_ok()
}

/// Analyze all domains from an email.
///
/// Filters out: brand domains (exact match), IP addresses, single-char labels.
/// Returns results sorted by final_score descending.
pub fn analyze_email_domains(
    domains: &[String],
    brand_registry: &Arc<BrandRegistry>,
    min_score: f32,
) -> Vec<DomainAnalysis> {
    let mut results: Vec<DomainAnalysis> = Vec::new();

    for domain in domains {
        let domain_lower = domain.to_lowercase();

        // Skip brand domains — we only check external/suspicious ones
        if brand_registry.is_brand_domain(&domain_lower) {
            continue;
        }

        // Skip IP addresses
        if is_ip_address(&domain_lower) {
            continue;
        }

        // Skip domains with single-char labels (too generic)
        if domain_lower.split('.').any(|label| label.len() <= 1 && !label.is_empty()) {
            // Allow single-char if it's part of a multi-label domain
            let parts: Vec<&str> = domain_lower.split('.').collect();
            if parts.len() < 2 {
                continue;
            }
        }

        // Skip too-short domains
        if domain_lower.len() < 4 {
            continue;
        }

        let analysis = analyze_domain(&domain_lower, brand_registry);

        // Only keep if above minimum score
        if analysis.best_match.final_score >= min_score {
            results.push(analysis);
        }
    }

    // Sort by final_score descending
    results.sort_by(|a, b| {
        b.best_match
            .final_score
            .partial_cmp(&a.best_match.final_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    results
}
