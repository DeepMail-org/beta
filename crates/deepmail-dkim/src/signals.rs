use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalScores {
    pub body_hash_mismatch: f32,
    pub timestamp_drift: f32,
    pub received_before_signed: f32,
    pub key_rotated_since: f32,
    pub excessive_replay_gap: f32,
    pub header_order_tampered: f32,
}

const WEIGHT_BODY_HASH_MISMATCH: f32 = 0.30;
const WEIGHT_TIMESTAMP_DRIFT: f32 = 0.20;
const WEIGHT_RECEIVED_BEFORE_SIGNED: f32 = 0.20;
const WEIGHT_KEY_ROTATED_SINCE: f32 = 0.15;
const WEIGHT_EXCESSIVE_REPLAY_GAP: f32 = 0.10;
const WEIGHT_HEADER_ORDER_TAMPERED: f32 = 0.05;

pub struct SignalInput {
    pub body_hash_matches: bool,
    pub signature_timestamp: Option<DateTime<Utc>>,
    pub date_header: Option<DateTime<Utc>>,
    pub received_time: Option<DateTime<Utc>>,
    pub key_found_in_dns: bool,
    pub signed_headers: Vec<String>,
    pub actual_headers_present: Vec<String>,
}

pub fn evaluate_signals(input: &SignalInput) -> SignalScores {
    let body_hash_mismatch = if input.body_hash_matches { 0.0 } else { 1.0 };

    let timestamp_drift = evaluate_timestamp_drift(
        input.signature_timestamp,
        input.date_header,
    );

    let received_before_signed = evaluate_received_before_signed(
        input.signature_timestamp,
        input.received_time,
    );

    let key_rotated_since = if input.key_found_in_dns { 0.0 } else { 1.0 };

    let excessive_replay_gap = evaluate_replay_gap(
        input.signature_timestamp,
        input.received_time,
    );

    let header_order_tampered = evaluate_header_order(
        &input.signed_headers,
        &input.actual_headers_present,
    );

    SignalScores {
        body_hash_mismatch,
        timestamp_drift,
        received_before_signed,
        key_rotated_since,
        excessive_replay_gap,
        header_order_tampered,
    }
}

pub fn compute_confidence(scores: &SignalScores) -> f32 {
    let weighted_sum = scores.body_hash_mismatch * WEIGHT_BODY_HASH_MISMATCH
        + scores.timestamp_drift * WEIGHT_TIMESTAMP_DRIFT
        + scores.received_before_signed * WEIGHT_RECEIVED_BEFORE_SIGNED
        + scores.key_rotated_since * WEIGHT_KEY_ROTATED_SINCE
        + scores.excessive_replay_gap * WEIGHT_EXCESSIVE_REPLAY_GAP
        + scores.header_order_tampered * WEIGHT_HEADER_ORDER_TAMPERED;

    weighted_sum.clamp(0.0, 1.0)
}

pub fn confidence_to_verdict(confidence: f32) -> &'static str {
    if confidence >= 0.8 {
        "CRITICAL"
    } else if confidence >= 0.6 {
        "HIGH"
    } else if confidence >= 0.4 {
        "MEDIUM"
    } else if confidence >= 0.2 {
        "LOW"
    } else {
        "NONE"
    }
}

fn evaluate_timestamp_drift(
    sig_ts: Option<DateTime<Utc>>,
    date_header: Option<DateTime<Utc>>,
) -> f32 {
    match (sig_ts, date_header) {
        (Some(t), Some(d)) => {
            let drift_seconds = (t - d).num_seconds().unsigned_abs();
            if drift_seconds > 300 {
                (drift_seconds as f32 / 3600.0).min(1.0)
            } else {
                0.0
            }
        }
        _ => 0.0,
    }
}

fn evaluate_received_before_signed(
    sig_ts: Option<DateTime<Utc>>,
    received_time: Option<DateTime<Utc>>,
) -> f32 {
    match (sig_ts, received_time) {
        (Some(t), Some(r)) => {
            if r < t { 1.0 } else { 0.0 }
        }
        _ => 0.0,
    }
}

fn evaluate_replay_gap(
    sig_ts: Option<DateTime<Utc>>,
    received_time: Option<DateTime<Utc>>,
) -> f32 {
    match (sig_ts, received_time) {
        (Some(t), Some(r)) => {
            let gap_seconds = (r - t).num_seconds();
            if gap_seconds > 3600 {
                (gap_seconds as f32 / 86400.0).min(1.0)
            } else {
                0.0
            }
        }
        _ => 0.0,
    }
}

fn evaluate_header_order(
    signed_headers: &[String],
    actual_headers: &[String],
) -> f32 {
    if signed_headers.is_empty() {
        return 0.0;
    }

    let actual_lower: Vec<String> = actual_headers
        .iter()
        .map(|h| h.to_lowercase())
        .collect();

    let missing_count = signed_headers
        .iter()
        .filter(|h| !actual_lower.contains(&h.to_lowercase()))
        .count();

    if missing_count == 0 {
        return 0.0;
    }

    (missing_count as f32 / signed_headers.len() as f32).min(1.0)
}
