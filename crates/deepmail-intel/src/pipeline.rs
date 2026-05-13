/// Per-email orchestration: extract IOCs from parser DB → enrich → persist → publish.

use std::collections::HashSet;
use std::sync::Arc;

use sqlx::PgPool;
use uuid::Uuid;

use crate::enricher::{enrich_ioc, EnrichCtx, IocEnrichment};
use crate::error::IntelError;

/// A single IOC extracted from a parsed email.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct Ioc {
    value: String,
    ioc_type: String,
}

/// Run the full intel pipeline for a given email.
pub async fn run_pipeline(
    email_id: Uuid,
    tenant_id: Uuid,
    ctx: Arc<EnrichCtx>,
    parser_pool: &PgPool,
    ingest_pool: &PgPool,
) -> Result<(), IntelError> {
    tracing::info!(%email_id, "starting intel pipeline");

    // 1. Extract IOCs from parser DB (cross-service)
    let iocs = extract_iocs(email_id, parser_pool).await?;

    if iocs.is_empty() {
        tracing::info!(%email_id, "no IOCs extracted, skipping enrichment");
        // Persist empty result
        crate::db::results::upsert_email_result(
            &ctx.pool,
            email_id,
            tenant_id,
            0,
            0.0,
            0,
            &[],
            &serde_json::json!({"status": "no_iocs"}),
        )
        .await?;
        return Ok(());
    }

    tracing::info!(%email_id, ioc_count = iocs.len(), "enriching IOCs");

    // 2. Enrich IOCs concurrently (max 20)
    let semaphore = Arc::new(tokio::sync::Semaphore::new(20));
    let mut handles = Vec::new();

    for ioc in &iocs {
        let ctx = ctx.clone();
        let sem = semaphore.clone();
        let ioc_value = ioc.value.clone();
        let ioc_type = ioc.ioc_type.clone();

        let handle = tokio::spawn(async move {
            let _permit = sem
                .acquire()
                .await
                .map_err(|_| IntelError::Internal("semaphore closed".to_string()))?;
            enrich_ioc(&ioc_value, &ioc_type, &[], false, &ctx).await
        });

        handles.push(handle);
    }

    // Collect results
    let mut enrichments: Vec<IocEnrichment> = Vec::new();
    for handle in handles {
        match handle.await {
            Ok(Ok(result)) => enrichments.push(result),
            Ok(Err(e)) => {
                tracing::warn!(error = %e, "IOC enrichment failed");
            }
            Err(e) => {
                tracing::warn!(error = %e, "IOC enrichment task panicked");
            }
        }
    }

    // 3. Compute summary
    let iocs_analyzed = enrichments.len() as i32;
    let max_vt_score = enrichments
        .iter()
        .map(|e| e.max_score)
        .fold(0.0_f32, f32::max);
    let malicious_iocs = enrichments.iter().filter(|e| e.is_malicious).count() as i32;

    let mut all_providers: HashSet<String> = HashSet::new();
    for e in &enrichments {
        for key in e.provider_results.keys() {
            all_providers.insert(key.clone());
        }
    }
    let provider_hits: Vec<String> = all_providers.into_iter().collect();

    let summary = serde_json::json!({
        "iocs_analyzed": iocs_analyzed,
        "max_vt_score": max_vt_score,
        "malicious_iocs": malicious_iocs,
        "provider_hits": provider_hits,
        "ioc_details": enrichments.iter().map(|e| {
            serde_json::json!({
                "ioc": e.ioc_value,
                "type": e.ioc_type,
                "score": e.max_score,
                "malicious": e.is_malicious,
                "providers": e.provider_results.keys().collect::<Vec<_>>(),
            })
        }).collect::<Vec<_>>(),
    });

    // 4. Persist email_intel_results
    crate::db::results::upsert_email_result(
        &ctx.pool,
        email_id,
        tenant_id,
        iocs_analyzed,
        max_vt_score,
        malicious_iocs,
        &provider_hits,
        &summary,
    )
    .await?;

    // 5. Update ingest job progress (cross-service, runtime query)
    let _ = sqlx::query(
        r#"UPDATE analysis_jobs
           SET progress = jsonb_set(
               COALESCE(progress, '{}'::jsonb),
               '{intel}', '"completed"'::jsonb
           ),
           updated_at = now()
           WHERE email_id = $1"#,
    )
    .bind(email_id)
    .execute(ingest_pool)
    .await;

    tracing::info!(
        %email_id,
        iocs_analyzed,
        malicious_iocs,
        max_vt_score,
        "intel pipeline complete"
    );

    Ok(())
}

/// Extract IOCs from parser DB tables.
async fn extract_iocs(email_id: Uuid, parser_pool: &PgPool) -> Result<HashSet<Ioc>, IntelError> {
    let mut iocs = HashSet::new();

    // Get parsed_email_id
    let parsed_email_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM parsed_emails WHERE email_id = $1 LIMIT 1",
    )
    .bind(email_id)
    .fetch_optional(parser_pool)
    .await?;

    let parsed_id = match parsed_email_id {
        Some(id) => id,
        None => {
            tracing::warn!(%email_id, "no parsed email found");
            return Ok(iocs);
        }
    };

    // IPs from received hops (using ::text cast for INET)
    let ip_rows: Vec<(String,)> = sqlx::query_as(
        r#"SELECT CAST(from_ip AS TEXT)
           FROM received_hops
           WHERE parsed_email_id = $1
             AND from_ip IS NOT NULL"#,
    )
    .bind(parsed_id)
    .fetch_all(parser_pool)
    .await
    .unwrap_or_default();

    for (ip,) in ip_rows {
        let trimmed = ip.trim().to_string();
        if !trimmed.is_empty() {
            iocs.insert(Ioc {
                value: trimmed,
                ioc_type: "ip".to_string(),
            });
        }
    }

    // Domains from email headers (from, reply-to, return-path)
    let header_rows: Vec<(String,)> = sqlx::query_as(
        r#"SELECT value
           FROM email_headers
           WHERE parsed_email_id = $1
             AND name IN ('from','reply-to','return-path')"#,
    )
    .bind(parsed_id)
    .fetch_all(parser_pool)
    .await
    .unwrap_or_default();

    for (val,) in header_rows {
        if let Some(domain) = extract_domain_from_header(&val) {
            iocs.insert(Ioc {
                value: domain,
                ioc_type: "domain".to_string(),
            });
        }
    }

    // File hashes from attachments
    let hash_rows: Vec<(String,)> = sqlx::query_as(
        r#"SELECT sha256_hash
           FROM attachments
           WHERE parsed_email_id = $1
             AND sha256_hash IS NOT NULL"#,
    )
    .bind(parsed_id)
    .fetch_all(parser_pool)
    .await
    .unwrap_or_default();

    for (hash,) in hash_rows {
        let trimmed = hash.trim().to_string();
        if !trimmed.is_empty() {
            iocs.insert(Ioc {
                value: trimmed,
                ioc_type: "hash".to_string(),
            });
        }
    }

    // URL extraction is intentionally omitted here because this pipeline reads
    // from the parser database only. URL enrichment is handled by deepmail-body
    // and deepmail-ioc in their own pipelines.

    Ok(iocs)
}

/// Extract domain from an email header value like "Name <user@example.com>".
fn extract_domain_from_header(value: &str) -> Option<String> {
    // Look for @ sign to extract domain
    let addr = if let Some(start) = value.rfind('<') {
        if let Some(end) = value.rfind('>') {
            &value[start + 1..end]
        } else {
            value
        }
    } else {
        value
    };

    if let Some(at_pos) = addr.rfind('@') {
        let domain = addr[at_pos + 1..].trim().to_lowercase();
        if !domain.is_empty() && domain.contains('.') {
            return Some(domain);
        }
    }

    None
}
