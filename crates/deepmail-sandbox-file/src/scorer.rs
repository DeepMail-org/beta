/// Threat scoring engine for static file analysis.

/// Aggregated findings from all analysis tools.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct FileFindings {
    pub entropy: f32,
    pub has_macros: bool,
    pub has_vba: bool,
    pub vba_suspicious: bool,
    pub is_pe: bool,
    pub pe_is_packed: bool,
    pub pe_suspicious_imports: Vec<String>,
    pub is_pdf: bool,
    pub pdf_has_js: bool,
    pub pdf_has_launch: bool,
    pub pdf_is_encrypted: bool,
    pub has_embedded: bool,
    pub yara_matches: Vec<String>,
    pub suspicious_strings: Vec<String>,
    pub macro_count: i32,
    /// True if at least one tool ran successfully.
    pub any_tool_ran: bool,
}

/// Threat verdict categories.
#[derive(Debug, Clone, PartialEq)]
pub enum ThreatVerdict {
    Clean,
    Suspicious,
    Malicious,
    Packed,
    Unknown,
}

impl ThreatVerdict {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Clean => "CLEAN",
            Self::Suspicious => "SUSPICIOUS",
            Self::Malicious => "MALICIOUS",
            Self::Packed => "PACKED",
            Self::Unknown => "UNKNOWN",
        }
    }
}

/// Compute composite threat score, verdict, and analysis notes.
pub fn compute_threat_score(findings: &FileFindings) -> (f32, ThreatVerdict, Vec<String>) {
    if !findings.any_tool_ran {
        return (0.0, ThreatVerdict::Unknown, vec!["No tools ran successfully".into()]);
    }

    let mut score: f32 = 0.0;
    let mut notes: Vec<String> = Vec::new();

    // ── Entropy signals ─────────────────────────────────────────────────
    if findings.entropy > 7.5 {
        score += 0.35;
        notes.push(format!("Very high entropy: {:.2}", findings.entropy));
    } else if findings.entropy > 7.0 {
        score += 0.20;
        notes.push(format!("High entropy: {:.2}", findings.entropy));
    }

    // ── YARA matches ────────────────────────────────────────────────────
    if !findings.yara_matches.is_empty() {
        let yara_contrib = (findings.yara_matches.len() as f32 * 0.30).min(0.60);
        score += yara_contrib;
        for rule in &findings.yara_matches {
            notes.push(format!("YARA match: {}", rule));
        }
    }

    // ── Macro signals ───────────────────────────────────────────────────
    if findings.has_macros && findings.vba_suspicious {
        score += 0.50;
        notes.push("Suspicious VBA macro detected".into());
    } else if findings.has_macros {
        score += 0.20;
        notes.push("Contains macros".into());
    }

    // ── PDF signals ─────────────────────────────────────────────────────
    if findings.pdf_has_js && findings.pdf_has_launch {
        score += 0.55;
        notes.push("PDF with JavaScript and launch action".into());
    } else if findings.pdf_has_js {
        score += 0.30;
        notes.push("PDF with JavaScript".into());
    } else if findings.pdf_has_launch {
        score += 0.40;
        notes.push("PDF with launch action".into());
    }

    // ── PE signals ──────────────────────────────────────────────────────
    if findings.pe_is_packed {
        score += 0.35;
        notes.push("PE appears packed".into());
    }
    if !findings.pe_suspicious_imports.is_empty() {
        let pe_contrib = (findings.pe_suspicious_imports.len() as f32 * 0.05).min(0.30);
        score += pe_contrib;
        for func in &findings.pe_suspicious_imports {
            notes.push(format!("Suspicious import: {}", func));
        }
    }

    // ── Embedded file signals ───────────────────────────────────────────
    if findings.has_embedded {
        score += 0.20;
        notes.push("Embedded files detected".into());
    }

    // ── Suspicious strings ──────────────────────────────────────────────
    if findings.suspicious_strings.len() >= 5 {
        score += 0.15;
        notes.push(format!("Multiple suspicious strings: {}", findings.suspicious_strings.len()));
    } else if !findings.suspicious_strings.is_empty() {
        score += 0.05;
    }

    // Clamp score
    let final_score = score.clamp(0.0, 1.0);

    // Determine verdict
    let verdict = if final_score >= 0.80 {
        ThreatVerdict::Malicious
    } else if final_score >= 0.55 {
        ThreatVerdict::Suspicious
    } else if final_score >= 0.35 && findings.entropy > 7.0 && !findings.has_macros && !findings.pdf_has_js {
        ThreatVerdict::Packed
    } else if final_score >= 0.35 {
        ThreatVerdict::Suspicious
    } else {
        ThreatVerdict::Clean
    };

    (final_score, verdict, notes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_file() {
        let findings = FileFindings {
            entropy: 4.5,
            any_tool_ran: true,
            ..Default::default()
        };
        let (score, verdict, _notes) = compute_threat_score(&findings);
        assert!(score < 0.35);
        assert_eq!(verdict, ThreatVerdict::Clean);
    }

    #[test]
    fn test_malicious_macro_file() {
        let findings = FileFindings {
            entropy: 5.0,
            has_macros: true,
            has_vba: true,
            vba_suspicious: true,
            yara_matches: vec!["SuspiciousMacroKeywords".into()],
            suspicious_strings: (0..6).map(|i| format!("susp{}", i)).collect(),
            any_tool_ran: true,
            ..Default::default()
        };
        let (score, verdict, notes) = compute_threat_score(&findings);
        assert!(score >= 0.80, "score should be >= 0.80, got {}", score);
        assert_eq!(verdict, ThreatVerdict::Malicious);
        assert!(notes.iter().any(|n| n.contains("YARA match")));
    }

    #[test]
    fn test_unknown_no_tools() {
        let findings = FileFindings::default();
        let (_score, verdict, _notes) = compute_threat_score(&findings);
        assert_eq!(verdict, ThreatVerdict::Unknown);
    }
}
