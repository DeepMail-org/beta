use crate::error::ScoringError;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignalName {
    Header,
    Dkim,
    Geo,
    Ip,
    Intel,
    Ioc,
    Homograph,
    Body,
    SandboxUrl,
    SandboxFile,
    SandboxDynamic,
}

impl SignalName {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Header => "header",
            Self::Dkim => "dkim",
            Self::Geo => "geo",
            Self::Ip => "ip",
            Self::Intel => "intel",
            Self::Ioc => "ioc",
            Self::Homograph => "homograph",
            Self::Body => "body",
            Self::SandboxUrl => "sandbox_url",
            Self::SandboxFile => "sandbox_file",
            Self::SandboxDynamic => "sandbox_dynamic",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "header" => Some(Self::Header),
            "dkim" => Some(Self::Dkim),
            "geo" => Some(Self::Geo),
            "ip" => Some(Self::Ip),
            "intel" => Some(Self::Intel),
            "ioc" => Some(Self::Ioc),
            "homograph" => Some(Self::Homograph),
            "body" => Some(Self::Body),
            "sandbox_url" => Some(Self::SandboxUrl),
            "sandbox_file" => Some(Self::SandboxFile),
            "sandbox_dynamic" => Some(Self::SandboxDynamic),
            _ => None,
        }
    }

    pub fn weight(&self) -> f32 {
        match self {
            Self::Header => 0.12,
            Self::Dkim => 0.08,
            Self::Geo => 0.08,
            Self::Ip => 0.10,
            Self::Intel => 0.10,
            Self::Ioc => 0.10,
            Self::Homograph => 0.08,
            Self::Body => 0.14,
            Self::SandboxUrl => 0.10,
            Self::SandboxFile => 0.10,
            Self::SandboxDynamic => 0.14,
        }
    }

    pub fn all() -> &'static [SignalName] {
        &[
            Self::Header,
            Self::Dkim,
            Self::Geo,
            Self::Ip,
            Self::Intel,
            Self::Ioc,
            Self::Homograph,
            Self::Body,
            Self::SandboxUrl,
            Self::SandboxFile,
            Self::SandboxDynamic,
        ]
    }
}

pub const NATS_SUBJECT_MAP: &[(&str, SignalName)] = &[
    ("deepmail.events.header.completed", SignalName::Header),
    ("deepmail.events.dkim.completed", SignalName::Dkim),
    ("deepmail.events.geo.completed", SignalName::Geo),
    ("deepmail.events.ip.completed", SignalName::Ip),
    ("deepmail.events.intel.completed", SignalName::Intel),
    ("deepmail.events.ioc.completed", SignalName::Ioc),
    ("deepmail.events.homograph.completed", SignalName::Homograph),
    ("deepmail.events.body.completed", SignalName::Body),
    ("deepmail.events.sandbox.url.completed", SignalName::SandboxUrl),
    ("deepmail.events.sandbox.file.completed", SignalName::SandboxFile),
    (
        "deepmail.events.sandbox.dynamic.completed",
        SignalName::SandboxDynamic,
    ),
];

pub fn subject_to_signal(subject: &str) -> Option<SignalName> {
    NATS_SUBJECT_MAP
        .iter()
        .find(|(s, _)| *s == subject)
        .map(|(_, sig)| *sig)
}

pub fn risk_text_to_float(risk: &str) -> f32 {
    match risk {
        "NONE" => 0.0,
        "LOW" => 0.25,
        "MEDIUM" => 0.50,
        "HIGH" => 0.75,
        "CRITICAL" => 1.0,
        _ => 0.0,
    }
}

pub struct EventPayload {
    pub email_id: Uuid,
    pub tenant_id: Uuid,
    pub score: Option<f32>,
}

pub fn parse_event_payload(data: &[u8]) -> Result<EventPayload, ScoringError> {
    let v: serde_json::Value =
        serde_json::from_slice(data).map_err(|e| ScoringError::PayloadParse(e.to_string()))?;

    let email_id = v
        .get("email_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| ScoringError::PayloadParse("missing email_id".into()))?;

    let tenant_id = v
        .get("tenant_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| ScoringError::PayloadParse("missing tenant_id".into()))?;

    let score = v
        .get("score")
        .and_then(|v| v.as_f64())
        .map(|f| f as f32)
        .or_else(|| {
            v.get("final_phishing_score")
                .and_then(|v| v.as_f64())
                .map(|f| f as f32)
        })
        .or_else(|| {
            v.get("dynamic_score")
                .and_then(|v| v.as_f64())
                .map(|f| f as f32)
        })
        .or_else(|| {
            v.get("threat_score")
                .and_then(|v| v.as_f64())
                .map(|f| f as f32)
        })
        .or_else(|| {
            v.get("replay_confidence")
                .and_then(|v| v.as_f64())
                .map(|f| f as f32)
        });

    Ok(EventPayload {
        email_id,
        tenant_id,
        score,
    })
}
