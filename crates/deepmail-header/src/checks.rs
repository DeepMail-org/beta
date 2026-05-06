//! Forensic header analysis checks.
//!
//! 8 dedicated checks performed on parsed email header data:
//! 1. Return-Path vs. From domain mismatch
//! 2. Reply-To vs. From domain mismatch
//! 3. Message-ID domain validation
//! 4. Message-ID fake pattern detection
//! 5. X-Mailer / User-Agent fingerprinting
//! 6. Time anomaly detection (future-dated, suspicious delays)
//! 7. Received-hop analysis
//! 8. Sender spoofing likelihood

use chrono::{DateTime, Utc};

/// Container for all header check outputs.
#[derive(Debug, Clone)]
pub struct ChecksOutput {
    pub return_path_mismatch: bool,
    pub return_path_domain: Option<String>,
    pub reply_to_mismatch: bool,
    pub reply_to_domain: Option<String>,
    pub from_domain: Option<String>,
    pub message_id_valid: bool,
    pub message_id_domain: Option<String>,
    pub message_id_domain_match: bool,
    pub message_id_fake_pattern: bool,
    pub x_mailer: Option<String>,
    pub x_mailer_risk: Option<String>,
    pub x_mailer_known_kit: Option<String>,
    pub time_anomaly: TimeAnomalyOutput,
    pub sender_spoofing_likely: bool,
    pub findings: Vec<Finding>,
}

/// Time anomaly analysis output.
#[derive(Debug, Clone, Default)]
pub struct TimeAnomalyOutput {
    pub future_dated: bool,
    pub date_predates_received: bool,
    pub suspicious_delay: bool,
    pub max_hop_delay_sec: i32,
    pub total_transit_sec: i32,
    pub hop_count: i32,
    pub date_header_ts: Option<DateTime<Utc>>,
    pub earliest_received_ts: Option<DateTime<Utc>>,
    pub latest_received_ts: Option<DateTime<Utc>>,
}

/// Individual finding entry.
#[derive(Debug, Clone)]
pub struct Finding {
    pub check_name: String,
    pub severity: String,
    pub passed: bool,
    pub evidence: String,
    pub raw_value: Option<String>,
    pub risk_weight: f32,
}

/// Input data extracted from the parser database.
pub struct HeaderInput {
    pub from_address: Option<String>,
    pub reply_to: Option<String>,
    pub message_id: Option<String>,
    pub date_sent: Option<DateTime<Utc>>,
    pub headers: Vec<(String, String)>,
    pub received_hops: Vec<ReceivedHop>,
}

/// Received hop from the parser DB.
pub struct ReceivedHop {
    pub hop_index: i32,
    pub from_host: Option<String>,
    pub by_host: Option<String>,
    pub received_at: Option<DateTime<Utc>>,
    pub raw_value: String,
}

/// MTA fingerprint from the mta_fingerprints table.
pub struct MtaFingerprint {
    pub pattern: String,
    pub match_type: String,
    pub kit_name: String,
    pub risk_level: String,
}

/// Run all 8 header forensic checks.
pub fn run_all_checks(
    input: &HeaderInput,
    mta_fingerprints: &[MtaFingerprint],
) -> ChecksOutput {
    let mut findings = Vec::new();

    // -- Extract domains from headers --
    let from_domain = input.from_address.as_deref().and_then(extract_domain);

    let return_path = input.headers.iter()
        .find(|(n, _)| n.eq_ignore_ascii_case("Return-Path"))
        .map(|(_, v)| v.as_str());
    let return_path_domain = return_path.and_then(extract_domain);

    let reply_to_domain = input.reply_to.as_deref().and_then(extract_domain);

    let x_mailer_value = input.headers.iter()
        .find(|(n, _)| n.eq_ignore_ascii_case("X-Mailer") || n.eq_ignore_ascii_case("User-Agent"))
        .map(|(_, v)| v.clone());

    // ── Check 1: Return-Path vs From ──────────────────────────────
    let return_path_mismatch = match (&from_domain, &return_path_domain) {
        (Some(from_d), Some(rp_d)) => {
            let m = !domains_match(from_d, rp_d);
            findings.push(Finding {
                check_name: "return_path_mismatch".to_string(),
                severity: if m { "medium".to_string() } else { "info".to_string() },
                passed: !m,
                evidence: if m {
                    format!("Return-Path domain '{}' ≠ From domain '{}'", rp_d, from_d)
                } else {
                    "Return-Path domain matches From domain".to_string()
                },
                raw_value: return_path.map(String::from),
                risk_weight: if m { 0.15 } else { 0.0 },
            });
            m
        }
        _ => {
            findings.push(Finding {
                check_name: "return_path_mismatch".to_string(),
                severity: "info".to_string(),
                passed: true,
                evidence: "Return-Path or From header not present".to_string(),
                raw_value: None,
                risk_weight: 0.0,
            });
            false
        }
    };

    // ── Check 2: Reply-To vs From ─────────────────────────────────
    let reply_to_mismatch = match (&from_domain, &reply_to_domain) {
        (Some(from_d), Some(rt_d)) => {
            let m = !domains_match(from_d, rt_d);
            findings.push(Finding {
                check_name: "reply_to_mismatch".to_string(),
                severity: if m { "medium".to_string() } else { "info".to_string() },
                passed: !m,
                evidence: if m {
                    format!("Reply-To domain '{}' ≠ From domain '{}'", rt_d, from_d)
                } else {
                    "Reply-To domain matches From domain".to_string()
                },
                raw_value: input.reply_to.clone(),
                risk_weight: if m { 0.15 } else { 0.0 },
            });
            m
        }
        _ => false,
    };

    // ── Check 3 & 4: Message-ID validation ────────────────────────
    let (message_id_valid, message_id_domain, message_id_domain_match, message_id_fake_pattern) =
        check_message_id(input.message_id.as_deref(), from_domain.as_deref(), &mut findings);

    // ── Check 5: X-Mailer fingerprinting ──────────────────────────
    let (x_mailer_risk, x_mailer_known_kit) =
        check_x_mailer(x_mailer_value.as_deref(), mta_fingerprints, &mut findings);

    // ── Check 6 & 7: Time anomaly & received hop analysis ─────────
    let time_anomaly = check_time_anomaly(input, &mut findings);

    // ── Check 8: Sender spoofing likelihood ───────────────────────
    let spoofing_signals: u8 = u8::from(return_path_mismatch)
        + u8::from(reply_to_mismatch)
        + u8::from(message_id_fake_pattern)
        + u8::from(x_mailer_risk.as_deref() == Some("high") || x_mailer_risk.as_deref() == Some("critical"))
        + u8::from(time_anomaly.suspicious_delay);

    let sender_spoofing_likely = spoofing_signals >= 2;
    findings.push(Finding {
        check_name: "sender_spoofing".to_string(),
        severity: if sender_spoofing_likely { "high".to_string() } else { "info".to_string() },
        passed: !sender_spoofing_likely,
        evidence: format!("{} of 5 spoofing indicators triggered", spoofing_signals),
        raw_value: None,
        risk_weight: if sender_spoofing_likely { 0.25 } else { 0.0 },
    });

    ChecksOutput {
        return_path_mismatch,
        return_path_domain: return_path_domain.map(String::from),
        reply_to_mismatch,
        reply_to_domain: reply_to_domain.map(String::from),
        from_domain: from_domain.map(String::from),
        message_id_valid,
        message_id_domain: message_id_domain.map(String::from),
        message_id_domain_match,
        message_id_fake_pattern,
        x_mailer: x_mailer_value,
        x_mailer_risk,
        x_mailer_known_kit,
        time_anomaly,
        sender_spoofing_likely,
        findings,
    }
}

// ── Message-ID checks ────────────────────────────────────────────

fn check_message_id(
    message_id: Option<&str>,
    from_domain: Option<&str>,
    findings: &mut Vec<Finding>,
) -> (bool, Option<String>, bool, bool) {
    let mid = match message_id {
        Some(m) if !m.is_empty() => m,
        _ => {
            findings.push(Finding {
                check_name: "message_id_fake".to_string(),
                severity: "low".to_string(),
                passed: true,
                evidence: "No Message-ID header present".to_string(),
                raw_value: None,
                risk_weight: 0.0,
            });
            return (false, None, false, false);
        }
    };

    // Strip angle brackets
    let cleaned = mid.trim_start_matches('<').trim_end_matches('>');

    // Valid format: local@domain
    let valid = cleaned.contains('@') && !cleaned.contains(' ');

    // Extract domain from Message-ID
    let mid_domain = cleaned.rsplit('@').next().map(|d| d.to_lowercase());

    // Check domain match
    let domain_match = match (&mid_domain, from_domain) {
        (Some(md), Some(fd)) => domains_match(md, fd),
        _ => false,
    };

    // Detect fake patterns: very short IDs, hex-only local parts,
    // localhost/invalid domains
    let fake_patterns = [
        "localhost",
        "invalid",
        "local",
        ".internal",
        "example.com",
        "fake",
    ];
    let fake_domain = mid_domain.as_ref().map_or(false, |d| {
        fake_patterns.iter().any(|fp| d.contains(fp))
    });

    let local_part = cleaned.split('@').next().unwrap_or("");
    let suspiciously_short = local_part.len() < 4;
    let all_hex = !local_part.is_empty()
        && local_part.chars().all(|c| c.is_ascii_hexdigit() || c == '.');
    let fake_pattern = fake_domain || (suspiciously_short && all_hex);

    let severity = if fake_pattern {
        "high"
    } else if !domain_match {
        "medium"
    } else {
        "info"
    };

    findings.push(Finding {
        check_name: "message_id_fake".to_string(),
        severity: severity.to_string(),
        passed: !fake_pattern,
        evidence: if fake_pattern {
            format!("Suspicious Message-ID pattern: {cleaned}")
        } else if !domain_match {
            format!(
                "Message-ID domain '{}' does not match From domain",
                mid_domain.as_deref().unwrap_or("(none)")
            )
        } else {
            "Message-ID appears legitimate".to_string()
        },
        raw_value: Some(mid.to_string()),
        risk_weight: if fake_pattern { 0.2 } else if !domain_match { 0.1 } else { 0.0 },
    });

    (valid, mid_domain, domain_match, fake_pattern)
}

// ── X-Mailer fingerprinting ─────────────────────────────────────

fn check_x_mailer(
    x_mailer: Option<&str>,
    fingerprints: &[MtaFingerprint],
    findings: &mut Vec<Finding>,
) -> (Option<String>, Option<String>) {
    let mailer = match x_mailer {
        Some(m) if !m.is_empty() => m,
        _ => return (None, None),
    };

    let mailer_lower = mailer.to_lowercase();

    for fp in fingerprints {
        let matched = match fp.match_type.as_str() {
            "exact" => mailer_lower == fp.pattern.to_lowercase(),
            "contains" => mailer_lower.contains(&fp.pattern.to_lowercase()),
            "regex" => {
                // Regex matching (best-effort without regex crate)
                mailer_lower.contains(&fp.pattern.to_lowercase())
            }
            _ => false,
        };

        if matched {
            findings.push(Finding {
                check_name: "x_mailer_kit".to_string(),
                severity: fp.risk_level.clone(),
                passed: fp.risk_level == "none" || fp.risk_level == "low",
                evidence: format!(
                    "X-Mailer '{}' matches known kit '{}' (risk: {})",
                    mailer, fp.kit_name, fp.risk_level
                ),
                raw_value: Some(mailer.to_string()),
                risk_weight: match fp.risk_level.as_str() {
                    "critical" => 0.3,
                    "high"     => 0.2,
                    "medium"   => 0.1,
                    "low"      => 0.05,
                    _          => 0.0,
                },
            });

            return (Some(fp.risk_level.clone()), Some(fp.kit_name.clone()));
        }
    }

    findings.push(Finding {
        check_name: "x_mailer_kit".to_string(),
        severity: "info".to_string(),
        passed: true,
        evidence: format!("X-Mailer '{}' is not in the known-kit database", mailer),
        raw_value: Some(mailer.to_string()),
        risk_weight: 0.0,
    });

    (Some("none".to_string()), None)
}

// ── Time anomaly analysis ────────────────────────────────────────

fn check_time_anomaly(input: &HeaderInput, findings: &mut Vec<Finding>) -> TimeAnomalyOutput {
    let now = Utc::now();
    let mut output = TimeAnomalyOutput::default();
    output.hop_count = input.received_hops.len() as i32;

    // Sort hops by timestamp where available
    let mut hop_times: Vec<DateTime<Utc>> = input
        .received_hops
        .iter()
        .filter_map(|h| h.received_at)
        .collect();
    hop_times.sort();

    if !hop_times.is_empty() {
        output.earliest_received_ts = hop_times.first().copied();
        output.latest_received_ts = hop_times.last().copied();
    }

    // Check Date header
    if let Some(date_ts) = input.date_sent {
        output.date_header_ts = Some(date_ts);

        // Future-dated check (more than 5 minutes ahead)
        if date_ts > now + chrono::Duration::minutes(5) {
            output.future_dated = true;
        }

        // Date predates first Received header
        if let Some(earliest) = output.earliest_received_ts {
            if date_ts > earliest + chrono::Duration::hours(1) {
                output.date_predates_received = true;
            }
        }
    }

    // Hop delay analysis
    if hop_times.len() >= 2 {
        let total = (hop_times[hop_times.len() - 1] - hop_times[0])
            .num_seconds()
            .max(0) as i32;
        output.total_transit_sec = total;

        let mut max_delay: i32 = 0;
        for window in hop_times.windows(2) {
            let delay = (window[1] - window[0]).num_seconds().max(0) as i32;
            if delay > max_delay {
                max_delay = delay;
            }
        }
        output.max_hop_delay_sec = max_delay;

        // Suspicious if any single hop > 30 min or total transit > 2 hours
        if max_delay > 1800 || total > 7200 {
            output.suspicious_delay = true;
        }
    }

    // Build time anomaly finding
    let has_anomaly = output.future_dated
        || output.date_predates_received
        || output.suspicious_delay;

    findings.push(Finding {
        check_name: "time_anomaly".to_string(),
        severity: if output.future_dated {
            "high".to_string()
        } else if has_anomaly {
            "medium".to_string()
        } else {
            "info".to_string()
        },
        passed: !has_anomaly,
        evidence: if output.future_dated {
            format!(
                "Date header is set in the future ({} hops, max delay {} sec)",
                output.hop_count, output.max_hop_delay_sec
            )
        } else if output.suspicious_delay {
            format!(
                "Suspicious transit delay: total {} sec, max single-hop {} sec across {} hops",
                output.total_transit_sec, output.max_hop_delay_sec, output.hop_count
            )
        } else {
            format!(
                "Normal transit: {} sec total across {} hops",
                output.total_transit_sec, output.hop_count
            )
        },
        raw_value: None,
        risk_weight: if output.future_dated { 0.2 } else if has_anomaly { 0.1 } else { 0.0 },
    });

    output
}

// ── Utility functions ────────────────────────────────────────────

/// Extract domain from an email address or angle-bracket address.
fn extract_domain(addr: &str) -> Option<String> {
    // Handle "<user@domain>" and "Display Name <user@domain>"
    let cleaned = if addr.contains('<') && addr.contains('>') {
        let start = addr.find('<').unwrap_or(0) + 1;
        let end = addr.find('>').unwrap_or(addr.len());
        &addr[start..end]
    } else {
        addr
    };
    let cleaned = cleaned.trim();
    let at_pos = cleaned.rfind('@')?;
    let domain = &cleaned[at_pos + 1..];
    if domain.is_empty() {
        None
    } else {
        Some(domain.to_lowercase())
    }
}

/// Relaxed domain comparison (organizational domain level).
fn domains_match(a: &str, b: &str) -> bool {
    let a_lower = a.to_lowercase();
    let b_lower = b.to_lowercase();
    if a_lower == b_lower {
        return true;
    }
    // Check organizational domain (last 2 labels)
    let org_a = org_domain(&a_lower);
    let org_b = org_domain(&b_lower);
    org_a == org_b
}

fn org_domain(domain: &str) -> String {
    let labels: Vec<&str> = domain.rsplitn(3, '.').collect();
    if labels.len() >= 2 {
        format!("{}.{}", labels[1], labels[0])
    } else {
        domain.to_string()
    }
}
