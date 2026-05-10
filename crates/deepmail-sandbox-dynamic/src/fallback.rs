/// Fallback: use static analysis results when CAPEv2 is unavailable.

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::DynamicError;
use crate::parser::{CapeSignature, DynamicFindings};

/// Query the deepmail_sandbox_file database for existing static analysis.
/// Returns DynamicFindings with cape_unavailable=true.
pub async fn fallback_from_static(
    static_pool: &PgPool,
    attachment_id: Uuid,
    sha256_hash: Option<&str>,
) -> Result<DynamicFindings, DynamicError> {
    let row = sqlx::query(
        "SELECT threat_score, has_macros, vba_suspicious, yara_matches,
                pe_is_packed, pdf_has_js, suspicious_strings
         FROM file_static_reports
         WHERE attachment_id = $1
            OR (sha256_hash IS NOT NULL AND sha256_hash = $2)
         LIMIT 1",
    )
    .bind(attachment_id)
    .bind(sha256_hash.unwrap_or(""))
    .fetch_optional(static_pool)
    .await?;

    let mut findings = DynamicFindings {
        cape_unavailable: true,
        ..Default::default()
    };

    let row = match row {
        Some(r) => r,
        None => return Ok(findings),
    };

    use sqlx::Row;

    // Use static threat_score as malscore proxy (already 0.0–1.0)
    findings.malscore = row.try_get::<f32, _>("threat_score").unwrap_or(0.0);

    let has_macros: bool = row.try_get("has_macros").unwrap_or(false);
    let vba_suspicious: bool = row.try_get("vba_suspicious").unwrap_or(false);
    let pe_is_packed: bool = row.try_get("pe_is_packed").unwrap_or(false);
    let pdf_has_js: bool = row.try_get("pdf_has_js").unwrap_or(false);

    // Build proxy signatures from static indicators
    if has_macros {
        findings.cape_signatures.push(CapeSignature {
            name: "StaticMacroDetection".into(),
            description: "Macros detected in static analysis".into(),
            severity: 2,
            families: vec![],
        });
    }
    if vba_suspicious {
        findings.cape_signatures.push(CapeSignature {
            name: "SuspiciousVBA".into(),
            description: "Suspicious VBA code detected in static analysis".into(),
            severity: 3,
            families: vec![],
        });
    }
    if pe_is_packed {
        findings.cape_signatures.push(CapeSignature {
            name: "PackedPE".into(),
            description: "PE file appears packed (static analysis)".into(),
            severity: 2,
            families: vec![],
        });
    }
    if pdf_has_js {
        findings.cape_signatures.push(CapeSignature {
            name: "PDFJavaScript".into(),
            description: "PDF contains JavaScript (static analysis)".into(),
            severity: 2,
            families: vec![],
        });
    }

    // Extract C2 and persistence indicators from YARA matches
    let yara_matches: Vec<String> = row.try_get("yara_matches").unwrap_or_default();
    for rule in &yara_matches {
        let lower = rule.to_lowercase();
        if lower.contains("c2") || lower.contains("network") {
            findings
                .c2_indicators
                .push(format!("YARA match: {}", rule));
        }
        if lower.contains("autorun") || lower.contains("startup") || lower.contains("persist") {
            findings
                .persistence_indicators
                .push(format!("YARA match: {}", rule));
        }
    }

    Ok(findings)
}
