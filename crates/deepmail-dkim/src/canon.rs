use crate::parse::DkimSignature;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CanonicalizationType {
    Simple,
    Relaxed,
}

pub fn parse_canonicalization(c: &str) -> (CanonicalizationType, CanonicalizationType) {
    let parts: Vec<&str> = c.split('/').collect();
    let header_canon = match parts.first().map(|s| s.to_lowercase()).as_deref() {
        Some("relaxed") => CanonicalizationType::Relaxed,
        _ => CanonicalizationType::Simple,
    };
    let body_canon = match parts.get(1).map(|s| s.to_lowercase()).as_deref() {
        Some("relaxed") => CanonicalizationType::Relaxed,
        Some("simple") => CanonicalizationType::Simple,
        _ => header_canon,
    };
    (header_canon, body_canon)
}

pub fn canonicalize_body(body: &[u8], canon_type: CanonicalizationType) -> Vec<u8> {
    match canon_type {
        CanonicalizationType::Simple => canonicalize_body_simple(body),
        CanonicalizationType::Relaxed => canonicalize_body_relaxed(body),
    }
}

fn canonicalize_body_simple(body: &[u8]) -> Vec<u8> {
    if body.is_empty() {
        return b"\r\n".to_vec();
    }

    let mut result = normalize_line_endings(body);

    while result.ends_with(b"\r\n\r\n") {
        result.truncate(result.len() - 2);
    }

    if !result.ends_with(b"\r\n") {
        result.extend_from_slice(b"\r\n");
    }

    result
}

fn canonicalize_body_relaxed(body: &[u8]) -> Vec<u8> {
    if body.is_empty() {
        return b"\r\n".to_vec();
    }

    let text = String::from_utf8_lossy(body);
    let mut lines: Vec<String> = Vec::new();

    for line in text.lines() {
        let mut processed = String::new();
        let mut last_was_ws = false;

        for ch in line.chars() {
            if ch == ' ' || ch == '\t' {
                last_was_ws = true;
            } else {
                if last_was_ws {
                    processed.push(' ');
                    last_was_ws = false;
                }
                processed.push(ch);
            }
        }

        lines.push(processed);
    }

    while lines.last().map_or(false, |l| l.is_empty()) {
        lines.pop();
    }

    if lines.is_empty() {
        return b"\r\n".to_vec();
    }

    let mut result = Vec::new();
    for line in &lines {
        result.extend_from_slice(line.as_bytes());
        result.extend_from_slice(b"\r\n");
    }
    result
}

pub fn canonicalize_headers(
    raw_email: &[u8],
    sig: &DkimSignature,
    canon_type: CanonicalizationType,
) -> Vec<u8> {
    let email_str = String::from_utf8_lossy(raw_email);
    let header_section = email_str.split("\r\n\r\n").next()
        .or_else(|| email_str.split("\n\n").next())
        .unwrap_or(&email_str);

    let parsed_headers = parse_raw_headers(header_section);

    let mut result = Vec::new();
    let mut header_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for signed_name in &sig.signed_headers {
        let lower_name = signed_name.to_lowercase();
        let count = header_counts.entry(lower_name.clone()).or_insert(0);

        let matching: Vec<&(String, String)> = parsed_headers
            .iter()
            .filter(|(name, _)| name.to_lowercase() == lower_name)
            .collect();

        let idx = matching.len().saturating_sub(*count + 1);
        if let Some((name, value)) = matching.get(idx) {
            let canonical = canonicalize_single_header(name, value, canon_type);
            result.extend_from_slice(canonical.as_bytes());
            result.extend_from_slice(b"\r\n");
        }
        *count += 1;
    }

    let dkim_header_canonical = build_dkim_header_for_signing(sig, canon_type);
    result.extend_from_slice(dkim_header_canonical.as_bytes());

    result
}

fn canonicalize_single_header(
    name: &str,
    value: &str,
    canon_type: CanonicalizationType,
) -> String {
    match canon_type {
        CanonicalizationType::Simple => {
            format!("{name}:{value}")
        }
        CanonicalizationType::Relaxed => {
            let lower_name = name.to_lowercase();
            let lower_name = lower_name.trim();
            let normalized_value = normalize_header_value(value);
            format!("{lower_name}:{normalized_value}")
        }
    }
}

fn normalize_header_value(value: &str) -> String {
    let unfolded = value.replace("\r\n", "").replace('\n', "");
    let mut result = String::new();
    let mut last_was_ws = false;
    let trimmed = unfolded.trim();

    for ch in trimmed.chars() {
        if ch == ' ' || ch == '\t' {
            last_was_ws = true;
        } else {
            if last_was_ws && !result.is_empty() {
                result.push(' ');
            }
            last_was_ws = false;
            result.push(ch);
        }
    }

    result
}

fn build_dkim_header_for_signing(sig: &DkimSignature, canon_type: CanonicalizationType) -> String {
    let raw = &sig.raw_header;
    let without_b = remove_b_value(raw);
    let full_header = format!("DKIM-Signature:{without_b}");
    match canon_type {
        CanonicalizationType::Simple => full_header,
        CanonicalizationType::Relaxed => {
            let parts: Vec<&str> = full_header.splitn(2, ':').collect();
            if parts.len() == 2 {
                let lower_name = parts[0].to_lowercase();
                let normalized_value = normalize_header_value(parts[1]);
                format!("{lower_name}:{normalized_value}")
            } else {
                full_header
            }
        }
    }
}

fn remove_b_value(header: &str) -> String {
    let mut result = String::new();
    let mut remaining = header;

    while let Some(b_pos) = remaining.find("b=") {
        let before_b = &remaining[..b_pos];
        if before_b.ends_with("bh=") || before_b.trim_end().ends_with("bh") {
            result.push_str(&remaining[..b_pos + 2]);
            remaining = &remaining[b_pos + 2..];
            continue;
        }
        let prev_ok = b_pos == 0 || {
            let prev_char = remaining.as_bytes()[b_pos - 1];
            prev_char == b';' || prev_char == b' ' || prev_char == b'\t'
        };
        if prev_ok {
            result.push_str(before_b);
            result.push_str("b=");
            let after_eq = &remaining[b_pos + 2..];
            if let Some(semi_pos) = after_eq.find(';') {
                remaining = &after_eq[semi_pos..];
            } else {
                remaining = "";
            }
        } else {
            result.push_str(&remaining[..b_pos + 2]);
            remaining = &remaining[b_pos + 2..];
        }
    }
    result.push_str(remaining);
    result
}

fn parse_raw_headers(header_section: &str) -> Vec<(String, String)> {
    let mut headers = Vec::new();
    let mut current_name = String::new();
    let mut current_value = String::new();

    for line in header_section.lines() {
        if line.starts_with(' ') || line.starts_with('\t') {
            if !current_name.is_empty() {
                current_value.push('\n');
                current_value.push_str(line);
            }
        } else if let Some(colon_pos) = line.find(':') {
            if !current_name.is_empty() {
                headers.push((current_name.clone(), current_value.clone()));
            }
            current_name = line[..colon_pos].to_string();
            current_value = line[colon_pos + 1..].to_string();
        }
    }

    if !current_name.is_empty() {
        headers.push((current_name, current_value));
    }

    headers
}

fn normalize_line_endings(data: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(data.len());
    let mut i = 0;
    while i < data.len() {
        if data[i] == b'\r' {
            if i + 1 < data.len() && data[i + 1] == b'\n' {
                result.push(b'\r');
                result.push(b'\n');
                i += 2;
            } else {
                result.push(b'\r');
                result.push(b'\n');
                i += 1;
            }
        } else if data[i] == b'\n' {
            result.push(b'\r');
            result.push(b'\n');
            i += 1;
        } else {
            result.push(data[i]);
            i += 1;
        }
    }
    result
}
