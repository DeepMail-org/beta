/// Campaign clustering via Jaccard similarity.

use std::collections::{HashMap, HashSet};

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::IocError;

/// Jaccard similarity between two sets.
pub fn jaccard_similarity(set_a: &HashSet<String>, set_b: &HashSet<String>) -> f32 {
    if set_a.is_empty() && set_b.is_empty() {
        return 0.0;
    }
    let intersection = set_a.intersection(set_b).count();
    let union = set_a.union(set_b).count();
    if union == 0 {
        return 0.0;
    }
    intersection as f32 / union as f32
}

/// Get the IOC value set for an email (ip/domain/url/hash only).
pub async fn get_email_ioc_set(
    pool: &PgPool,
    email_id: Uuid,
    tenant_id: Uuid,
) -> Result<HashSet<String>, IocError> {
    let rows: Vec<(String,)> = sqlx::query_as(
        r#"SELECT n.ioc_value
           FROM ioc_nodes n
           JOIN email_ioc_occurrences o ON o.ioc_node_id = n.id
           WHERE o.email_id = $1 AND o.tenant_id = $2
             AND n.ioc_type IN ('ip','domain','url','hash')"#,
    )
    .bind(email_id)
    .bind(tenant_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|(v,)| v).collect())
}

/// Build IOC fingerprint: top IOCs by severity (MALICIOUS first, then by score).
async fn build_fingerprint(
    pool: &PgPool,
    email_id: Uuid,
    tenant_id: Uuid,
) -> Result<Vec<String>, IocError> {
    let rows: Vec<(String,)> = sqlx::query_as(
        r#"SELECT n.ioc_value
           FROM ioc_nodes n
           JOIN email_ioc_occurrences o ON o.ioc_node_id = n.id
           WHERE o.email_id = $1 AND o.tenant_id = $2
             AND n.ioc_type IN ('ip','domain','url','hash')
           ORDER BY
             CASE n.threat_level
               WHEN 'MALICIOUS' THEN 0
               WHEN 'SUSPICIOUS' THEN 1
               WHEN 'MODERATE' THEN 2
               WHEN 'CLEAN' THEN 3
               ELSE 4
             END,
             n.intel_score DESC
           LIMIT 20"#,
    )
    .bind(email_id)
    .bind(tenant_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|(v,)| v).collect())
}

/// Assign an email to a campaign (or create a new one). Runs in a transaction.
pub async fn assign_campaign(
    pool: &PgPool,
    email_id: Uuid,
    tenant_id: Uuid,
    ioc_set: &HashSet<String>,
    similarity_threshold: f32,
    window_days: i64,
) -> Result<Option<Uuid>, IocError> {
    if ioc_set.is_empty() {
        return Ok(None);
    }

    let mut tx = pool.begin().await?;

    // Step 1: Load existing campaigns for this tenant from last N days
    let campaigns: Vec<(Uuid, Vec<String>, i32)> = sqlx::query_as(
        r#"SELECT id, ioc_fingerprint, member_count
           FROM campaign_clusters
           WHERE tenant_id = $1
             AND status IN ('CANDIDATE','CONFIRMED')
             AND last_email_at > now() - ($2::text || ' days')::interval
           ORDER BY last_email_at DESC
           LIMIT 100"#,
    )
    .bind(tenant_id)
    .bind(window_days.to_string())
    .fetch_all(&mut *tx)
    .await?;

    // Step 2: Find best matching campaign
    let mut best_campaign_id: Option<Uuid> = None;
    let mut best_similarity: f32 = 0.0;
    let mut best_member_count: i32 = 0;

    for (campaign_id, fingerprint, member_count) in &campaigns {
        let campaign_set: HashSet<String> = if *member_count > 50 {
            // Use fingerprint as proxy for large campaigns
            fingerprint.iter().cloned().collect()
        } else {
            // Load all member IOC sets
            let member_rows: Vec<(Uuid,)> = sqlx::query_as(
                r#"SELECT email_id FROM campaign_members
                   WHERE campaign_id = $1"#,
            )
            .bind(campaign_id)
            .fetch_all(&mut *tx)
            .await?;

            let mut union_set = HashSet::new();
            for (member_email_id,) in member_rows {
                let member_iocs: Vec<(String,)> = sqlx::query_as(
                    r#"SELECT n.ioc_value
                       FROM ioc_nodes n
                       JOIN email_ioc_occurrences o ON o.ioc_node_id = n.id
                       WHERE o.email_id = $1 AND o.tenant_id = $2
                         AND n.ioc_type IN ('ip','domain','url','hash')"#,
                )
                .bind(member_email_id)
                .bind(tenant_id)
                .fetch_all(&mut *tx)
                .await?;

                for (v,) in member_iocs {
                    union_set.insert(v);
                }
            }
            union_set
        };

        let sim = jaccard_similarity(ioc_set, &campaign_set);
        if sim >= similarity_threshold && sim > best_similarity {
            best_similarity = sim;
            best_campaign_id = Some(*campaign_id);
            best_member_count = *member_count;
        }
    }

    let campaign_id = if let Some(cid) = best_campaign_id {
        // Step 3: Match found — add to existing campaign
        sqlx::query(
            r#"INSERT INTO campaign_members (campaign_id, email_id, tenant_id, similarity_score)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (campaign_id, email_id) DO NOTHING"#,
        )
        .bind(cid)
        .bind(email_id)
        .bind(tenant_id)
        .bind(best_similarity)
        .execute(&mut *tx)
        .await?;

        // Update campaign metadata
        let new_count = best_member_count + 1;
        let new_status = if best_member_count == 1 {
            "CONFIRMED"
        } else if best_member_count >= 1 {
            // Keep existing status (already CONFIRMED or higher)
            "CONFIRMED"
        } else {
            "CANDIDATE"
        };

        // Merge fingerprint
        let new_fingerprint = build_fingerprint_merged(pool, email_id, tenant_id, cid, &mut tx).await?;

        sqlx::query(
            r#"UPDATE campaign_clusters
               SET member_count = $1,
                   status = $2,
                   last_email_at = now(),
                   ioc_fingerprint = $3,
                   updated_at = now()
               WHERE id = $4"#,
        )
        .bind(new_count)
        .bind(new_status)
        .bind(&new_fingerprint)
        .bind(cid)
        .execute(&mut *tx)
        .await?;

        cid
    } else {
        // Step 4: No match — create new campaign
        let new_id = Uuid::new_v4();
        let campaign_name = format!("Campaign-{}", &new_id.to_string()[..8]);
        let fingerprint = build_fingerprint(pool, email_id, tenant_id).await?;

        sqlx::query(
            r#"INSERT INTO campaign_clusters
                 (id, tenant_id, campaign_name, status, ioc_fingerprint, member_count)
               VALUES ($1, $2, $3, 'CANDIDATE', $4, 1)"#,
        )
        .bind(new_id)
        .bind(tenant_id)
        .bind(&campaign_name)
        .bind(&fingerprint)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"INSERT INTO campaign_members (campaign_id, email_id, tenant_id, similarity_score)
               VALUES ($1, $2, $3, 1.0)"#,
        )
        .bind(new_id)
        .bind(email_id)
        .bind(tenant_id)
        .execute(&mut *tx)
        .await?;

        new_id
    };

    tx.commit().await?;
    Ok(Some(campaign_id))
}

/// Build merged fingerprint: existing + new email's top IOCs, capped at 20.
async fn build_fingerprint_merged(
    pool: &PgPool,
    email_id: Uuid,
    tenant_id: Uuid,
    campaign_id: Uuid,
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<Vec<String>, IocError> {
    // Get existing fingerprint
    let existing: Option<(Vec<String>,)> = sqlx::query_as(
        "SELECT ioc_fingerprint FROM campaign_clusters WHERE id = $1",
    )
    .bind(campaign_id)
    .fetch_optional(&mut **tx)
    .await?;

    let mut combined: Vec<String> = existing
        .map(|(fp,)| fp)
        .unwrap_or_default();

    // Add new email's top IOCs
    let new_fp = build_fingerprint(pool, email_id, tenant_id).await?;
    for v in new_fp {
        if !combined.contains(&v) {
            combined.push(v);
        }
    }

    combined.truncate(20);
    Ok(combined)
}
