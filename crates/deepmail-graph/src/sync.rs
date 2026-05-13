use crate::db;
use crate::error::GraphError;
use crate::neo4j::Neo4jClient;
use sqlx::postgres::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub struct SyncCtx {
    pub neo4j: Neo4jClient,
    pub pool: Arc<PgPool>,
    pub ioc_pool: Arc<PgPool>,
    pub scoring_pool: Arc<PgPool>,
}

fn ioc_type_to_label(ioc_type: &str) -> Option<&'static str> {
    match ioc_type {
        "ip" => Some("IpAddress"),
        "domain" => Some("Domain"),
        "url" => Some("Url"),
        "hash" => Some("FileHash"),
        "email" => Some("Sender"),
        _ => None,
    }
}

fn relation_type_to_neo4j(rel: &str) -> &'static str {
    match rel {
        "RESOLVES_TO" => "RESOLVES_TO",
        "HOSTED_ON" => "HOSTED_ON",
        "SUBDOMAIN_OF" => "SUBDOMAIN_OF",
        "SENT_FROM" => "SENT_BY",
        "HAS_HASH" => "HAS_HASH",
        "COMMUNICATES_WITH" => "C2_COMMUNICATES_WITH",
        "REDIRECTS_TO" => "HOSTED_ON",
        "DROPS" => "HAS_HASH",
        _ => "RELATED_TO",
    }
}

pub async fn sync_email(
    ctx: &SyncCtx,
    email_id: Uuid,
    tenant_id: Uuid,
) -> Result<(i32, i32), GraphError> {
    if let Some(existing) = db::sync_log::get_by_email_id(&ctx.pool, email_id).await? {
        if existing.sync_status == "completed" {
            return Ok((0, 0));
        }
    }

    db::sync_log::upsert_sync_log(&ctx.pool, email_id, tenant_id, 0, 0, "pending", None).await?;

    let result = do_sync(ctx, email_id, tenant_id).await;

    match &result {
        Ok((nodes, edges)) => {
            db::sync_log::upsert_sync_log(
                &ctx.pool,
                email_id,
                tenant_id,
                *nodes,
                *edges,
                "completed",
                None,
            )
            .await?;
        }
        Err(e) => {
            let _ = db::sync_log::upsert_sync_log(
                &ctx.pool,
                email_id,
                tenant_id,
                0,
                0,
                "failed",
                Some(&e.to_string()),
            )
            .await;
        }
    }

    result
}

async fn do_sync(
    ctx: &SyncCtx,
    email_id: Uuid,
    tenant_id: Uuid,
) -> Result<(i32, i32), GraphError> {
    let mut node_count: i32 = 0;
    let mut edge_count: i32 = 0;

    let email_props = fetch_email_score(ctx, email_id).await;
    let neo4j_id = ctx
        .neo4j
        .merge_node("Email", &email_id.to_string(), &email_props)
        .await?;
    node_count += 1;

    let _ = db::node_cache::upsert_node_cache(
        &ctx.pool,
        tenant_id,
        "email",
        &email_id.to_string(),
        &neo4j_id,
        &serde_json::json!(email_props),
    )
    .await;

    let iocs = fetch_email_iocs(ctx, email_id).await?;

    for ioc in &iocs {
        let label = match ioc_type_to_label(&ioc.ioc_type) {
            Some(l) => l,
            None => continue,
        };

        let mut props = HashMap::new();
        props.insert("threat_level".to_string(), ioc.threat_level.clone());
        props.insert("threat_score".to_string(), ioc.intel_score.to_string());
        props.insert("tenant_id".to_string(), tenant_id.to_string());

        let ioc_neo4j_id = ctx.neo4j.merge_node(label, &ioc.ioc_value, &props).await?;
        node_count += 1;

        let _ = db::node_cache::upsert_node_cache(
            &ctx.pool,
            tenant_id,
            &ioc.ioc_type,
            &ioc.ioc_value,
            &ioc_neo4j_id,
            &serde_json::json!({"threat_level": ioc.threat_level, "intel_score": ioc.intel_score}),
        )
        .await;

        let edge_props = HashMap::new();
        ctx.neo4j
            .merge_relationship(
                "Email",
                &email_id.to_string(),
                label,
                &ioc.ioc_value,
                "CONTAINS_IOC",
                &edge_props,
            )
            .await?;
        edge_count += 1;
    }

    let relations = fetch_ioc_relations(ctx, email_id).await?;
    let ioc_map: HashMap<Uuid, &IocRow> = iocs.iter().map(|i| (i.id, i)).collect();

    for rel in &relations {
        let source = match ioc_map.get(&rel.source_ioc_id) {
            Some(s) => s,
            None => continue,
        };
        let target = match ioc_map.get(&rel.target_ioc_id) {
            Some(t) => t,
            None => continue,
        };
        let source_label = match ioc_type_to_label(&source.ioc_type) {
            Some(l) => l,
            None => continue,
        };
        let target_label = match ioc_type_to_label(&target.ioc_type) {
            Some(l) => l,
            None => continue,
        };

        let neo4j_rel_type = relation_type_to_neo4j(&rel.relation_type);
        let mut props = HashMap::new();
        props.insert("confidence".to_string(), rel.confidence.to_string());
        props.insert("email_id".to_string(), email_id.to_string());

        ctx.neo4j
            .merge_relationship(
                source_label,
                &source.ioc_value,
                target_label,
                &target.ioc_value,
                neo4j_rel_type,
                &props,
            )
            .await?;
        edge_count += 1;
    }

    if let Some(campaign) = fetch_campaign_for_email(ctx, email_id, tenant_id).await? {
        let mut props = HashMap::new();
        props.insert("campaign_name".to_string(), campaign.campaign_name.clone());
        props.insert("tenant_id".to_string(), tenant_id.to_string());

        ctx.neo4j
            .merge_node("Campaign", &campaign.id.to_string(), &props)
            .await?;
        node_count += 1;

        let edge_props = HashMap::new();
        ctx.neo4j
            .merge_relationship(
                "Email",
                &email_id.to_string(),
                "Campaign",
                &campaign.id.to_string(),
                "MEMBER_OF",
                &edge_props,
            )
            .await?;
        edge_count += 1;
    }

    Ok((node_count, edge_count))
}

#[derive(Debug)]
struct IocRow {
    id: Uuid,
    ioc_type: String,
    ioc_value: String,
    threat_level: String,
    intel_score: f32,
}

#[derive(Debug)]
struct IocRelationRow {
    source_ioc_id: Uuid,
    target_ioc_id: Uuid,
    relation_type: String,
    confidence: f32,
}

#[derive(Debug)]
struct CampaignRow {
    id: Uuid,
    campaign_name: String,
}

async fn fetch_email_score(ctx: &SyncCtx, email_id: Uuid) -> HashMap<String, String> {
    let mut props = HashMap::new();

    let row: Option<(f32, String)> = sqlx::query_as(
        "SELECT final_score, final_verdict FROM threat_scores WHERE email_id = $1 LIMIT 1",
    )
    .bind(email_id)
    .fetch_optional(ctx.scoring_pool.as_ref())
    .await
    .unwrap_or_else(|e| { tracing::warn!(error = %e, "cross-service query failed"); None });

    if let Some((score, verdict)) = row {
        props.insert("threat_score".to_string(), score.to_string());
        props.insert("threat_verdict".to_string(), verdict);
    }

    props
}

async fn fetch_email_iocs(ctx: &SyncCtx, email_id: Uuid) -> Result<Vec<IocRow>, GraphError> {
    let rows: Vec<(Uuid, String, String, String, f32)> = sqlx::query_as(
        "SELECT n.id, n.ioc_type, n.ioc_value, n.threat_level, n.intel_score \
         FROM email_ioc_occurrences o \
         JOIN ioc_nodes n ON n.id = o.ioc_node_id \
         WHERE o.email_id = $1",
    )
    .bind(email_id)
    .fetch_all(ctx.ioc_pool.as_ref())
    .await
    .map_err(|e| GraphError::Internal(format!("ioc fetch: {e}")))?;

    Ok(rows
        .into_iter()
        .map(|(id, ioc_type, ioc_value, threat_level, intel_score)| IocRow {
            id,
            ioc_type,
            ioc_value,
            threat_level,
            intel_score,
        })
        .collect())
}

async fn fetch_ioc_relations(
    ctx: &SyncCtx,
    email_id: Uuid,
) -> Result<Vec<IocRelationRow>, GraphError> {
    let rows: Vec<(Uuid, Uuid, String, f32)> = sqlx::query_as(
        "SELECT source_ioc_id, target_ioc_id, relation_type, confidence \
         FROM ioc_relations WHERE email_id = $1",
    )
    .bind(email_id)
    .fetch_all(ctx.ioc_pool.as_ref())
    .await
    .map_err(|e| GraphError::Internal(format!("ioc_relations fetch: {e}")))?;

    Ok(rows
        .into_iter()
        .map(
            |(source_ioc_id, target_ioc_id, relation_type, confidence)| IocRelationRow {
                source_ioc_id,
                target_ioc_id,
                relation_type,
                confidence,
            },
        )
        .collect())
}

async fn fetch_campaign_for_email(
    ctx: &SyncCtx,
    email_id: Uuid,
    tenant_id: Uuid,
) -> Result<Option<CampaignRow>, GraphError> {
    let row: Option<(Uuid, String)> = sqlx::query_as(
        "SELECT c.id, c.campaign_name \
         FROM campaign_members m \
         JOIN campaign_clusters c ON c.id = m.campaign_id \
         WHERE m.email_id = $1 AND m.tenant_id = $2 \
         LIMIT 1",
    )
    .bind(email_id)
    .bind(tenant_id)
    .fetch_optional(ctx.ioc_pool.as_ref())
    .await
    .map_err(|e| GraphError::Internal(format!("campaign fetch: {e}")))?;

    Ok(row.map(|(id, campaign_name)| CampaignRow { id, campaign_name }))
}
