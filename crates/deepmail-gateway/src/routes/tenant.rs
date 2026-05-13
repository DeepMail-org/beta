use std::sync::Arc;

use axum::{extract::{Extension, State}, response::IntoResponse, Json};
use chrono::Utc;
use serde::Deserialize;
use serde_json::json;

use deepmail_common::proto::tenant;
use crate::auth_middleware::AuthClaims;
use crate::error::GatewayError;
use crate::GatewayCtx;

pub async fn get_tenant(
    State(ctx): State<Arc<GatewayCtx>>,
    Extension(claims): Extension<AuthClaims>,
) -> Result<impl IntoResponse, GatewayError> {
    let resp = ctx
        .tenant_client
        .get_tenant(&claims.tenant_id.to_string())
        .await?;

    Ok(Json(json!({
        "id": resp.id,
        "name": resp.name,
        "plan": resp.plan,
        "member_count": resp.member_count,
        "created_at": resp.created_at
    })))
}

#[derive(Deserialize)]
pub struct InviteBody {
    pub email: String,
    pub role: String,
}

pub async fn invite_member(
    State(ctx): State<Arc<GatewayCtx>>,
    Extension(claims): Extension<AuthClaims>,
    Json(body): Json<InviteBody>,
) -> Result<impl IntoResponse, GatewayError> {
    let resp = ctx
        .tenant_client
        .invite_member(tenant::InviteMemberRequest {
            tenant_id: claims.tenant_id.to_string(),
            email: body.email,
            role: body.role,
        })
        .await?;

    Ok(Json(json!({
        "invite_id": resp.invite_id,
        "sent": resp.sent
    })))
}

pub async fn usage(
    State(ctx): State<Arc<GatewayCtx>>,
    Extension(claims): Extension<AuthClaims>,
) -> Result<impl IntoResponse, GatewayError> {
    let period = Utc::now().format("%Y-%m").to_string();
    let resp = ctx
        .billing_client
        .get_usage(&claims.tenant_id.to_string(), &period)
        .await?;

    Ok(Json(json!({
        "tenant_id": resp.tenant_id,
        "billing_period": resp.billing_period,
        "total_paise": resp.total_paise,
        "event_count": resp.event_count
    })))
}
