/// Composite IP threat scoring with 16+ weighted signals.

#[derive(Debug, Clone, PartialEq)]
pub enum ThreatVerdict {
    Clean,
    Low,
    Medium,
    High,
    Critical,
}

impl ThreatVerdict {
    pub fn as_str(&self) -> &'static str {
        match self {
            ThreatVerdict::Clean => "CLEAN",
            ThreatVerdict::Low => "LOW",
            ThreatVerdict::Medium => "MEDIUM",
            ThreatVerdict::High => "HIGH",
            ThreatVerdict::Critical => "CRITICAL",
        }
    }

    pub fn from_score(score: f32) -> Self {
        if score >= 0.75 {
            ThreatVerdict::Critical
        } else if score >= 0.50 {
            ThreatVerdict::High
        } else if score >= 0.25 {
            ThreatVerdict::Medium
        } else if score >= 0.10 {
            ThreatVerdict::Low
        } else {
            ThreatVerdict::Clean
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SignalSet {
    pub in_feodo: bool,
    pub in_spamhaus_drop: bool,
    pub in_spamhaus_edrop: bool,
    pub in_emerging_threats: bool,
    pub in_cins_army: bool,
    pub in_blocklist_de: bool,
    pub in_tor: bool,
    pub in_brute_force: bool,
    pub in_alienvault: bool,
    pub abuse_score: Option<i32>,
    pub shodan_tags: Vec<String>,
    pub shodan_has_vulns: bool,
    pub is_bogon: bool,
    pub pdns_hostname_count: usize,
}

const MALICIOUS_SHODAN_TAGS: &[&str] = &["malware", "c2", "botnet", "scanner"];

pub fn compute_threat_score(signals: &SignalSet) -> (f32, ThreatVerdict) {
    let mut score: f32 = 0.0;

    if signals.in_feodo           { score += 0.45; }
    if signals.in_spamhaus_drop   { score += 0.40; }
    if signals.in_spamhaus_edrop  { score += 0.40; }
    if signals.in_emerging_threats { score += 0.35; }
    if signals.in_cins_army       { score += 0.30; }
    if signals.in_blocklist_de    { score += 0.25; }
    if signals.in_tor             { score += 0.35; }
    if signals.in_brute_force     { score += 0.20; }
    if signals.in_alienvault      { score += 0.30; }

    if let Some(abuse) = signals.abuse_score {
        if abuse >= 90 {
            score += 0.40;
        } else if abuse >= 70 {
            score += 0.30;
        } else if abuse >= 40 {
            score += 0.15;
        }
    }

    if signals.shodan_has_vulns { score += 0.25; }

    // Shodan malicious tags: +0.20 each, capped at 0.40
    let mut tag_contribution: f32 = 0.0;
    for tag in &signals.shodan_tags {
        let lower = tag.to_lowercase();
        if MALICIOUS_SHODAN_TAGS.contains(&lower.as_str()) {
            tag_contribution += 0.20;
            if tag_contribution >= 0.40 {
                tag_contribution = 0.40;
                break;
            }
        }
    }
    score += tag_contribution;

    if signals.is_bogon { score += 0.50; }

    if signals.pdns_hostname_count >= 100 { score += 0.15; }

    score = score.clamp(0.0, 1.0);
    let verdict = ThreatVerdict::from_score(score);
    (score, verdict)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_ip_scores_zero() {
        let signals = SignalSet::default();
        let (score, verdict) = compute_threat_score(&signals);
        assert_eq!(score, 0.0);
        assert_eq!(verdict, ThreatVerdict::Clean);
    }

    #[test]
    fn feodo_plus_high_abuse_is_critical() {
        let signals = SignalSet {
            in_feodo: true,
            abuse_score: Some(95),
            ..Default::default()
        };
        let (score, verdict) = compute_threat_score(&signals);
        assert!(score >= 0.75);
        assert_eq!(verdict, ThreatVerdict::Critical);
    }

    #[test]
    fn shodan_tags_capped() {
        let signals = SignalSet {
            shodan_tags: vec![
                "malware".to_string(),
                "c2".to_string(),
                "botnet".to_string(),
                "scanner".to_string(),
            ],
            ..Default::default()
        };
        let (score, _) = compute_threat_score(&signals);
        // 0.40 cap on shodan tags
        assert!((score - 0.40).abs() < f32::EPSILON);
    }

    #[test]
    fn score_clamped_to_one() {
        let signals = SignalSet {
            in_feodo: true,
            in_spamhaus_drop: true,
            in_spamhaus_edrop: true,
            in_emerging_threats: true,
            abuse_score: Some(95),
            is_bogon: true,
            ..Default::default()
        };
        let (score, verdict) = compute_threat_score(&signals);
        assert_eq!(score, 1.0);
        assert_eq!(verdict, ThreatVerdict::Critical);
    }
}
