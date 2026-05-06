//! CRUD for the file_hashes table.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::HashDbError;

/// A file hash row returned from the database.
#[derive(Debug, Clone)]
pub struct HashRow {
    pub id: Uuid,
    pub sha256: String,
    pub md5: String,
    pub ssdeep: Option<String>,
    pub imphash: Option<String>,
    pub file_type: String,
    pub verdict: String,
    pub verdict_confidence: f32,
    pub malware_family: Option<String>,
    pub analysis_required: bool,
    pub seen_count: i32,
    pub first_seen: DateTime<Utc>,
}

/// Look up a hash by its SHA-256 value.
/// Returns None if not found.
pub async fn get_by_sha256(
    pool: &PgPool,
    sha256: &str,
) -> Result<Option<HashRow>, HashDbError> {
    let row = sqlx::query!(
        r#"
        SELECT id, sha256, md5, ssdeep, imphash, file_type,
               verdict, verdict_confidence, malware_family,
               analysis_required, seen_count, first_seen
        FROM file_hashes
        WHERE sha256 = $1
        "#,
        sha256,
    )
    .fetch_optional(pool)
    .await
    .map_err(HashDbError::Database)?;

    Ok(row.map(|r| HashRow {
        id: r.id,
        sha256: r.sha256,
        md5: r.md5,
        ssdeep: r.ssdeep,
        imphash: r.imphash,
        file_type: r.file_type,
        verdict: r.verdict,
        verdict_confidence: r.verdict_confidence,
        malware_family: r.malware_family,
        analysis_required: r.analysis_required,
        seen_count: r.seen_count,
        first_seen: r.first_seen,
    }))
}

/// Fetch multiple hashes by SHA-256 in a single query.
/// Returns only the hashes that were found. Order is not preserved.
pub async fn get_many_by_sha256(
    pool: &PgPool,
    sha256_hashes: &[String],
) -> Result<Vec<HashRow>, HashDbError> {
    if sha256_hashes.is_empty() {
        return Ok(Vec::new());
    }

    let rows = sqlx::query!(
        r#"
        SELECT id, sha256, md5, ssdeep, imphash, file_type,
               verdict, verdict_confidence, malware_family,
               analysis_required, seen_count, first_seen
        FROM file_hashes
        WHERE sha256 = ANY($1)
        "#,
        sha256_hashes,
    )
    .fetch_all(pool)
    .await
    .map_err(HashDbError::Database)?;

    Ok(rows
        .into_iter()
        .map(|r| HashRow {
            id: r.id,
            sha256: r.sha256,
            md5: r.md5,
            ssdeep: r.ssdeep,
            imphash: r.imphash,
            file_type: r.file_type,
            verdict: r.verdict,
            verdict_confidence: r.verdict_confidence,
            malware_family: r.malware_family,
            analysis_required: r.analysis_required,
            seen_count: r.seen_count,
            first_seen: r.first_seen,
        })
        .collect())
}

/// Input for registering a new hash or updating an existing one.
pub struct HashRegisterInput<'a> {
    pub sha256: &'a str,
    pub md5: &'a str,
    pub sha1: Option<&'a str>,
    pub ssdeep: Option<&'a str>,
    pub tlsh: Option<&'a str>,
    pub imphash: Option<&'a str>,
    pub file_type: &'a str,
    pub file_size_bytes: i64,
    pub verdict: &'a str,
    pub verdict_confidence: f32,
    pub verdict_source: Option<&'a str>,
    pub malware_family: Option<&'a str>,
}

/// Insert a new hash or update seen_count + verdict if already exists.
/// Returns (hash_id, was_new).
pub async fn upsert_hash(
    pool: &PgPool,
    input: HashRegisterInput<'_>,
) -> Result<(Uuid, bool), HashDbError> {
    let row = sqlx::query!(
        r#"
        INSERT INTO file_hashes (
          sha256, md5, sha1, ssdeep, tlsh, imphash,
          file_type, file_size_bytes,
          verdict, verdict_confidence, verdict_source,
          malware_family, analysis_required
        )
        VALUES (
          $1, $2, $3, $4, $5, $6,
          $7, $8,
          $9, $10, $11,
          $12,
          CASE WHEN $9 = 'unknown' THEN true ELSE false END
        )
        ON CONFLICT (sha256) DO UPDATE
          SET seen_count        = file_hashes.seen_count + 1,
              last_seen         = now(),
              updated_at        = now(),
              verdict           = CASE
                WHEN EXCLUDED.verdict != 'unknown'
                THEN EXCLUDED.verdict
                ELSE file_hashes.verdict
              END,
              verdict_confidence = CASE
                WHEN EXCLUDED.verdict != 'unknown'
                THEN EXCLUDED.verdict_confidence
                ELSE file_hashes.verdict_confidence
              END,
              malware_family    = COALESCE(EXCLUDED.malware_family, file_hashes.malware_family),
              analysis_required = CASE
                WHEN EXCLUDED.verdict != 'unknown'
                THEN false
                ELSE file_hashes.analysis_required
              END
        RETURNING id,
                  (xmax::text = '0') AS "was_new!: bool"
        "#,
        input.sha256,
        input.md5,
        input.sha1,
        input.ssdeep,
        input.tlsh,
        input.imphash,
        input.file_type,
        input.file_size_bytes,
        input.verdict,
        input.verdict_confidence,
        input.verdict_source,
        input.malware_family,
    )
    .fetch_one(pool)
    .await
    .map_err(HashDbError::Database)?;

    Ok((row.id, row.was_new))
}

/// Fetch up to `limit` most recent hashes that have ssdeep values.
/// Used for fuzzy clustering comparison.
pub async fn get_recent_ssdeep_hashes(
    pool: &PgPool,
    limit: i64,
    exclude_sha256: &str,
) -> Result<Vec<(Uuid, String)>, HashDbError> {
    let rows = sqlx::query!(
        r#"
        SELECT id, ssdeep AS "ssdeep!: String"
        FROM file_hashes
        WHERE ssdeep IS NOT NULL
          AND sha256 != $2
        ORDER BY last_seen DESC
        LIMIT $1
        "#,
        limit,
        exclude_sha256,
    )
    .fetch_all(pool)
    .await
    .map_err(HashDbError::Database)?;

    Ok(rows.into_iter().map(|r| (r.id, r.ssdeep)).collect())
}
