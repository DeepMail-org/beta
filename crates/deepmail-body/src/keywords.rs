/// Phishing keyword scoring and urgency detection.

use once_cell::sync::Lazy;

/// Weighted keyword list for phishing detection.
/// Format: (keyword_phrase, weight)
pub static KEYWORD_LIST: Lazy<Vec<(&'static str, f32)>> = Lazy::new(|| {
    vec![
        // High weight (0.15)
        ("verify your account", 0.15),
        ("confirm your identity", 0.15),
        ("update your payment", 0.15),
        ("your account has been suspended", 0.15),
        ("unusual sign-in activity", 0.15),
        ("click here to verify", 0.15),
        ("your password will expire", 0.15),
        ("action required", 0.15),
        ("immediate action", 0.15),
        ("account locked", 0.15),
        ("unauthorized access", 0.15),
        ("security alert", 0.15),
        ("click the link below", 0.15),

        // Medium weight (0.08)
        ("dear customer", 0.08),
        ("dear user", 0.08),
        ("dear account holder", 0.08),
        ("kindly verify", 0.08),
        ("kindly confirm", 0.08),
        ("please verify", 0.08),
        ("login to continue", 0.08),
        ("sign in to continue", 0.08),
        ("validate your account", 0.08),
        ("reactivate your account", 0.08),
        ("limited time offer", 0.08),
        ("act now", 0.08),
        ("respond immediately", 0.08),
        ("bank transfer", 0.08),
        ("wire transfer", 0.08),
        ("gift card", 0.08),
        ("bitcoin", 0.08),
        ("cryptocurrency", 0.08),
        ("western union", 0.08),
        ("click here", 0.08),
        ("click the link", 0.08),
        ("follow the link", 0.08),

        // Low weight (0.03)
        ("free", 0.03),
        ("winner", 0.03),
        ("congratulations", 0.03),
        ("selected", 0.03),
        ("prize", 0.03),
        ("reward", 0.03),
        ("claim", 0.03),
        ("redeem", 0.03),
        ("invoice attached", 0.03),
        ("document shared", 0.03),
        ("fax received", 0.03),
        ("voicemail", 0.03),
        ("parcel", 0.03),
        ("delivery failed", 0.03),
        ("shipment", 0.03),
    ]
});

/// Urgency / social engineering phrases.
pub static URGENCY_PHRASES: Lazy<Vec<&'static str>> = Lazy::new(|| {
    vec![
        "act immediately",
        "respond within 24 hours",
        "respond within 48 hours",
        "expires today",
        "expires in 24 hours",
        "limited time",
        "urgent",
        "immediately",
        "without delay",
        "your account will be",
        "your account has been",
        "failure to respond",
        "failure to verify",
        "failure to confirm",
        "we detected",
        "we noticed",
        "suspicious activity",
        "unauthorized",
        "verify now",
        "confirm now",
        "update now",
        "click now",
        "your information",
        "personal details",
        "social security",
        "credit card",
        "bank account",
        "billing information",
        "password has expired",
        "password will expire",
        "reset your password",
    ]
});

/// Compute phishing keyword score from text.
///
/// Returns score in [0.0, 1.0]: sum of weights for matching keywords, clamped.
pub fn compute_keyword_score(text: &str) -> f32 {
    let lower = text.to_lowercase();
    let mut score: f32 = 0.0;

    for (keyword, weight) in KEYWORD_LIST.iter() {
        if lower.contains(keyword) {
            score += weight;
        }
    }

    score.clamp(0.0, 1.0)
}

/// Detect urgency level from text.
///
/// Returns score in [0.0, 1.0]: count * 0.08, clamped.
pub fn detect_urgency(text: &str) -> f32 {
    let lower = text.to_lowercase();
    let count = URGENCY_PHRASES
        .iter()
        .filter(|phrase| lower.contains(**phrase))
        .count();

    (count as f32 * 0.08).min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_score_phishing() {
        let text = "Dear customer, please verify your account. Click here to verify. Action required!";
        let score = compute_keyword_score(text);
        assert!(score > 0.3, "expected high score, got {}", score);
    }

    #[test]
    fn test_keyword_score_clean() {
        let text = "Hello, your order has been shipped. Track your package at the link.";
        let score = compute_keyword_score(text);
        assert!(score < 0.2, "expected low score, got {}", score);
    }

    #[test]
    fn test_urgency_detection() {
        let text = "Act immediately! Your account has been compromised. Verify now.";
        let urgency = detect_urgency(text);
        assert!(urgency > 0.0);
    }
}
