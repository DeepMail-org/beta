/// Database operations for file_static_reports.

use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool, Row};
use uuid::Uuid;

/// Full report row for pipeline results.
#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct FileReportRow {
    pub id: Uuid,
    pub email_id: Uuid,
    pub tenant_id: Uuid,
    pub attachment_id: Uuid,
    pub threat_score: f32,
    pub threat_verdict: String,
    pub has_macros: bool,
    pub is_pe: bool,
    pub yara_matches: Vec<String>,
    pub analysis_notes: Vec<String>,
}

/// Insert a complete report. Returns the report row.
#[allow(clippy::too_many_arguments)]
pub async fn insert_report(
    pool: &PgPool,
    email_id: Uuid,
    tenant_id: Uuid,
    attachment_id: Uuid,
    s3_key: &str,
    filename: &str,
    file_size: i64,
    mime_type: Option<&str>,
    file_magic: Option<&str>,
    sha256_hash: Option<&str>,
    md5_hash: Option<&str>,
    entropy: f32,
    has_macros: bool,
    macro_count: i32,
    has_vba: bool,
    vba_suspicious: bool,
    is_pe: bool,
    pe_is_signed: bool,
    pe_is_packed: bool,
    pe_suspicious_imports: &[String],
    is_pdf: bool,
    pdf_has_js: bool,
    pdf_has_launch: bool,
    pdf_has_embedded: bool,
    pdf_is_encrypted: bool,
    embedded_files: &serde_json::Value,
    yara_matches: &[String],
    suspicious_strings: &[String],
    exif_author: Option<&str>,
    exif_created: Option<DateTime<Utc>>,
    exif_modified: Option<DateTime<Utc>>,
    exif_software: Option<&str>,
    exif_raw: &serde_json::Value,
    strings_count: i32,
    tool_outputs: &serde_json::Value,
    threat_score: f32,
    threat_verdict: &str,
    analysis_notes: &[String],
) -> Result<FileReportRow, sqlx::Error> {
    let row = sqlx::query(
        "INSERT INTO file_static_reports (
            email_id, tenant_id, attachment_id, s3_key, filename,
            file_size, mime_type, file_magic, sha256_hash, md5_hash,
            entropy, has_macros, macro_count, has_vba, vba_suspicious,
            is_pe, pe_is_signed, pe_is_packed, pe_suspicious_imports,
            is_pdf, pdf_has_js, pdf_has_launch, pdf_has_embedded, pdf_is_encrypted,
            embedded_files, yara_matches, suspicious_strings,
            exif_author, exif_created, exif_modified, exif_software, exif_raw,
            strings_count, tool_outputs, threat_score, threat_verdict, analysis_notes
         ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
            $11, $12, $13, $14, $15, $16, $17, $18, $19,
            $20, $21, $22, $23, $24, $25, $26, $27,
            $28, $29, $30, $31, $32, $33, $34, $35, $36, $37
         )
         ON CONFLICT (attachment_id) DO UPDATE SET
            threat_score = EXCLUDED.threat_score,
            threat_verdict = EXCLUDED.threat_verdict,
            analyzed_at = now()
         RETURNING id, email_id, tenant_id, attachment_id,
                   threat_score, threat_verdict, has_macros, is_pe,
                   yara_matches, analysis_notes"
    )
    .bind(email_id)         // $1
    .bind(tenant_id)        // $2
    .bind(attachment_id)    // $3
    .bind(s3_key)           // $4
    .bind(filename)         // $5
    .bind(file_size)        // $6
    .bind(mime_type)        // $7
    .bind(file_magic)       // $8
    .bind(sha256_hash)      // $9
    .bind(md5_hash)         // $10
    .bind(entropy)          // $11
    .bind(has_macros)       // $12
    .bind(macro_count)      // $13
    .bind(has_vba)          // $14
    .bind(vba_suspicious)   // $15
    .bind(is_pe)            // $16
    .bind(pe_is_signed)     // $17
    .bind(pe_is_packed)     // $18
    .bind(pe_suspicious_imports) // $19
    .bind(is_pdf)           // $20
    .bind(pdf_has_js)       // $21
    .bind(pdf_has_launch)   // $22
    .bind(pdf_has_embedded) // $23
    .bind(pdf_is_encrypted) // $24
    .bind(embedded_files)   // $25
    .bind(yara_matches)     // $26
    .bind(suspicious_strings) // $27
    .bind(exif_author)      // $28
    .bind(exif_created)     // $29
    .bind(exif_modified)    // $30
    .bind(exif_software)    // $31
    .bind(exif_raw)         // $32
    .bind(strings_count)    // $33
    .bind(tool_outputs)     // $34
    .bind(threat_score)     // $35
    .bind(threat_verdict)   // $36
    .bind(analysis_notes)   // $37
    .fetch_one(pool)
    .await?;

    Ok(FileReportRow {
        id: row.get("id"),
        email_id: row.get("email_id"),
        tenant_id: row.get("tenant_id"),
        attachment_id: row.get("attachment_id"),
        threat_score: row.get("threat_score"),
        threat_verdict: row.get("threat_verdict"),
        has_macros: row.get("has_macros"),
        is_pe: row.get("is_pe"),
        yara_matches: row.get("yara_matches"),
        analysis_notes: row.get("analysis_notes"),
    })
}

/// Get report by attachment_id.
pub async fn get_by_attachment_id(
    pool: &PgPool,
    attachment_id: Uuid,
) -> Result<Option<FileReportRow>, sqlx::Error> {
    let row = sqlx::query_as::<_, FileReportRow>(
        "SELECT id, email_id, tenant_id, attachment_id,
                threat_score, threat_verdict, has_macros, is_pe,
                yara_matches, analysis_notes
         FROM file_static_reports
         WHERE attachment_id = $1"
    )
    .bind(attachment_id)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

/// Check if an attachment has already been analyzed.
#[allow(dead_code)]
pub async fn exists_for_attachment(
    pool: &PgPool,
    attachment_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let exists: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM file_static_reports WHERE attachment_id = $1 LIMIT 1"
    )
    .bind(attachment_id)
    .fetch_optional(pool)
    .await?;

    Ok(exists.is_some())
}
