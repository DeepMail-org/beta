/// Database operations for url_sandbox_reports.

use sqlx::{FromRow, PgPool, Row};
use uuid::Uuid;

/// Insert a sandbox report.
pub async fn insert_report(
    pool: &PgPool,
    job_id: Uuid,
    email_id: Uuid,
    tenant_id: Uuid,
    original_url: &str,
    final_url: Option<&str>,
    redirect_count: i32,
    redirect_chain: &serde_json::Value,
    page_title: Option<&str>,
    network_requests: &serde_json::Value,
    cookies: &serde_json::Value,
    external_scripts: &serde_json::Value,
    iframes: &serde_json::Value,
    has_password_field: bool,
    has_email_field: bool,
    has_login_form: bool,
    has_download_trigger: bool,
    js_dialogs: &serde_json::Value,
    qr_decoded_urls: &[String],
    screenshot_s3_key: Option<&str>,
    threat_class: &str,
    threat_score: f32,
    analysis_notes: &[String],
) -> Result<Uuid, sqlx::Error> {
    let row = sqlx::query(
        "INSERT INTO url_sandbox_reports (
            job_id, email_id, tenant_id, original_url, final_url,
            redirect_count, redirect_chain, page_title,
            network_requests, cookies, external_scripts, iframes,
            has_password_field, has_email_field, has_login_form,
            has_download_trigger, js_dialogs, qr_decoded_urls,
            screenshot_s3_key, threat_class, threat_score, analysis_notes
         ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12,
            $13, $14, $15, $16, $17, $18, $19, $20, $21, $22
         )
         ON CONFLICT (job_id) DO UPDATE SET
            final_url = EXCLUDED.final_url,
            threat_class = EXCLUDED.threat_class,
            threat_score = EXCLUDED.threat_score
         RETURNING id"
    )
    .bind(job_id)
    .bind(email_id)
    .bind(tenant_id)
    .bind(original_url)
    .bind(final_url)
    .bind(redirect_count)
    .bind(redirect_chain)
    .bind(page_title)
    .bind(network_requests)
    .bind(cookies)
    .bind(external_scripts)
    .bind(iframes)
    .bind(has_password_field)
    .bind(has_email_field)
    .bind(has_login_form)
    .bind(has_download_trigger)
    .bind(js_dialogs)
    .bind(qr_decoded_urls)
    .bind(screenshot_s3_key)
    .bind(threat_class)
    .bind(threat_score)
    .bind(analysis_notes)
    .fetch_one(pool)
    .await?;

    Ok(row.get("id"))
}

/// Report row for gRPC responses.
#[derive(Debug, Clone, FromRow)]
pub struct UrlSandboxReportRow {
    pub id: Uuid,
    pub job_id: Uuid,
    pub original_url: String,
    pub final_url: Option<String>,
    pub redirect_count: i32,
    pub has_login_form: bool,
    pub threat_class: String,
    pub threat_score: f32,
}

/// Get report by job_id.
pub async fn get_by_job_id(
    pool: &PgPool,
    job_id: Uuid,
) -> Result<Option<UrlSandboxReportRow>, sqlx::Error> {
    let row = sqlx::query_as::<_, UrlSandboxReportRow>(
        "SELECT id, job_id, original_url, final_url, redirect_count,
                has_login_form, threat_class, threat_score
         FROM url_sandbox_reports
         WHERE job_id = $1"
    )
    .bind(job_id)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}
