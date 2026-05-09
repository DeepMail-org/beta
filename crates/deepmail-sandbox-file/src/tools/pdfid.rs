/// pdfid.py — PDF structure analysis.

use std::path::Path;

use crate::error::SandboxFileError;
use super::run_tool_with_timeout;

#[derive(Debug, Clone, Default)]
pub struct PdfidResult {
    pub js_count: i32,
    pub launch_count: i32,
    pub embedded_count: i32,
    pub encrypt_count: i32,
    pub acroform_count: i32,
    pub objstm_count: i32,
    pub is_pdf: bool,
}

/// Run pdfid.py for PDF structure analysis.
pub async fn run_pdfid(path: &Path, timeout: u64) -> Result<PdfidResult, SandboxFileError> {
    let path_str = path.to_string_lossy().to_string();

    // Try pdfid.py first, then python3 -m pdfid
    let result = match run_tool_with_timeout(
        "pdfid.py", &[&path_str], None, timeout,
    ).await {
        Ok(r) => r,
        Err(SandboxFileError::ToolNotFound(_)) => {
            // Try python3 -m pdfid
            match run_tool_with_timeout(
                "python3", &["-m", "pdfid", &path_str], None, timeout,
            ).await {
                Ok(r) => r,
                Err(SandboxFileError::ToolNotFound(_)) => {
                    tracing::warn!("pdfid not available, checking magic bytes");
                    return Ok(detect_pdf_by_magic(path));
                }
                Err(e) => return Err(e),
            }
        }
        Err(e) => return Err(e),
    };

    if result.timed_out {
        return Ok(PdfidResult { is_pdf: true, ..Default::default() });
    }

    let mut res = PdfidResult { is_pdf: true, ..Default::default() };

    for line in result.stdout.lines() {
        let trimmed = line.trim();
        if let Some(count) = extract_pdfid_count(trimmed, "/JavaScript") {
            res.js_count += count;
        }
        if let Some(count) = extract_pdfid_count(trimmed, "/JS") {
            res.js_count += count;
        }
        if let Some(count) = extract_pdfid_count(trimmed, "/Launch") {
            res.launch_count = count;
        }
        if let Some(count) = extract_pdfid_count(trimmed, "/EmbeddedFile") {
            res.embedded_count = count;
        }
        if let Some(count) = extract_pdfid_count(trimmed, "/Encrypt") {
            res.encrypt_count = count;
        }
        if let Some(count) = extract_pdfid_count(trimmed, "/AcroForm") {
            res.acroform_count = count;
        }
        if let Some(count) = extract_pdfid_count(trimmed, "/ObjStm") {
            res.objstm_count = count;
        }
    }

    Ok(res)
}

/// Extract count from pdfid output line: " /Keyword  count"
fn extract_pdfid_count(line: &str, keyword: &str) -> Option<i32> {
    let trimmed = line.trim();
    if !trimmed.starts_with(keyword) {
        return None;
    }
    let rest = trimmed[keyword.len()..].trim();
    rest.parse().ok()
}

/// Detect PDF by magic bytes when pdfid is unavailable.
fn detect_pdf_by_magic(path: &Path) -> PdfidResult {
    if let Ok(data) = std::fs::read(path) {
        if data.len() >= 4 && &data[..4] == b"%PDF" {
            return PdfidResult { is_pdf: true, ..Default::default() };
        }
    }
    PdfidResult::default()
}
