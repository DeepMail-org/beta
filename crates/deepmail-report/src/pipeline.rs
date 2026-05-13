use crate::assembler::{self, ServicePools};
use crate::db::{digests, exports};
use crate::error::ReportError;
use crate::renderer;
use crate::s3;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

pub struct PipelineCtx {
    pub pools: Arc<ServicePools>,
    pub s3: Arc<aws_sdk_s3::Client>,
    pub own_pool: Arc<PgPool>,
    pub bucket: String,
}

pub struct PipelineResult {
    pub report_id: Uuid,
    pub email_id: Uuid,
    pub json_s3_key: String,
    pub html_s3_key: String,
    pub final_score: f32,
    pub final_verdict: String,
}

pub async fn generate_report(
    ctx: &PipelineCtx,
    email_id: Uuid,
    tenant_id: Uuid,
    force: bool,
) -> Result<PipelineResult, ReportError> {
    if !force {
        let existing_json =
            exports::get_by_email_format(&ctx.own_pool, email_id, "json").await?;
        let existing_html =
            exports::get_by_email_format(&ctx.own_pool, email_id, "html").await?;

        if let (Some(j), Some(h)) = (existing_json, existing_html) {
            let scoring: Option<(f32, String)> = sqlx::query_as(
                "SELECT final_score, final_verdict FROM threat_scores WHERE email_id = $1 LIMIT 1",
            )
            .bind(email_id)
            .fetch_optional(ctx.pools.scoring.as_ref())
            .await
            .unwrap_or(None);

            let (score, verdict) = scoring.unwrap_or((0.0, "CLEAN".to_string()));

            return Ok(PipelineResult {
                report_id: j.id,
                email_id,
                json_s3_key: j.s3_key,
                html_s3_key: h.s3_key,
                final_score: score,
                final_verdict: verdict,
            });
        }
    }

    let report = assembler::assemble_report(&ctx.pools, email_id, tenant_id).await?;

    let json_bytes = renderer::render_json(&report)?;
    let html_bytes = renderer::render_html(&report);

    let json_key = format!("reports/{tenant_id}/{email_id}.json");
    let html_key = format!("reports/{tenant_id}/{email_id}.html");

    let (json_upload, html_upload) = tokio::join!(
        s3::upload_report(
            &ctx.s3,
            &ctx.bucket,
            &json_key,
            json_bytes.clone(),
            "application/json",
        ),
        s3::upload_report(
            &ctx.s3,
            &ctx.bucket,
            &html_key,
            html_bytes.clone(),
            "text/html; charset=utf-8",
        ),
    );
    json_upload?;
    html_upload?;

    let json_size = json_bytes.len() as i64;
    let html_size = html_bytes.len() as i64;

    let (json_db, html_db) = tokio::join!(
        exports::upsert_export(
            &ctx.own_pool,
            email_id,
            tenant_id,
            "json",
            &json_key,
            json_size,
        ),
        exports::upsert_export(
            &ctx.own_pool,
            email_id,
            tenant_id,
            "html",
            &html_key,
            html_size,
        ),
    );
    json_db?;
    html_db?;

    Ok(PipelineResult {
        report_id: report.report_id,
        email_id,
        json_s3_key: json_key,
        html_s3_key: html_key,
        final_score: report.final_score,
        final_verdict: report.final_verdict,
    })
}

pub async fn schedule_digest(
    pool: &PgPool,
    tenant_id: Uuid,
    frequency: &str,
    recipient_emails: &[String],
) -> Result<digests::DigestRow, ReportError> {
    let valid = ["daily", "weekly", "monthly", "never"];
    if !valid.contains(&frequency) {
        return Err(ReportError::InvalidArgument(format!(
            "frequency must be one of: {}",
            valid.join(", ")
        )));
    }

    let next_send_at = match frequency {
        "daily" => Some(chrono::Utc::now() + chrono::Duration::days(1)),
        "weekly" => Some(chrono::Utc::now() + chrono::Duration::weeks(1)),
        "monthly" => Some(chrono::Utc::now() + chrono::Duration::days(30)),
        _ => None,
    };

    digests::upsert_schedule(pool, tenant_id, frequency, recipient_emails, next_send_at)
        .await
        .map_err(ReportError::from)
}
