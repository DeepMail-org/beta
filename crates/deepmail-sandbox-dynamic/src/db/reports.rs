/// Database operations for dynamic analysis reports.

use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool, Row};
use uuid::Uuid;

use crate::error::DynamicError;

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct DynamicReportRow {
    pub id: Uuid,
    pub job_id: Uuid,
    pub email_id: Uuid,
    pub tenant_id: Uuid,
    pub attachment_id: Uuid,
    pub filename: String,
    pub sha256_hash: Option<String>,
    pub malscore: f32,
    pub cape_unavailable: bool,
    pub network_hosts: Vec<String>,
    pub dns_requests: Vec<String>,
    pub http_requests: serde_json::Value,
    pub smtp_activity: bool,
    pub processes_spawned: Vec<String>,
    pub files_dropped: serde_json::Value,
    pub registry_mods: Vec<String>,
    pub persistence_indicators: Vec<String>,
    pub c2_indicators: Vec<String>,
    pub cape_signatures: serde_json::Value,
    pub dynamic_score: f32,
    pub dynamic_verdict: String,
    pub analysis_notes: Vec<String>,
    pub cape_report_s3_key: Option<String>,
    pub analyzed_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Insert a dynamic analysis report. ON CONFLICT (job_id) DO UPDATE.
#[allow(clippy::too_many_arguments)]
pub async fn insert_report(
    pool: &PgPool,
    job_id: Uuid,
    email_id: Uuid,
    tenant_id: Uuid,
    attachment_id: Uuid,
    filename: &str,
    sha256_hash: Option<&str>,
    malscore: f32,
    cape_unavailable: bool,
    network_hosts: &[String],
    dns_requests: &[String],
    http_requests: &serde_json::Value,
    smtp_activity: bool,
    processes_spawned: &[String],
    files_dropped: &serde_json::Value,
    registry_mods: &[String],
    persistence_indicators: &[String],
    c2_indicators: &[String],
    cape_signatures: &serde_json::Value,
    dynamic_score: f32,
    dynamic_verdict: &str,
    analysis_notes: &[String],
    cape_report_s3_key: Option<&str>,
) -> Result<Uuid, DynamicError> {
    let row = sqlx::query(
        "INSERT INTO dynamic_analysis_reports
            (job_id, email_id, tenant_id, attachment_id, filename, sha256_hash,
             malscore, cape_unavailable, network_hosts, dns_requests,
             http_requests, smtp_activity, processes_spawned, files_dropped,
             registry_mods, persistence_indicators, c2_indicators,
             cape_signatures, dynamic_score, dynamic_verdict, analysis_notes,
             cape_report_s3_key)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22)
         ON CONFLICT (job_id) DO UPDATE SET
            malscore = EXCLUDED.malscore,
            cape_unavailable = EXCLUDED.cape_unavailable,
            network_hosts = EXCLUDED.network_hosts,
            dns_requests = EXCLUDED.dns_requests,
            http_requests = EXCLUDED.http_requests,
            smtp_activity = EXCLUDED.smtp_activity,
            processes_spawned = EXCLUDED.processes_spawned,
            files_dropped = EXCLUDED.files_dropped,
            registry_mods = EXCLUDED.registry_mods,
            persistence_indicators = EXCLUDED.persistence_indicators,
            c2_indicators = EXCLUDED.c2_indicators,
            cape_signatures = EXCLUDED.cape_signatures,
            dynamic_score = EXCLUDED.dynamic_score,
            dynamic_verdict = EXCLUDED.dynamic_verdict,
            analysis_notes = EXCLUDED.analysis_notes,
            cape_report_s3_key = EXCLUDED.cape_report_s3_key,
            analyzed_at = now()
         RETURNING id",
    )
    .bind(job_id)
    .bind(email_id)
    .bind(tenant_id)
    .bind(attachment_id)
    .bind(filename)
    .bind(sha256_hash)
    .bind(malscore)
    .bind(cape_unavailable)
    .bind(network_hosts)
    .bind(dns_requests)
    .bind(http_requests)
    .bind(smtp_activity)
    .bind(processes_spawned)
    .bind(files_dropped)
    .bind(registry_mods)
    .bind(persistence_indicators)
    .bind(c2_indicators)
    .bind(cape_signatures)
    .bind(dynamic_score)
    .bind(dynamic_verdict)
    .bind(analysis_notes)
    .bind(cape_report_s3_key)
    .fetch_one(pool)
    .await?;

    Ok(row.get("id"))
}

/// Get report by job_id.
#[allow(dead_code)]
pub async fn get_by_job_id(
    pool: &PgPool,
    job_id: Uuid,
) -> Result<Option<DynamicReportRow>, DynamicError> {
    let row = sqlx::query_as::<_, DynamicReportRow>(
        "SELECT * FROM dynamic_analysis_reports WHERE job_id = $1",
    )
    .bind(job_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// Get report by attachment_id (joins through jobs table).
pub async fn get_by_attachment_id(
    pool: &PgPool,
    attachment_id: Uuid,
) -> Result<Option<DynamicReportRow>, DynamicError> {
    let row = sqlx::query_as::<_, DynamicReportRow>(
        "SELECT r.*
         FROM dynamic_analysis_reports r
         JOIN dynamic_analysis_jobs j ON j.id = r.job_id
         WHERE j.attachment_id = $1
         LIMIT 1",
    )
    .bind(attachment_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}
