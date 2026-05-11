use crate::signals::{self, SignalName};
use sqlx::postgres::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub struct ServicePools {
    pub header: Arc<PgPool>,
    pub dkim: Arc<PgPool>,
    pub geo: Arc<PgPool>,
    pub ip: Arc<PgPool>,
    pub intel: Arc<PgPool>,
    pub ioc: Arc<PgPool>,
    pub homograph: Arc<PgPool>,
    pub body: Arc<PgPool>,
    pub sandbox_url: Arc<PgPool>,
    pub sandbox_file: Arc<PgPool>,
    pub sandbox_dynamic: Arc<PgPool>,
}

async fn fetch_header_score(pool: &PgPool, email_id: Uuid) -> anyhow::Result<Option<f32>> {
    let row: Option<(f32,)> = sqlx::query_as(
        "SELECT overall_risk_score FROM header_analyses WHERE email_id = $1 LIMIT 1",
    )
    .bind(email_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| r.0.clamp(0.0, 1.0)))
}

async fn fetch_dkim_score(pool: &PgPool, email_id: Uuid) -> anyhow::Result<Option<f32>> {
    let row: Option<(f32,)> =
        sqlx::query_as("SELECT replay_confidence FROM dkim_analyses WHERE email_id = $1 LIMIT 1")
            .bind(email_id)
            .fetch_optional(pool)
            .await?;
    Ok(row.map(|r| r.0))
}

async fn fetch_geo_score(pool: &PgPool, email_id: Uuid) -> anyhow::Result<Option<f32>> {
    let row: Option<(f32,)> = sqlx::query_as(
        "SELECT risk_score FROM email_geo_analyses WHERE email_id = $1 LIMIT 1",
    )
    .bind(email_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| r.0))
}

async fn fetch_ip_score(pool: &PgPool, email_id: Uuid) -> anyhow::Result<Option<f32>> {
    let row: Option<(f32,)> = sqlx::query_as(
        "SELECT max_threat_score FROM email_ip_analyses WHERE email_id = $1 LIMIT 1",
    )
    .bind(email_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| r.0))
}

async fn fetch_intel_score(pool: &PgPool, email_id: Uuid) -> anyhow::Result<Option<f32>> {
    let row: Option<(f32,)> = sqlx::query_as(
        "SELECT max_vt_score FROM email_intel_results WHERE email_id = $1 LIMIT 1",
    )
    .bind(email_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| r.0))
}

async fn fetch_ioc_score(pool: &PgPool, email_id: Uuid) -> anyhow::Result<Option<f32>> {
    let row: Option<(i64, i64)> = sqlx::query_as(
        r#"SELECT
             COUNT(*) FILTER (WHERE n.threat_level = 'MALICIOUS'),
             COUNT(*)
           FROM email_ioc_occurrences o
           JOIN ioc_nodes n ON n.id = o.ioc_node_id
           WHERE o.email_id = $1"#,
    )
    .bind(email_id)
    .fetch_optional(pool)
    .await?;
    match row {
        Some((_, 0)) | None => Ok(None),
        Some((malicious, total)) => Ok(Some((malicious as f32) / (total as f32))),
    }
}

async fn fetch_homograph_score(pool: &PgPool, email_id: Uuid) -> anyhow::Result<Option<f32>> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT overall_risk FROM homograph_analyses WHERE email_id = $1 LIMIT 1",
    )
    .bind(email_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| signals::risk_text_to_float(&r.0)))
}

async fn fetch_body_score(pool: &PgPool, email_id: Uuid) -> anyhow::Result<Option<f32>> {
    let row: Option<(f32,)> = sqlx::query_as(
        "SELECT final_phishing_score FROM body_analyses WHERE email_id = $1 LIMIT 1",
    )
    .bind(email_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| r.0))
}

async fn fetch_sandbox_url_score(pool: &PgPool, email_id: Uuid) -> anyhow::Result<Option<f32>> {
    let row: Option<(Option<f32>,)> = sqlx::query_as(
        "SELECT MAX(threat_score) FROM url_sandbox_reports WHERE email_id = $1",
    )
    .bind(email_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.and_then(|r| r.0))
}

async fn fetch_sandbox_file_score(pool: &PgPool, email_id: Uuid) -> anyhow::Result<Option<f32>> {
    let row: Option<(Option<f32>,)> = sqlx::query_as(
        "SELECT MAX(threat_score) FROM file_static_reports WHERE email_id = $1",
    )
    .bind(email_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.and_then(|r| r.0))
}

async fn fetch_sandbox_dynamic_score(
    pool: &PgPool,
    email_id: Uuid,
) -> anyhow::Result<Option<f32>> {
    let row: Option<(Option<f32>,)> = sqlx::query_as(
        "SELECT MAX(dynamic_score) FROM dynamic_analysis_reports WHERE email_id = $1",
    )
    .bind(email_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.and_then(|r| r.0))
}

pub async fn fetch_all_signals(
    pools: &ServicePools,
    email_id: Uuid,
) -> HashMap<SignalName, f32> {
    let (header, dkim, geo, ip, intel, ioc, homograph, body, s_url, s_file, s_dyn) = tokio::join!(
        fetch_header_score(&pools.header, email_id),
        fetch_dkim_score(&pools.dkim, email_id),
        fetch_geo_score(&pools.geo, email_id),
        fetch_ip_score(&pools.ip, email_id),
        fetch_intel_score(&pools.intel, email_id),
        fetch_ioc_score(&pools.ioc, email_id),
        fetch_homograph_score(&pools.homograph, email_id),
        fetch_body_score(&pools.body, email_id),
        fetch_sandbox_url_score(&pools.sandbox_url, email_id),
        fetch_sandbox_file_score(&pools.sandbox_file, email_id),
        fetch_sandbox_dynamic_score(&pools.sandbox_dynamic, email_id),
    );

    let mut signals = HashMap::new();

    let pairs: [(SignalName, anyhow::Result<Option<f32>>); 11] = [
        (SignalName::Header, header),
        (SignalName::Dkim, dkim),
        (SignalName::Geo, geo),
        (SignalName::Ip, ip),
        (SignalName::Intel, intel),
        (SignalName::Ioc, ioc),
        (SignalName::Homograph, homograph),
        (SignalName::Body, body),
        (SignalName::SandboxUrl, s_url),
        (SignalName::SandboxFile, s_file),
        (SignalName::SandboxDynamic, s_dyn),
    ];

    for (name, result) in pairs {
        match result {
            Ok(Some(score)) => {
                signals.insert(name, score.clamp(0.0, 1.0));
            }
            Ok(None) => {}
            Err(e) => {
                tracing::warn!(signal = name.as_str(), error = %e, "failed to fetch signal score");
            }
        }
    }

    signals
}
