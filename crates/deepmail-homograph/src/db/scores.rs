/// Domain score database operations.

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::HomographError;

/// Insert a domain score record.
pub async fn insert_domain_score(
    pool: &PgPool,
    analysis_id: Uuid,
    domain: &str,
    decoded_domain: &str,
    skeleton: &str,
    best_brand_match: &str,
    raw_similarity: f32,
    final_score: f32,
    edit_distance: i32,
    mixed_script: bool,
    punycode_abuse: bool,
    risk_level: &str,
) -> Result<(), HomographError> {
    sqlx::query(
        r#"INSERT INTO domain_scores
             (analysis_id, domain, decoded_domain, skeleton, best_brand_match,
              raw_similarity, final_score, edit_distance, mixed_script,
              punycode_abuse, risk_level)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"#,
    )
    .bind(analysis_id)
    .bind(domain)
    .bind(decoded_domain)
    .bind(skeleton)
    .bind(best_brand_match)
    .bind(raw_similarity)
    .bind(final_score)
    .bind(edit_distance)
    .bind(mixed_script)
    .bind(punycode_abuse)
    .bind(risk_level)
    .execute(pool)
    .await?;

    Ok(())
}

/// Domain score row.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DomainScoreRow {
    pub id: Uuid,
    pub domain: String,
    pub decoded_domain: String,
    pub skeleton: String,
    pub best_brand_match: String,
    pub raw_similarity: f32,
    pub final_score: f32,
    pub edit_distance: i32,
    pub mixed_script: bool,
    pub punycode_abuse: bool,
    pub risk_level: String,
}

/// List all domain scores for an analysis.
pub async fn list_by_analysis(
    pool: &PgPool,
    analysis_id: Uuid,
) -> Result<Vec<DomainScoreRow>, HomographError> {
    let rows = sqlx::query_as::<_, DomainScoreRow>(
        r#"SELECT id, domain, decoded_domain, skeleton, best_brand_match,
                  raw_similarity, final_score, edit_distance, mixed_script,
                  punycode_abuse, risk_level
           FROM domain_scores
           WHERE analysis_id = $1
           ORDER BY final_score DESC"#,
    )
    .bind(analysis_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}
