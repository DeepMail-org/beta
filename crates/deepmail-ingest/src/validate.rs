//! Five-step file validation pipeline.
//!
//! Steps run in order. The first failure short-circuits the rest.
//! Every step result (pass or fail) is logged to file_validations table.
//! This function does NOT write to the emails table — that is the
//! caller's responsibility after all steps pass.

/// Magic bytes for OLE Compound Document (used by .msg files).
/// Source: https://www.nationalarchives.gov.uk/PRONOM/fmt/111
const MSG_MAGIC: &[u8] = &[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];

/// Common email header keywords that valid .eml files start with.
const EML_HEADER_PREFIXES: &[&[u8]] = &[
    b"From ",
    b"From:",
    b"Return-Path:",
    b"Received:",
    b"Date:",
    b"MIME-Version:",
    b"Content-Type:",
    b"Message-ID:",
    b"X-",
];

use crate::error::IngestError;

/// Result of a single validation step.
pub struct StepResult {
    pub step: &'static str,
    pub passed: bool,
    pub detail: Option<String>,
}

/// Run all five validation steps against the uploaded file bytes.
///
/// Returns Ok(ValidationOutput) with all results if every step passes.
/// Returns Err((IngestError, Vec<StepResult>)) on the first failing step,
/// with the Vec<StepResult> attached for logging the partial results.
///
/// The caller must log every StepResult to file_validations regardless
/// of success or failure.
pub struct ValidationOutput {
    /// All step results in order (may be partial on failure).
    pub steps: Vec<StepResult>,
    /// The file extension, lowercased and including the dot.
    pub extension: String,
    /// The quarantine UUID name (without extension).
    pub quarantine_uuid: String,
}

pub fn validate_upload(
    original_filename: &str,
    content_type: &str,
    file_bytes: &[u8],
    max_size_bytes: u64,
) -> Result<ValidationOutput, (IngestError, Vec<StepResult>)> {
    let mut steps: Vec<StepResult> = Vec::with_capacity(5);

    // ── Step 1: Extension ──────────────────────────────────────────
    let extension = {
        let lower = original_filename.to_lowercase();
        if lower.ends_with(".eml") {
            ".eml".to_string()
        } else if lower.ends_with(".msg") {
            ".msg".to_string()
        } else {
            let ext = std::path::Path::new(original_filename)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_string();
            steps.push(StepResult {
                step: "extension",
                passed: false,
                detail: Some(format!("rejected extension: .{ext}")),
            });
            return Err((
                IngestError::DisallowedExtension(ext),
                steps,
            ));
        }
    };
    steps.push(StepResult {
        step: "extension",
        passed: true,
        detail: Some(format!("accepted: {extension}")),
    });

    // ── Step 2: Size ───────────────────────────────────────────────
    let size = file_bytes.len() as u64;
    if size > max_size_bytes {
        steps.push(StepResult {
            step: "size",
            passed: false,
            detail: Some(format!(
                "file is {size} bytes, limit is {max_size_bytes}"
            )),
        });
        return Err((
            IngestError::FileTooLarge {
                size_bytes: size,
                limit_bytes: max_size_bytes,
            },
            steps,
        ));
    }
    steps.push(StepResult {
        step: "size",
        passed: true,
        detail: Some(format!("{size} bytes")),
    });

    // ── Step 3: Magic bytes ────────────────────────────────────────
    let magic_ok = match extension.as_str() {
        ".eml" => EML_HEADER_PREFIXES
            .iter()
            .any(|prefix| file_bytes.starts_with(prefix)),
        ".msg" => file_bytes.len() >= MSG_MAGIC.len()
            && file_bytes.starts_with(MSG_MAGIC),
        _ => false,
    };
    if !magic_ok {
        steps.push(StepResult {
            step: "magic_bytes",
            passed: false,
            detail: Some(format!(
                "first bytes do not match {extension} signature"
            )),
        });
        return Err((IngestError::MagicBytesMismatch, steps));
    }
    steps.push(StepResult {
        step: "magic_bytes",
        passed: true,
        detail: None,
    });

    // ── Step 4: MIME type ──────────────────────────────────────────
    let allowed_mime = match extension.as_str() {
        ".eml" => &[
            "message/rfc822",
            "text/plain",
            "application/octet-stream",
        ][..],
        ".msg" => &[
            "application/vnd.ms-outlook",
            "application/octet-stream",
        ][..],
        _ => &[][..],
    };
    let ct_lower = content_type
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_lowercase();
    if !allowed_mime.contains(&ct_lower.as_str()) {
        steps.push(StepResult {
            step: "mime_type",
            passed: false,
            detail: Some(format!("disallowed content-type: {ct_lower}")),
        });
        return Err((IngestError::DisallowedMimeType(ct_lower), steps));
    }
    steps.push(StepResult {
        step: "mime_type",
        passed: true,
        detail: Some(ct_lower),
    });

    // ── Step 5: Path traversal ────────────────────────────────────
    // Build the quarantine name and verify it has no traversal sequences.
    let quarantine_uuid = uuid::Uuid::new_v4().to_string();
    let quarantine_name = format!("{quarantine_uuid}{extension}");
    if quarantine_name.contains("..") || quarantine_name.contains('\0') {
        steps.push(StepResult {
            step: "path_traversal",
            passed: false,
            detail: Some("traversal sequence detected".into()),
        });
        return Err((IngestError::PathTraversal, steps));
    }
    steps.push(StepResult {
        step: "path_traversal",
        passed: true,
        detail: None,
    });

    Ok(ValidationOutput {
        steps,
        extension,
        quarantine_uuid,
    })
}
