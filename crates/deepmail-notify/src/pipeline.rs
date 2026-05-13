use crate::error::NotifyError;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize)]
pub struct NotifyEvent {
    pub email_id: Uuid,
    pub tenant_id: Uuid,
    pub event_type: String,
    pub verdict: String,
    pub score: f32,
    pub details: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

#[derive(serde::Deserialize)]
struct ScoringPayload {
    email_id: Uuid,
    tenant_id: Uuid,
    #[serde(default)]
    final_verdict: String,
    #[serde(default)]
    final_score: f32,
}

#[derive(serde::Deserialize)]
struct ReportPayload {
    email_id: Uuid,
    tenant_id: Uuid,
    #[serde(default)]
    final_verdict: String,
    #[serde(default)]
    final_score: f32,
    #[serde(default)]
    json_s3_key: String,
    #[serde(default)]
    html_s3_key: String,
}

#[derive(serde::Deserialize)]
struct IocPayload {
    email_id: Uuid,
    tenant_id: Uuid,
    #[serde(default)]
    malicious_count: u32,
    #[serde(default)]
    total_count: u32,
}

#[derive(serde::Deserialize)]
struct DynamicPayload {
    email_id: Uuid,
    tenant_id: Uuid,
    #[serde(default)]
    verdict: String,
    #[serde(default)]
    risk_score: f32,
}

pub fn parse_scoring_event(payload: &[u8]) -> Result<NotifyEvent, NotifyError> {
    let p: ScoringPayload =
        serde_json::from_slice(payload).map_err(|e| NotifyError::PayloadParse(e.to_string()))?;

    let details = serde_json::json!({
        "final_verdict": &p.final_verdict,
        "final_score": p.final_score,
    });

    Ok(NotifyEvent {
        email_id: p.email_id,
        tenant_id: p.tenant_id,
        event_type: "scoring.completed".to_string(),
        verdict: p.final_verdict,
        score: p.final_score,
        details,
        timestamp: Utc::now(),
    })
}

pub fn parse_report_event(payload: &[u8]) -> Result<NotifyEvent, NotifyError> {
    let p: ReportPayload =
        serde_json::from_slice(payload).map_err(|e| NotifyError::PayloadParse(e.to_string()))?;

    let details = serde_json::json!({
        "json_s3_key": &p.json_s3_key,
        "html_s3_key": &p.html_s3_key,
    });

    Ok(NotifyEvent {
        email_id: p.email_id,
        tenant_id: p.tenant_id,
        event_type: "report.completed".to_string(),
        verdict: p.final_verdict,
        score: p.final_score,
        details,
        timestamp: Utc::now(),
    })
}

pub fn parse_ioc_event(payload: &[u8]) -> Option<NotifyEvent> {
    let p: IocPayload = serde_json::from_slice(payload).ok()?;

    if p.malicious_count == 0 {
        return None;
    }

    Some(NotifyEvent {
        email_id: p.email_id,
        tenant_id: p.tenant_id,
        event_type: "ioc.completed".to_string(),
        verdict: "MALICIOUS".to_string(),
        score: 0.0,
        details: serde_json::json!({
            "malicious_count": p.malicious_count,
            "total_count": p.total_count,
        }),
        timestamp: Utc::now(),
    })
}

pub fn parse_dynamic_event(payload: &[u8]) -> Option<NotifyEvent> {
    let p: DynamicPayload = serde_json::from_slice(payload).ok()?;

    if p.verdict != "MALWARE" {
        return None;
    }

    Some(NotifyEvent {
        email_id: p.email_id,
        tenant_id: p.tenant_id,
        event_type: "sandbox.dynamic.completed".to_string(),
        verdict: p.verdict,
        score: p.risk_score,
        details: serde_json::json!({
            "risk_score": p.risk_score,
        }),
        timestamp: Utc::now(),
    })
}

pub fn severity_rank(verdict: &str) -> u8 {
    match verdict {
        "CLEAN" => 0,
        "LOW_RISK" => 1,
        "SUSPICIOUS" => 2,
        "PHISHING" => 3,
        "MALICIOUS" | "MALWARE" => 4,
        _ => 2,
    }
}
