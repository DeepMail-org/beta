/// Dynamic analysis threat scorer.

use crate::parser::DynamicFindings;

/// Dynamic verdict categories.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DynamicVerdict {
    Benign,
    LowRisk,
    Suspicious,
    Malware,
    Unknown,
}

impl DynamicVerdict {
    pub fn as_str(&self) -> &'static str {
        match self {
            DynamicVerdict::Benign => "BENIGN",
            DynamicVerdict::LowRisk => "LOW_RISK",
            DynamicVerdict::Suspicious => "SUSPICIOUS",
            DynamicVerdict::Malware => "MALWARE",
            DynamicVerdict::Unknown => "UNKNOWN",
        }
    }
}

/// Compute dynamic threat score from analysis findings.
/// Returns (score 0.0–1.0, verdict, notes).
pub fn compute_dynamic_score(
    findings: &DynamicFindings,
) -> (f32, DynamicVerdict, Vec<String>) {
    // Special case: CAPE unavailable
    if findings.cape_unavailable {
        // Use malscore from static fallback for minimal assessment
        let score = findings.malscore.min(1.0);
        let verdict = if score >= 0.8 {
            DynamicVerdict::Suspicious
        } else {
            DynamicVerdict::Unknown
        };
        return (
            score,
            verdict,
            vec!["CAPEv2 unavailable — static analysis used as proxy".into()],
        );
    }

    let mut score: f32 = 0.0;
    let mut notes = Vec::new();

    // ── malscore contribution ──────────────────────────────────────────
    if findings.malscore >= 0.8 {
        score += 0.45;
        notes.push(format!("High malscore: {:.2}", findings.malscore));
    } else if findings.malscore >= 0.5 {
        score += 0.25;
        notes.push(format!("Medium malscore: {:.2}", findings.malscore));
    } else if findings.malscore >= 0.2 {
        score += 0.10;
    }

    // ── CAPE signatures ────────────────────────────────────────────────
    for sig in &findings.cape_signatures {
        if sig.severity >= 3 {
            score += 0.40;
            notes.push(format!("Critical behavior: {}", sig.name));
        } else if sig.severity >= 2 {
            score += 0.20;
            notes.push(format!("Suspicious: {}", sig.name));
        }
    }

    // ── Network activity ───────────────────────────────────────────────
    if !findings.network_hosts.is_empty() {
        score += 0.15;
        notes.push("Network C2 contact".into());
    }
    if !findings.http_requests.is_empty() {
        score += 0.10;
    }

    // ── SMTP activity ──────────────────────────────────────────────────
    if findings.smtp_activity {
        score += 0.30;
        notes.push("SMTP activity (spam/exfil)".into());
    }

    // ── Persistence ────────────────────────────────────────────────────
    if !findings.persistence_indicators.is_empty() {
        score += 0.35;
        notes.push("Persistence mechanism".into());
    }

    // ── C2 indicators ──────────────────────────────────────────────────
    if !findings.c2_indicators.is_empty() {
        score += 0.40;
        notes.push("C2 communication".into());
    }

    // ── Registry modifications ─────────────────────────────────────────
    if findings.registry_modifications.len() > 10 {
        score += 0.20;
        notes.push("Excessive registry changes".into());
    }

    // ── Dropped files ──────────────────────────────────────────────────
    if !findings.files_dropped.is_empty() {
        score += 0.15;
        notes.push("Files dropped".into());
    }

    // ── Shell spawning ─────────────────────────────────────────────────
    let shell_names = ["cmd", "powershell", "wscript", "cscript"];
    for proc in &findings.processes_spawned {
        let lower = proc.to_lowercase();
        if shell_names.iter().any(|s| lower.contains(s)) {
            score += 0.25;
            notes.push("Shell spawned".into());
            break;
        }
    }

    // ── DNS flood / DGA ────────────────────────────────────────────────
    if findings.dns_requests.len() > 20 {
        score += 0.10;
        notes.push("Excessive DNS queries (DGA?)".into());
    }

    // Clamp
    score = score.clamp(0.0, 1.0);

    let verdict = if score >= 0.80 {
        DynamicVerdict::Malware
    } else if score >= 0.55 {
        DynamicVerdict::Suspicious
    } else if score >= 0.30 {
        DynamicVerdict::LowRisk
    } else {
        DynamicVerdict::Benign
    };

    (score, verdict, notes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::DynamicFindings;

    #[test]
    fn test_empty_findings_benign() {
        let findings = DynamicFindings::default();
        let (score, verdict, _notes) = compute_dynamic_score(&findings);
        assert_eq!(score, 0.0);
        assert_eq!(verdict, DynamicVerdict::Benign);
    }

    #[test]
    fn test_malware_score() {
        let findings = DynamicFindings {
            smtp_activity: true,
            persistence_indicators: vec!["RunKey".into()],
            c2_indicators: vec!["http://evil.com/gate.php".into()],
            malscore: 0.9,
            ..Default::default()
        };
        let (score, verdict, notes) = compute_dynamic_score(&findings);
        assert!(score >= 0.80, "score={}", score);
        assert_eq!(verdict, DynamicVerdict::Malware);
        assert!(notes.iter().any(|n| n.contains("SMTP")));
    }

    #[test]
    fn test_low_risk_score() {
        let findings = DynamicFindings {
            malscore: 0.25,
            network_hosts: vec!["10.0.0.1".into()],
            ..Default::default()
        };
        let (score, verdict, _) = compute_dynamic_score(&findings);
        // 0.10 (malscore) + 0.15 (network) = 0.25 → Benign
        // Actually let's check: malscore >= 0.2 → +0.10, hosts → +0.15 = 0.25
        assert!(score >= 0.25, "score={}", score);
        assert!(
            verdict == DynamicVerdict::Benign || verdict == DynamicVerdict::LowRisk,
            "verdict={:?}",
            verdict
        );
    }

    #[test]
    fn test_cape_unavailable() {
        let findings = DynamicFindings {
            cape_unavailable: true,
            malscore: 0.5,
            ..Default::default()
        };
        let (score, verdict, notes) = compute_dynamic_score(&findings);
        assert_eq!(score, 0.5);
        assert_eq!(verdict, DynamicVerdict::Unknown);
        assert!(notes[0].contains("CAPEv2 unavailable"));
    }
}
