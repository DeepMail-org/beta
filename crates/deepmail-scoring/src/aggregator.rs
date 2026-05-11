use crate::signals::SignalName;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum FinalVerdict {
    Clean,
    LowRisk,
    Suspicious,
    Phishing,
    Malicious,
}

impl FinalVerdict {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Clean => "CLEAN",
            Self::LowRisk => "LOW_RISK",
            Self::Suspicious => "SUSPICIOUS",
            Self::Phishing => "PHISHING",
            Self::Malicious => "MALICIOUS",
        }
    }

    pub fn from_score(score: f32) -> Self {
        if score >= 0.80 {
            Self::Malicious
        } else if score >= 0.60 {
            Self::Phishing
        } else if score >= 0.40 {
            Self::Suspicious
        } else if score >= 0.20 {
            Self::LowRisk
        } else {
            Self::Clean
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompositeResult {
    pub final_score: f32,
    pub final_verdict: FinalVerdict,
    pub signals: HashMap<SignalName, f32>,
    pub signals_available: i32,
    pub weight_total: f32,
}

pub fn compute_composite_score(signals: &HashMap<SignalName, f32>) -> CompositeResult {
    let mut weighted_sum = 0.0_f32;
    let mut weight_total = 0.0_f32;

    for (name, &score) in signals {
        let w = name.weight();
        weighted_sum += score * w;
        weight_total += w;
    }

    let final_score = if weight_total > 0.0 {
        (weighted_sum / weight_total).clamp(0.0, 1.0)
    } else {
        0.0
    };

    CompositeResult {
        final_score,
        final_verdict: FinalVerdict::from_score(final_score),
        signals: signals.clone(),
        signals_available: signals.len() as i32,
        weight_total,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_signals() {
        let signals = HashMap::new();
        let result = compute_composite_score(&signals);
        assert_eq!(result.final_score, 0.0);
        assert_eq!(result.final_verdict, FinalVerdict::Clean);
        assert_eq!(result.signals_available, 0);
    }

    #[test]
    fn all_signals_max() {
        let mut signals = HashMap::new();
        for &name in SignalName::all() {
            signals.insert(name, 1.0);
        }
        let result = compute_composite_score(&signals);
        assert!((result.final_score - 1.0).abs() < 0.001);
        assert_eq!(result.final_verdict, FinalVerdict::Malicious);
        assert_eq!(result.signals_available, 11);
    }

    #[test]
    fn single_body_signal() {
        let mut signals = HashMap::new();
        signals.insert(SignalName::Body, 0.9);
        let result = compute_composite_score(&signals);
        assert!((result.final_score - 0.9).abs() < 0.001);
        assert_eq!(result.final_verdict, FinalVerdict::Malicious);
        assert_eq!(result.signals_available, 1);
    }

    #[test]
    fn mixed_signals() {
        let mut signals = HashMap::new();
        signals.insert(SignalName::Header, 0.5);
        signals.insert(SignalName::Body, 0.8);
        signals.insert(SignalName::Ip, 0.3);

        let expected_weighted = (0.5 * 0.12) + (0.8 * 0.14) + (0.3 * 0.10);
        let expected_weight_total = 0.12 + 0.14 + 0.10;
        let expected_score = expected_weighted / expected_weight_total;

        let result = compute_composite_score(&signals);
        assert!((result.final_score - expected_score).abs() < 0.001);
        assert_eq!(result.signals_available, 3);
    }
}
