use std::sync::Arc;

use axum::{extract::{Extension, State}, response::IntoResponse, Json};
use serde::Deserialize;
use serde_json::json;

use deepmail_common::proto::notify;
use crate::auth_middleware::AuthClaims;
use crate::error::GatewayError;
use crate::GatewayCtx;

pub async fn get_config(
    State(ctx): State<Arc<GatewayCtx>>,
    Extension(claims): Extension<AuthClaims>,
) -> Result<impl IntoResponse, GatewayError> {
    let resp = ctx
        .notify_client
        .get_config(&claims.tenant_id.to_string())
        .await?;

    Ok(Json(json!({
        "config_id": resp.config_id,
        "tenant_id": resp.tenant_id,
        "webhook_url": resp.webhook_url,
        "webhook_active": resp.webhook_active,
        "smtp_enabled": resp.smtp_enabled,
        "min_severity": resp.min_severity
    })))
}

#[derive(Deserialize)]
pub struct UpsertConfigBody {
    pub webhook_url: Option<String>,
    pub webhook_secret: Option<String>,
    pub smtp_enabled: Option<bool>,
    pub min_severity: Option<String>,
}

pub async fn upsert_config(
    State(ctx): State<Arc<GatewayCtx>>,
    Extension(claims): Extension<AuthClaims>,
    Json(body): Json<UpsertConfigBody>,
) -> Result<impl IntoResponse, GatewayError> {
    let resp = ctx
        .notify_client
        .upsert_config(notify::UpsertConfigRequest {
            tenant_id: claims.tenant_id.to_string(),
            webhook_url: body.webhook_url.unwrap_or_default(),
            webhook_secret: body.webhook_secret.unwrap_or_default(),
            smtp_enabled: body.smtp_enabled.unwrap_or(false),
            min_severity: body.min_severity.unwrap_or_default(),
        })
        .await?;

    Ok(Json(json!({
        "config_id": resp.config_id,
        "tenant_id": resp.tenant_id,
        "webhook_url": resp.webhook_url,
        "webhook_active": resp.webhook_active,
        "smtp_enabled": resp.smtp_enabled,
        "min_severity": resp.min_severity
    })))
}
