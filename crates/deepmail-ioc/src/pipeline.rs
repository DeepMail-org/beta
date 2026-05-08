/// Full IOC pipeline orchestration:
/// extract → normalize → upsert → enrich → relations → cluster → publish.

use std::collections::HashSet;
use std::sync::Arc;

use sqlx::PgPool;
use uuid::Uuid;

use crate::cluster;
use crate::db;
use crate::enrich::IntelGrpcClient;
use crate::error::IocError;
use crate::extract;
use crate::normalize;
use crate::relations::{self, NormalizedIoc};

/// Result of the full pipeline run.
#[derive(Debug, Clone)]
pub struct PipelineResult {
    pub ioc_count: i32,
    pub malicious_count: i32,
    pub campaign_id: String,
    pub campaign_status: String,
}

/// Shared context for the pipeline.
pub struct PipelineCtx {
    pub pool: PgPool,
    pub parser_pool: PgPool,
    pub ingest_pool: PgPool,
    pub intel_client: Arc<IntelGrpcClient>,
    pub js: async_nats::jetstream::Context,
    pub enrich_concurrency: usize,
    pub campaign_window_days: i64,
    pub similarity_threshold: f32,
}

/// Run the complete IOC pipeline for one email. Idempotent.
pub async fn run_pipeline(
    ctx: &PipelineCtx,
    email_id: Uuid,
    tenant_id: Uuid,
) -> Result<PipelineResult, IocError> {
    // a. Idempotency check
    if db::occurrences::has_occurrences(&ctx.pool, email_id).await? {
        tracing::info!(%email_id, "IOCs already extracted, returning cached result");
        return build_existing_result(&ctx.pool, email_id, tenant_id).await;
    }

    // b. Extract raw IOCs from parser DB
    let raw_iocs = extract::extract_email_iocs(&ctx.parser_pool, email_id).await?;

    if raw_iocs.is_empty() {
        tracing::info!(%email_id, "no IOCs extracted");
        return Ok(PipelineResult {
            ioc_count: 0,
            malicious_count: 0,
            campaign_id: String::new(),
            campaign_status: String::new(),
        });
    }

    // c. Normalize + deduplicate
    let deduped = normalize::deduplicate(raw_iocs);
    tracing::info!(%email_id, deduped_count = deduped.len(), "IOCs normalized");

    // d. Upsert IOC nodes + occurrences
    let mut normalized_iocs: Vec<NormalizedIoc> = Vec::new();

    for ioc in &deduped {
        let node_id = db::nodes::upsert_node(
            &ctx.pool,
            tenant_id,
            &ioc.ioc_type,
            &ioc.value,
            email_id,
        )
        .await?;

        db::occurrences::insert_occurrence(
            &ctx.pool,
            email_id,
            tenant_id,
            node_id,
            &ioc.source,
            &ioc.raw_value,
        )
        .await?;

        normalized_iocs.push(NormalizedIoc {
            node_id,
            ioc_type: ioc.ioc_type.clone(),
            ioc_value: ioc.value.clone(),
            source: ioc.source.clone(),
        });
    }

    // e. Enrich IOCs concurrently (only ip/domain/url/hash)
    let enrichable: Vec<&NormalizedIoc> = normalized_iocs
        .iter()
        .filter(|i| matches!(i.ioc_type.as_str(), "ip" | "domain" | "url" | "hash"))
        .collect();

    let semaphore = Arc::new(tokio::sync::Semaphore::new(ctx.enrich_concurrency));
    let mut enrich_handles = Vec::new();

    for ioc in enrichable {
        let client = ctx.intel_client.clone();
        let sem = semaphore.clone();
        let node_id = ioc.node_id;
        let ioc_value = ioc.ioc_value.clone();
        let ioc_type = ioc.ioc_type.clone();
        let pool = ctx.pool.clone();

        let handle = tokio::spawn(async move {
            let _permit = sem.acquire().await.map_err(|_| {
                IocError::Internal("semaphore closed".into())
            })?;

            let summary = client.enrich_ioc(&ioc_value, &ioc_type).await;

            // Update node with enrichment
            let intel_json = serde_json::to_value(&summary.provider_results)
                .unwrap_or(serde_json::json!({}));

            db::nodes::update_enrichment(
                &pool,
                node_id,
                summary.threat_level.as_str(),
                summary.score,
                &intel_json,
            )
            .await?;

            Ok::<(uuid::Uuid, crate::enrich::EnrichmentSummary), IocError>((node_id, summary))
        });

        enrich_handles.push(handle);
    }

    let mut malicious_count = 0i32;
    for handle in enrich_handles {
        match handle.await {
            Ok(Ok((_node_id, summary))) => {
                if summary.threat_level == crate::enrich::ThreatLevel::Malicious {
                    malicious_count += 1;
                }
            }
            Ok(Err(e)) => {
                tracing::warn!(error = %e, "enrichment update failed");
            }
            Err(e) => {
                tracing::warn!(error = %e, "enrichment task panicked");
            }
        }
    }

    // f. Infer IOC relations
    let ioc_relations = relations::infer_relations(&normalized_iocs, email_id);
    if !ioc_relations.is_empty() {
        db::relations::bulk_insert_relations(&ctx.pool, tenant_id, email_id, &ioc_relations)
            .await?;
        tracing::info!(
            %email_id,
            relation_count = ioc_relations.len(),
            "IOC relations persisted"
        );
    }

    // g. Campaign clustering
    let ioc_set: HashSet<String> = normalized_iocs
        .iter()
        .filter(|i| matches!(i.ioc_type.as_str(), "ip" | "domain" | "url" | "hash"))
        .map(|i| i.ioc_value.clone())
        .collect();

    let campaign_id = cluster::assign_campaign(
        &ctx.pool,
        email_id,
        tenant_id,
        &ioc_set,
        ctx.similarity_threshold,
        ctx.campaign_window_days,
    )
    .await?;

    let (campaign_id_str, campaign_status) = if let Some(cid) = campaign_id {
        match db::campaigns::get_campaign_for_email(&ctx.pool, email_id).await? {
            Some((_, status)) => (cid.to_string(), status),
            None => (cid.to_string(), "CANDIDATE".into()),
        }
    } else {
        (String::new(), String::new())
    };

    let ioc_count = normalized_iocs.len() as i32;

    // i. Update ingest job_progress (cross-service)
    let _ = sqlx::query(
        r#"UPDATE analysis_jobs
           SET progress = jsonb_set(
               COALESCE(progress, '{}'::jsonb),
               '{ioc}', '"completed"'::jsonb
           ),
           updated_at = now()
           WHERE email_id = $1"#,
    )
    .bind(email_id)
    .execute(&ctx.ingest_pool)
    .await;

    // j. Publish NATS event
    let event = serde_json::json!({
        "email_id": email_id.to_string(),
        "tenant_id": tenant_id.to_string(),
        "ioc_count": ioc_count,
        "malicious_count": malicious_count,
        "campaign_id": campaign_id_str,
    });
    if let Ok(payload) = serde_json::to_vec(&event) {
        let _ = ctx
            .js
            .publish(
                "deepmail.events.ioc.completed".to_string(),
                payload.into(),
            )
            .await;
    }

    tracing::info!(
        %email_id,
        ioc_count,
        malicious_count,
        campaign_id = %campaign_id_str,
        "IOC pipeline complete"
    );

    Ok(PipelineResult {
        ioc_count,
        malicious_count,
        campaign_id: campaign_id_str,
        campaign_status,
    })
}

/// Build result from existing data (for idempotent re-runs).
async fn build_existing_result(
    pool: &PgPool,
    email_id: Uuid,
    tenant_id: Uuid,
) -> Result<PipelineResult, IocError> {
    let nodes = db::nodes::get_by_email(pool, email_id, tenant_id).await?;
    let ioc_count = nodes.len() as i32;
    let malicious_count = nodes
        .iter()
        .filter(|n| n.threat_level == "MALICIOUS")
        .count() as i32;

    let (campaign_id, campaign_status) =
        match db::campaigns::get_campaign_for_email(pool, email_id).await? {
            Some((cid, status)) => (cid.to_string(), status),
            None => (String::new(), String::new()),
        };

    Ok(PipelineResult {
        ioc_count,
        malicious_count,
        campaign_id,
        campaign_status,
    })
}
