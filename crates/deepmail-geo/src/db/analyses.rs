use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::GeoError;

pub struct AnalysisRow {
    pub id: Uuid,
    pub email_id: Uuid,
    pub tenant_id: Uuid,
    pub hop_count: i32,
    pub origin_ip: Option<String>,
    pub origin_country: Option<String>,
    pub origin_asn: Option<i32>,
    pub overall_risk: String,
    pub risk_score: f32,
    pub analyzed_at: DateTime<Utc>,
}

pub async fn upsert_analysis(
    pool: &PgPool,
    email_id: Uuid,
    tenant_id: Uuid,
    hop_count: i32,
    origin_ip: Option<&str>,
    origin_country: Option<&str>,
    origin_asn: Option<i32>,
    overall_risk: &str,
    risk_score: f32,
) -> Result<Uuid, GeoError> {
    let row = sqlx::query(
        r#"INSERT INTO email_geo_analyses (
             email_id, tenant_id, hop_count,
             origin_ip, origin_country, origin_asn,
             overall_risk, risk_score
           ) VALUES ($1, $2, $3, $4::inet, $5, $6, $7, $8)
           ON CONFLICT (email_id) DO UPDATE SET
             hop_count      = EXCLUDED.hop_count,
             origin_ip      = EXCLUDED.origin_ip,
             origin_country = EXCLUDED.origin_country,
             origin_asn     = EXCLUDED.origin_asn,
             overall_risk   = EXCLUDED.overall_risk,
             risk_score     = EXCLUDED.risk_score,
             analyzed_at    = now()
           RETURNING id"#,
    )
    .bind(email_id)
    .bind(tenant_id)
    .bind(hop_count)
    .bind(origin_ip)
    .bind(origin_country)
    .bind(origin_asn)
    .bind(overall_risk)
    .bind(risk_score)
    .fetch_one(pool)
    .await?;

    use sqlx::Row;
    let id: Uuid = row.get("id");
    Ok(id)
}

pub async fn get_by_email_id(
    pool: &PgPool,
    email_id: Uuid,
) -> Result<Option<AnalysisRow>, GeoError> {
    let row = sqlx::query(
        r#"SELECT
             id, email_id, tenant_id, hop_count,
             origin_ip::text, origin_country, origin_asn,
             overall_risk, risk_score, analyzed_at
           FROM email_geo_analyses
           WHERE email_id = $1"#,
    )
    .bind(email_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| {
        use sqlx::Row;
        AnalysisRow {
            id: r.get("id"),
            email_id: r.get("email_id"),
            tenant_id: r.get("tenant_id"),
            hop_count: r.get("hop_count"),
            origin_ip: r.get("origin_ip"),
            origin_country: r.get("origin_country"),
            origin_asn: r.get("origin_asn"),
            overall_risk: r.get("overall_risk"),
            risk_score: r.get("risk_score"),
            analyzed_at: r.get("analyzed_at"),
        }
    }))
}
