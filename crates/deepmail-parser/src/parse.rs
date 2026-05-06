//! RFC 5322 / MIME email parsing logic.
//!
//! Wraps `mail-parser` 0.9 to extract structured fields, headers, Received
//! hops, and attachment metadata from raw email bytes.

use mail_parser::{Address, HeaderName, HeaderValue, Host, MessageParser, MimeHeaders};
use sha2::{Digest, Sha256};

use crate::db::insert::{
    AttachmentRow, HeaderRow, ParsedEmailRow, ReceivedHopRow,
};
use crate::error::ParserError;

/// All data extracted from a single email file.
pub struct ParsedEmail {
    pub email_row: ParsedEmailRow,
    pub headers: Vec<HeaderRow>,
    pub received_hops: Vec<ReceivedHopRow>,
    pub attachments: Vec<AttachmentData>,
}

/// Attachment data ready for S3 upload + DB insert.
pub struct AttachmentData {
    pub row: AttachmentRow,
    pub bytes: Vec<u8>,
}

/// Parse raw email bytes and produce structured output.
///
/// This function is **CPU-bound** for large messages; callers should
/// wrap it in `spawn_blocking` if running on a multi-task executor.
pub fn parse_email(
    raw: &[u8],
    email_id: uuid::Uuid,
    tenant_id: uuid::Uuid,
    s3_bucket: &str,
) -> Result<ParsedEmail, ParserError> {
    let message = MessageParser::default()
        .parse(raw)
        .ok_or_else(|| ParserError::EmailParse("mail-parser returned None".into()))?;

    // ── Envelope fields ──────────────────────────────────────────
    let message_id = message.message_id().map(|s| s.to_string());
    let subject = message.subject().map(|s| s.to_string());

    let from_address = extract_first_address(message.from());
    let reply_to = extract_first_address(message.reply_to());
    let to_addresses = extract_addresses(message.to());
    let cc_addresses = extract_addresses(message.cc());
    let bcc_addresses = extract_addresses(message.bcc());

    // Date — mail-parser 0.9 DateTime has to_timestamp() -> i64
    let date_sent = message.date().and_then(|dt| {
        chrono::DateTime::from_timestamp(dt.to_timestamp(), 0)
    });

    // Bodies
    let body_text = message.body_text(0).map(|s| s.to_string());
    let body_html = message.body_html(0).map(|s| s.to_string());

    // ── Headers ──────────────────────────────────────────────────
    let mut headers = Vec::new();
    for (i, hdr) in message.headers().iter().enumerate() {
        let header_name = hdr.name.as_str().to_string();
        let header_value = format_header_value(&hdr.value);
        headers.push(HeaderRow {
            header_name,
            header_value,
            ordinal: i as i32,
        });
    }

    // ── Received hops ────────────────────────────────────────────
    let received_hops = extract_received_hops(&message);

    // ── Attachments ──────────────────────────────────────────────
    let attachment_count = message.attachment_count();
    let mut attachments = Vec::with_capacity(attachment_count);

    for i in 0..attachment_count {
        if let Some(part) = message.attachment(i) {
            let bytes = part.contents().to_vec();
            let size_bytes = bytes.len() as i64;

            // SHA-256
            let mut hasher = Sha256::new();
            hasher.update(&bytes);
            let sha256_hash = hex::encode(hasher.finalize());

            // Filename from Content-Disposition or Content-Type
            let filename = part.attachment_name().map(|s| s.to_string());
            let safe_name = filename.as_deref().unwrap_or("unnamed");

            // Content-Type
            let content_type = part.content_type().map(|ct| {
                let main = ct.ctype();
                match ct.subtype() {
                    Some(sub) => format!("{main}/{sub}"),
                    None => main.to_string(),
                }
            });

            // Shannon entropy
            let entropy = Some(compute_entropy(&bytes));

            let s3_key = format!(
                "attachments/{tenant_id}/{email_id}/{i}_{safe_name}"
            );

            attachments.push(AttachmentData {
                row: AttachmentRow {
                    email_id,
                    tenant_id,
                    filename,
                    content_type,
                    size_bytes,
                    sha256_hash,
                    s3_bucket: s3_bucket.to_string(),
                    s3_key,
                    entropy,
                    ordinal: i as i32,
                },
                bytes,
            });
        }
    }

    let email_row = ParsedEmailRow {
        email_id,
        tenant_id,
        message_id,
        subject,
        from_address,
        to_addresses,
        cc_addresses,
        bcc_addresses,
        reply_to,
        date_sent,
        body_text,
        body_html,
        attachment_count: attachment_count as i32,
    };

    Ok(ParsedEmail {
        email_row,
        headers,
        received_hops,
        attachments,
    })
}

/// Shannon entropy of a byte slice (0.0 – 8.0).
/// High entropy (>7.0) may indicate encrypted/compressed content.
pub fn compute_entropy(data: &[u8]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }

    let mut freq = [0u64; 256];
    for &b in data {
        freq[b as usize] += 1;
    }

    let len = data.len() as f64;
    let mut entropy = 0.0f64;
    for &count in &freq {
        if count > 0 {
            let p = count as f64 / len;
            entropy -= p * p.log2();
        }
    }
    entropy
}

// ── Address extraction helpers ──────────────────────────────────

/// Extract the first email address from an `Address` (From, Reply-To).
///
/// mail-parser 0.9: `from()` returns `Option<&Address>` where
/// `Address` is `List(Vec<Addr>)` or `Group(Vec<Group>)`.
/// `Addr` has `.address() -> Option<&str>`.
fn extract_first_address(val: Option<&Address<'_>>) -> Option<String> {
    let addr = val?;
    // Address::first() returns the first Addr in either List or Group
    addr.first()?.address().map(|s| s.to_string())
}

/// Extract all email addresses from an `Address` (To, Cc, Bcc).
fn extract_addresses(val: Option<&Address<'_>>) -> Vec<String> {
    match val {
        Some(addr) => addr
            .iter()
            .filter_map(|a| a.address().map(|s| s.to_string()))
            .collect(),
        None => vec![],
    }
}

/// Format a HeaderValue for text storage.
fn format_header_value(value: &HeaderValue<'_>) -> String {
    match value {
        HeaderValue::Text(t) => t.to_string(),
        HeaderValue::DateTime(dt) => dt.to_rfc3339(),
        HeaderValue::Address(addr) => {
            let addrs: Vec<String> = addr
                .iter()
                .filter_map(|a| {
                    let email = a.address()?;
                    match a.name() {
                        Some(name) => Some(format!("{name} <{email}>")),
                        None => Some(email.to_string()),
                    }
                })
                .collect();
            addrs.join(", ")
        }
        HeaderValue::ContentType(ct) => {
            match ct.subtype() {
                Some(sub) => format!("{}/{}", ct.ctype(), sub),
                None => ct.ctype().to_string(),
            }
        }
        HeaderValue::Received(recv) => {
            format_received_raw(recv)
        }
        HeaderValue::TextList(list) => list.join(", "),
        HeaderValue::Empty => String::new(),
    }
}

/// Format a Received header value as a human-readable string.
fn format_received_raw(recv: &mail_parser::Received<'_>) -> String {
    let mut parts = Vec::new();
    if let Some(from) = recv.from() {
        parts.push(format!("from {}", format_host(from)));
    }
    if let Some(by) = recv.by() {
        parts.push(format!("by {}", format_host(by)));
    }
    if let Some(for_) = recv.for_() {
        parts.push(format!("for <{for_}>"));
    }
    if let Some(with) = recv.with() {
        parts.push(format!("with {with:?}"));
    }
    if let Some(date) = recv.date() {
        parts.push(date.to_rfc3339());
    }
    parts.join("; ")
}

/// Format a Host enum as a string.
fn format_host(host: &Host<'_>) -> String {
    match host {
        Host::Name(name) => name.to_string(),
        Host::IpAddr(ip) => ip.to_string(),
    }
}

/// Extract Received headers into ReceivedHopRow structs.
fn extract_received_hops(message: &mail_parser::Message<'_>) -> Vec<ReceivedHopRow> {
    let mut hops = Vec::new();
    let mut hop_index = 0i32;

    for hdr in message.headers() {
        if hdr.name == HeaderName::Received {
            let raw_value = format_header_value(&hdr.value);

            let (from_host, by_host, for_address, via_protocol, received_at) =
                match &hdr.value {
                    HeaderValue::Received(recv) => {
                        let from = recv.from().map(|h| format_host(h));
                        let by = recv.by().map(|h| format_host(h));
                        let for_addr = recv.for_().map(|s| s.to_string());
                        let via = recv.with().map(|p| format!("{p:?}"));
                        let date = recv.date().and_then(|dt| {
                            chrono::DateTime::from_timestamp(dt.to_timestamp(), 0)
                        });
                        (from, by, for_addr, via, date)
                    }
                    _ => (None, None, None, None, None),
                };

            hops.push(ReceivedHopRow {
                hop_index,
                from_host,
                by_host,
                for_address,
                via_protocol,
                received_at,
                raw_value,
            });
            hop_index += 1;
        }
    }

    hops
}
