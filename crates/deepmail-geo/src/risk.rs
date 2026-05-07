#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskVerdict {
    None,
    Low,
    Medium,
    High,
    Critical,
}

impl RiskVerdict {
    pub fn as_str(&self) -> &'static str {
        match self {
            RiskVerdict::None => "NONE",
            RiskVerdict::Low => "LOW",
            RiskVerdict::Medium => "MEDIUM",
            RiskVerdict::High => "HIGH",
            RiskVerdict::Critical => "CRITICAL",
        }
    }
}

impl std::fmt::Display for RiskVerdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

pub struct HopRiskInput {
    pub is_tor: bool,
    pub is_vpn: bool,
    pub is_botnet: bool,
    pub abuse_score: Option<i32>,
    pub greynoise_class: Option<String>,
    pub ip_type: crate::classifier::IpType,
}

pub fn compute_risk(input: &HopRiskInput) -> (f32, RiskVerdict) {
    let mut score: f32 = 0.0;

    if input.is_tor {
        score += 0.40;
    }
    if input.is_vpn {
        score += 0.25;
    }
    if input.is_botnet {
        score += 0.30;
    }

    if let Some(abuse) = input.abuse_score {
        if abuse >= 80 {
            score += 0.25;
        } else if abuse >= 50 {
            score += 0.15;
        } else if abuse >= 20 {
            score += 0.05;
        }
    }

    if let Some(ref class) = input.greynoise_class {
        match class.as_str() {
            "malicious" => score += 0.30,
            "benign" => score -= 0.10,
            _ => {}
        }
    }

    if input.ip_type == crate::classifier::IpType::Hosting {
        score += 0.05;
    }

    score = score.clamp(0.0, 1.0);

    let verdict = if score < 0.15 {
        RiskVerdict::None
    } else if score < 0.35 {
        RiskVerdict::Low
    } else if score < 0.55 {
        RiskVerdict::Medium
    } else if score < 0.75 {
        RiskVerdict::High
    } else {
        RiskVerdict::Critical
    };

    (score, verdict)
}
