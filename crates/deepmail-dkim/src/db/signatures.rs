use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::DkimError;

pub async fn insert_signature(
    pool: &PgPool,
    analysis_id: Uuid,
    selector: &str,
    domain: &str,
    algorithm: &str,
    canonicalization: &str,
    signed_headers: &[String],
    body_hash_claimed: &str,
    body_hash_computed: &str,
    body_hash_match: bool,
    signature_timestamp: Option<DateTime<Utc>>,
    signature_valid: bool,
    key_found_in_dns: bool,
) -> Result<Uuid, DkimError> {
    let id = sqlx::query_scalar!(
        r#"INSERT INTO dkim_signatures (
             analysis_id, selector, domain, algorithm,
             canonicalization, signed_headers,
             body_hash_claimed, body_hash_computed, body_hash_match,
             signature_timestamp, signature_valid, key_found_in_dns
           ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
           RETURNING id"#,
        analysis_id,
        selector,
        domain,
        algorithm,
        canonicalization,
        signed_headers,
        body_hash_claimed,
        body_hash_computed,
        body_hash_match,
        signature_timestamp,
        signature_valid,
        key_found_in_dns,
    )
    .fetch_one(pool)
    .await?;

    Ok(id)
}

pub async fn list_by_analysis(
    pool: &PgPool,
    analysis_id: Uuid,
) -> Result<Vec<SignatureRow>, DkimError> {
    let rows = sqlx::query_as!(
        SignatureRow,
        r#"SELECT
             id, analysis_id, selector, domain, algorithm,
             canonicalization, signed_headers,
             body_hash_claimed, body_hash_computed, body_hash_match,
             signature_timestamp, signature_valid, key_found_in_dns
           FROM dkim_signatures
           WHERE analysis_id = $1
           ORDER BY created_at"#,
        analysis_id,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub struct SignatureRow {
    pub id: Uuid,
    pub analysis_id: Uuid,
    pub selector: String,
    pub domain: String,
    pub algorithm: String,
    pub canonicalization: String,
    pub signed_headers: Vec<String>,
    pub body_hash_claimed: String,
    pub body_hash_computed: String,
    pub body_hash_match: bool,
    pub signature_timestamp: Option<DateTime<Utc>>,
    pub signature_valid: bool,
    pub key_found_in_dns: bool,
}
