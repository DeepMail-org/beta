//! Fuzzy hash similarity clustering using ssdeep.
//!
//! Compares a new ssdeep hash against recent hashes in the database.
//! Any pair with similarity >= threshold gets a cluster record.
//! All comparison work runs in tokio::task::spawn_blocking.

use uuid::Uuid;

use crate::{config::Config, db, error::HashDbError};

/// Cluster a newly registered hash against recent ssdeep hashes.
///
/// Fetches up to config.ssdeep_cluster_limit recent hashes from DB,
/// compares each pair, and inserts cluster records for matches.
/// Fire-and-forget in production — the caller spawns this as a background task.
pub async fn run_ssdeep_clustering(
    pool: std::sync::Arc<sqlx::PgPool>,
    new_hash_id: Uuid,
    new_ssdeep: String,
    new_sha256: String,
    config: std::sync::Arc<Config>,
) -> Result<(), HashDbError> {
    // Fetch recent hashes with ssdeep values
    let candidates =
        db::hashes::get_recent_ssdeep_hashes(&pool, config.ssdeep_cluster_limit, &new_sha256)
            .await?;

    if candidates.is_empty() {
        return Ok(());
    }

    let threshold = config.ssdeep_threshold;

    // ssdeep comparison is CPU-bound — run in spawn_blocking
    let matches: Vec<(Uuid, u32)> =
        // ssdeep::compare returns Result<u8, Error>, cast to u32 for consistency
        tokio::task::spawn_blocking(move || {
            candidates
                .into_iter()
                .filter_map(|(existing_id, existing_ssdeep)| {
                    // ssdeep::compare returns Result<u8, Error> (0–100 similarity)
                    let score = ssdeep::compare(&new_ssdeep, &existing_ssdeep)
                        .unwrap_or(0);
                    let score_u32 = u32::from(score);
                    if score_u32 >= threshold {
                        Some((existing_id, score_u32))
                    } else {
                        None
                    }
                })
                .collect()
        })
        .await
        .map_err(|e| HashDbError::Internal(format!("spawn_blocking join: {e}")))?;

    // Insert cluster records for all matches
    for (existing_id, score) in matches {
        db::clusters::insert_cluster(
            &pool,
            existing_id,        // earlier-registered = representative
            new_hash_id,        // newer = cluster member
            score as i32,
            "ssdeep",
        )
        .await?;
    }

    Ok(())
}
