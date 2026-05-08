/// Brand impersonation registry with precomputed skeletons.

use crate::confusables::compute_skeleton;
use crate::similarity::extract_registrable_domain;

/// Static list of protected brand domains.
static BRAND_DOMAINS: &[&str] = &[
    "google.com",
    "gmail.com",
    "microsoft.com",
    "outlook.com",
    "office365.com",
    "apple.com",
    "icloud.com",
    "amazon.com",
    "aws.amazon.com",
    "paypal.com",
    "facebook.com",
    "instagram.com",
    "twitter.com",
    "linkedin.com",
    "github.com",
    "dropbox.com",
    "salesforce.com",
    "slack.com",
    "zoom.us",
    "stripe.com",
    "shopify.com",
    "wordpress.com",
    "godaddy.com",
    "namecheap.com",
    "cloudflare.com",
    "netflix.com",
    "spotify.com",
    "steam.com",
    "discord.com",
    "reddit.com",
    "chase.com",
    "bankofamerica.com",
    "wellsfargo.com",
    "citibank.com",
    "hsbc.com",
    "barclays.com",
    "icicibank.com",
    "hdfcbank.com",
    "sbi.co.in",
    "binance.com",
    "coinbase.com",
    "blockchain.com",
    "kraken.com",
];

/// A single brand entry with precomputed skeleton.
#[derive(Debug, Clone)]
pub struct BrandEntry {
    pub domain: String,
    pub registrable: String,
    pub skeleton: String,
}

/// Registry of all protected brands with precomputed skeletons.
pub struct BrandRegistry {
    pub entries: Vec<BrandEntry>,
}

impl BrandRegistry {
    /// Build the registry, precomputing skeleton for every brand.
    /// Entries sorted by domain length descending (longer = more specific first).
    pub fn new() -> Self {
        let mut entries: Vec<BrandEntry> = BRAND_DOMAINS
            .iter()
            .map(|&domain| {
                let registrable = extract_registrable_domain(domain);
                let skeleton = compute_skeleton(&registrable);
                BrandEntry {
                    domain: domain.to_string(),
                    registrable,
                    skeleton,
                }
            })
            .collect();

        // Sort by domain length descending for more specific match preference
        entries.sort_by(|a, b| b.domain.len().cmp(&a.domain.len()));

        tracing::info!(
            brand_count = entries.len(),
            "brand registry initialized"
        );

        Self { entries }
    }

    /// Check if a domain exactly matches a brand domain.
    pub fn is_brand_domain(&self, domain: &str) -> bool {
        let lower = domain.to_lowercase();
        self.entries.iter().any(|e| e.domain == lower)
    }
}
