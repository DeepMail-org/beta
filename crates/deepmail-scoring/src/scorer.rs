use crate::aggregator::{self, CompositeResult};
use crate::db;
use crate::db::scores::ThreatScoreRow;
use crate::error::ScoringError;
use crate::fetcher::ServicePools;
use crate::signals::SignalName;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

pub struct ScoringCtx {
    pub pool: Arc<PgPool>,
    pub service_pools: Arc<ServicePools>,
    pub ingest_pool: Arc<PgPool>,
    pub nats: async_nats::Client,
    pub final_min_signals: usize,
}

pub async fn compute_and_store(
    ctx: &Arc<ScoringCtx>,
    email_id: Uuid,
    tenant_id: Uuid,
    force: bool,
) -> Result<CompositeResult, ScoringError> {
    if !force {
        if let Some(existing) = db::scores::get_by_email_id(&ctx.pool, email_id).await? {
            if existing.is_final {
                let mut signals = std::collections::HashMap::new();
                for (name, score) in signal_columns_to_map(&existing) {
                    signals.insert(name, score);
                }
                return Ok(CompositeResult {
                    final_score: existing.final_score,
                    final_verdict: aggregator::FinalVerdict::from_score(existing.final_score),
                    signals,
                    signals_available: existing.signals_available,
                    weight_total: existing.weight_total,
                });
            }
        }
    }

    let mut signals = crate::fetcher::fetch_all_signals(&ctx.service_pools, email_id).await;

    let stored_signals = db::signals::list_by_email(&ctx.pool, email_id).await?;
    for (name_str, score) in &stored_signals {
        if let Some(name) = SignalName::from_str(name_str) {
            signals.entry(name).or_insert(*score);
        }
    }

    let result = aggregator::compute_composite_score(&signals);

    let row = ThreatScoreRow {
        email_id,
        tenant_id,
        final_score: result.final_score,
        final_verdict: result.final_verdict.as_str().to_string(),
        signals_available: result.signals_available,
        signals_total: 11,
        header_score: signals.get(&SignalName::Header).copied(),
        dkim_score: signals.get(&SignalName::Dkim).copied(),
        geo_score: signals.get(&SignalName::Geo).copied(),
        ip_score: signals.get(&SignalName::Ip).copied(),
        intel_score: signals.get(&SignalName::Intel).copied(),
        ioc_score: signals.get(&SignalName::Ioc).copied(),
        homograph_score: signals.get(&SignalName::Homograph).copied(),
        body_score: signals.get(&SignalName::Body).copied(),
        sandbox_url_score: signals.get(&SignalName::SandboxUrl).copied(),
        sandbox_file_score: signals.get(&SignalName::SandboxFile).copied(),
        sandbox_dynamic_score: signals.get(&SignalName::SandboxDynamic).copied(),
        weight_total: result.weight_total,
        is_final: false,
    };

    db::scores::upsert_threat_score(&ctx.pool, &row).await?;

    let has_all_sandbox = signals.contains_key(&SignalName::SandboxUrl)
        && signals.contains_key(&SignalName::SandboxFile)
        && signals.contains_key(&SignalName::SandboxDynamic);
    let is_final = result.signals_available as usize >= ctx.final_min_signals
        || (result.signals_available >= 5 && has_all_sandbox);

    if is_final {
        db::scores::mark_final(&ctx.pool, email_id).await?;
    }

    let event = serde_json::json!({
        "email_id": email_id.to_string(),
        "tenant_id": tenant_id.to_string(),
        "final_score": result.final_score,
        "final_verdict": result.final_verdict.as_str(),
        "signals_available": result.signals_available,
        "is_final": is_final,
    });
    if let Err(e) = ctx
        .nats
        .publish(
            "deepmail.events.scoring.completed".to_string(),
            event.to_string().into(),
        )
        .await
    {
        tracing::warn!(%email_id, error = %e, "failed to publish scoring.completed");
    }

    if let Err(e) = sqlx::query(
        r#"INSERT INTO job_progress (email_id, tenant_id, stage, status, completed_at)
           VALUES ($1, $2, 'scoring', 'completed', now())
           ON CONFLICT (email_id, stage) DO UPDATE SET
             status = 'completed', completed_at = now()"#,
    )
    .bind(email_id)
    .bind(tenant_id)
    .execute(ctx.ingest_pool.as_ref())
    .await
    {
        tracing::warn!(%email_id, error = %e, "failed to update ingest job_progress");
    }

    Ok(result)
}

fn signal_columns_to_map(row: &ThreatScoreRow) -> Vec<(SignalName, f32)> {
    let mut out = Vec::new();
    if let Some(v) = row.header_score {
        out.push((SignalName::Header, v));
    }
    if let Some(v) = row.dkim_score {
        out.push((SignalName::Dkim, v));
    }
    if let Some(v) = row.geo_score {
        out.push((SignalName::Geo, v));
    }
    if let Some(v) = row.ip_score {
        out.push((SignalName::Ip, v));
    }
    if let Some(v) = row.intel_score {
        out.push((SignalName::Intel, v));
    }
    if let Some(v) = row.ioc_score {
        out.push((SignalName::Ioc, v));
    }
    if let Some(v) = row.homograph_score {
        out.push((SignalName::Homograph, v));
    }
    if let Some(v) = row.body_score {
        out.push((SignalName::Body, v));
    }
    if let Some(v) = row.sandbox_url_score {
        out.push((SignalName::SandboxUrl, v));
    }
    if let Some(v) = row.sandbox_file_score {
        out.push((SignalName::SandboxFile, v));
    }
    if let Some(v) = row.sandbox_dynamic_score {
        out.push((SignalName::SandboxDynamic, v));
    }
    out
}
