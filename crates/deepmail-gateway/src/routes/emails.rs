use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::auth_middleware::AuthClaims;
use crate::error::GatewayError;
use crate::GatewayCtx;

fn parse_email_id(id: &str) -> Result<Uuid, GatewayError> {
    Uuid::parse_str(id).map_err(|_| GatewayError::Validation("invalid email_id UUID".into()))
}

pub async fn upload(
    State(ctx): State<Arc<GatewayCtx>>,
    Extension(claims): Extension<AuthClaims>,
    req: axum::http::Request<Body>,
) -> Result<impl IntoResponse, GatewayError> {
    let content_type = req
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();

    let body_bytes = axum::body::to_bytes(req.into_body(), ctx.config.max_upload_bytes)
        .await
        .map_err(|_| GatewayError::Validation("body too large".into()))?;

    let resp = ctx
        .http_client
        .post(format!("{}/api/v1/upload", ctx.config.ingest_http_url))
        .header("content-type", &content_type)
        .header("x-deepmail-user-id", claims.user_id.to_string())
        .header("x-deepmail-tenant-id", claims.tenant_id.to_string())
        .body(body_bytes)
        .send()
        .await?;

    let status = StatusCode::from_u16(resp.status().as_u16())
        .unwrap_or(StatusCode::BAD_GATEWAY);
    let body: serde_json::Value = resp.json().await.unwrap_or(json!({"error": "upstream error"}));

    Ok((status, Json(body)))
}

pub async fn status(
    State(ctx): State<Arc<GatewayCtx>>,
    Extension(claims): Extension<AuthClaims>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, GatewayError> {
    let email_id = parse_email_id(&id)?;

    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT stage, status FROM job_progress WHERE email_id = $1 AND tenant_id = $2",
    )
    .bind(email_id)
    .bind(claims.tenant_id)
    .fetch_all(ctx.ingest_pool.as_ref())
    .await?;

    if rows.is_empty() {
        return Ok((StatusCode::NOT_FOUND, Json(json!({"error": "not found"}))));
    }

    let total_steps = rows.len() as i32;
    let completed_steps = rows.iter().filter(|(_, s)| s == "completed").count() as i32;
    let current_step = rows
        .iter()
        .find(|(_, s)| s == "running")
        .map(|(stage, _)| stage.as_str())
        .unwrap_or("none");

    let overall_status = if completed_steps == total_steps {
        "completed"
    } else if rows.iter().any(|(_, s)| s == "failed") {
        "failed"
    } else if rows.iter().any(|(_, s)| s == "running") {
        "running"
    } else {
        "pending"
    };

    Ok((
        StatusCode::OK,
        Json(json!({
            "email_id": email_id,
            "current_step": current_step,
            "completed_steps": completed_steps,
            "total_steps": total_steps,
            "status": overall_status
        })),
    ))
}

pub async fn score(
    State(ctx): State<Arc<GatewayCtx>>,
    Extension(claims): Extension<AuthClaims>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, GatewayError> {
    let _ = parse_email_id(&id)?;
    let resp = ctx.scoring_client.get_score(&id, &claims.tenant_id.to_string()).await?;

    Ok(Json(json!({
        "email_id": resp.email_id,
        "final_score": resp.final_score,
        "final_verdict": resp.final_verdict,
        "signals_available": resp.signals_available,
        "is_final": resp.is_final
    })))
}

pub async fn report(
    State(ctx): State<Arc<GatewayCtx>>,
    Extension(claims): Extension<AuthClaims>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, GatewayError> {
    let _ = parse_email_id(&id)?;
    let resp = ctx
        .report_client
        .get_report(&id, &claims.tenant_id.to_string(), "json")
        .await?;

    Ok(Json(json!({
        "report_id": resp.report_id,
        "email_id": resp.email_id,
        "json_s3_key": resp.json_s3_key,
        "html_s3_key": resp.html_s3_key,
        "status": resp.status,
        "final_verdict": resp.final_verdict,
        "final_score": resp.final_score
    })))
}

pub async fn iocs(
    State(ctx): State<Arc<GatewayCtx>>,
    Extension(claims): Extension<AuthClaims>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, GatewayError> {
    let _ = parse_email_id(&id)?;
    let resp = ctx
        .ioc_client
        .get_email_iocs(&id, &claims.tenant_id.to_string())
        .await?;

    let iocs: Vec<serde_json::Value> = resp
        .iocs
        .into_iter()
        .map(|i| {
            json!({
                "id": i.id,
                "ioc_type": i.ioc_type,
                "ioc_value": i.ioc_value,
                "threat_level": i.threat_level,
                "intel_score": i.intel_score,
                "sighting_count": i.sighting_count
            })
        })
        .collect();

    Ok(Json(json!({ "iocs": iocs })))
}

#[derive(Deserialize)]
pub struct GraphQuery {
    pub ioc_value: Option<String>,
    pub ioc_type: Option<String>,
    pub depth: Option<i32>,
}

pub async fn graph(
    State(ctx): State<Arc<GatewayCtx>>,
    Extension(claims): Extension<AuthClaims>,
    Path(id): Path<String>,
    Query(params): Query<GraphQuery>,
) -> Result<impl IntoResponse, GatewayError> {
    let _ = parse_email_id(&id)?;
    let ioc_value = params.ioc_value.unwrap_or_default();
    let ioc_type = params.ioc_type.unwrap_or_default();
    let depth = params.depth.unwrap_or(2);

    let resp = ctx
        .graph_client
        .query_related(&ioc_value, &ioc_type, depth, &claims.tenant_id.to_string())
        .await?;

    let nodes: Vec<serde_json::Value> = resp
        .nodes
        .into_iter()
        .map(|n| {
            json!({
                "id": n.id,
                "node_type": n.node_type,
                "value": n.value,
                "threat_score": n.threat_score
            })
        })
        .collect();

    let edges: Vec<serde_json::Value> = resp
        .edges
        .into_iter()
        .map(|e| {
            json!({
                "source_id": e.source_id,
                "target_id": e.target_id,
                "relation_type": e.relation_type,
                "confidence": e.confidence
            })
        })
        .collect();

    Ok(Json(json!({ "nodes": nodes, "edges": edges })))
}
