/// Full homograph pipeline: extract → analyze → score → persist → publish.

use std::sync::Arc;

use sqlx::PgPool;
use uuid::Uuid;

use crate::analyzer;
use crate::brands::BrandRegistry;
use crate::db;
use crate::error::HomographError;
use crate::extractor;
use crate::similarity::RiskLevel;

/// Result of the pipeline.
#[derive(Debug, Clone)]
pub struct PipelineResult {
    pub analysis_id: Uuid,
    pub domains_checked: i32,
    pub high_risk_count: i32,
    pub overall_risk: String,
}

/// Shared context.
pub struct PipelineCtx {
    pub pool: PgPool,
    pub parser_pool: PgPool,
    pub ioc_pool: PgPool,
    pub brand_registry: Arc<BrandRegistry>,
    pub js: async_nats::jetstream::Context,
    pub min_score_threshold: f32,
}

/// Run the full homograph analysis pipeline. Idempotent.
pub async fn run_pipeline(
    ctx: &PipelineCtx,
    email_id: Uuid,
    tenant_id: Uuid,
) -> Result<PipelineResult, HomographError> {
    // a. Idempotency check
    if let Some(existing) = db::analyses::get_by_email(&ctx.pool, email_id).await? {
        tracing::info!(%email_id, "homograph analysis already exists, returning cached");
        return Ok(PipelineResult {
            analysis_id: existing.id,
            domains_checked: existing.domains_checked,
            high_risk_count: existing.high_risk_count,
            overall_risk: existing.overall_risk,
        });
    }

    // b. Extract domains
    let domains = extractor::extract_domains_from_email(
        &ctx.parser_pool,
        &ctx.ioc_pool,
        email_id,
    )
    .await?;

    // c. Analyze
    let analyses = analyzer::analyze_email_domains(
        &domains,
        &ctx.brand_registry,
        ctx.min_score_threshold,
    );

    // d. Determine overall risk
    let overall_risk = analyses
        .iter()
        .map(|a| a.best_match.risk_level)
        .max()
        .unwrap_or(RiskLevel::None);

    let high_risk_count = analyses
        .iter()
        .filter(|a| a.best_match.risk_level >= RiskLevel::High)
        .count() as i32;

    let domains_checked = domains.len() as i32;

    // e. Insert analysis
    let analysis_id = db::analyses::insert_analysis(
        &ctx.pool,
        email_id,
        tenant_id,
        domains_checked,
        high_risk_count,
        overall_risk.as_str(),
    )
    .await?;

    // f. Insert domain scores
    for da in &analyses {
        db::scores::insert_domain_score(
            &ctx.pool,
            analysis_id,
            &da.original_domain,
            &da.decoded_domain,
            &da.skeleton,
            &da.best_match.brand,
            da.best_match.raw_similarity,
            da.best_match.final_score,
            da.best_match.edit_distance as i32,
            da.best_match.mixed_script,
            da.best_match.punycode_abuse,
            da.best_match.risk_level.as_str(),
        )
        .await?;
    }

    // g. Update ingest job_progress (cross-service)
    let _ = sqlx::query(
        r#"UPDATE analysis_jobs
           SET progress = jsonb_set(
               COALESCE(progress, '{}'::jsonb),
               '{homograph}', '"completed"'::jsonb
           ),
           updated_at = now()
           WHERE email_id = $1"#,
    )
    .bind(email_id)
    .execute(&ctx.parser_pool)
    .await;

    // h. Publish NATS event
    let event = serde_json::json!({
        "email_id": email_id.to_string(),
        "tenant_id": tenant_id.to_string(),
        "overall_risk": overall_risk.as_str(),
        "high_risk_count": high_risk_count,
    });
    if let Ok(payload) = serde_json::to_vec(&event) {
        let _ = ctx
            .js
            .publish(
                "deepmail.events.homograph.completed".to_string(),
                payload.into(),
            )
            .await;
    }

    tracing::info!(
        %email_id,
        domains_checked,
        high_risk_count,
        overall_risk = overall_risk.as_str(),
        "homograph pipeline complete"
    );

    Ok(PipelineResult {
        analysis_id,
        domains_checked,
        high_risk_count,
        overall_risk: overall_risk.as_str().to_string(),
    })
}
