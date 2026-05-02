//! Razorpay webhook HTTP handler.
//!
//! Endpoint: POST /webhooks/razorpay
//!
//! Verifies HMAC-SHA256 signature using the webhook secret.
//! Stores the raw event for idempotent processing.
//! Dispatches to the appropriate handler by event type.

use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use sqlx::PgPool;

use crate::{config::Config, db, error::TenantError};

/// Shared state for the webhook handler.
pub struct WebhookState {
    pub pool: Arc<PgPool>,
    pub config: Arc<Config>,
}

/// Verify Razorpay webhook signature.
///
/// Razorpay sends: X-Razorpay-Signature: HMAC-SHA256(raw_body, webhook_secret)
fn verify_signature(
    body: &[u8],
    signature_header: &str,
    secret: &str,
) -> Result<(), TenantError> {
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|e| TenantError::Internal(e.to_string()))?;
    mac.update(body);
    let computed = hex::encode(mac.finalize().into_bytes());

    if computed != signature_header {
        return Err(TenantError::InvalidWebhookSignature);
    }
    Ok(())
}

/// POST /webhooks/razorpay
///
/// Returns 200 immediately after signature verification and DB insert.
/// Processing is async — the event is processed after the 200 response.
/// This is the correct Razorpay pattern (they retry on non-200).
pub async fn handle_razorpay_webhook(
    State(state): State<Arc<WebhookState>>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    // Extract signature header
    let signature = match headers
        .get("X-Razorpay-Signature")
        .and_then(|v| v.to_str().ok())
    {
        Some(s) => s.to_string(),
        None => {
            tracing::warn!("razorpay webhook missing signature header");
            return StatusCode::UNAUTHORIZED.into_response();
        }
    };

    // Verify HMAC-SHA256 signature
    if let Err(e) = verify_signature(
        &body,
        &signature,
        &state.config.razorpay_webhook_secret,
    ) {
        tracing::warn!(error = %e, "razorpay webhook signature verification failed");
        return StatusCode::UNAUTHORIZED.into_response();
    }

    // Parse JSON payload
    let payload: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(error = %e, "razorpay webhook payload parse error");
            return StatusCode::BAD_REQUEST.into_response();
        }
    };

    let event_id = match payload["id"].as_str() {
        Some(id) => id.to_string(),
        None => {
            tracing::error!("razorpay webhook missing event id");
            return StatusCode::BAD_REQUEST.into_response();
        }
    };

    let event_type = payload["event"]
        .as_str()
        .unwrap_or("unknown")
        .to_string();

    // Extract tenant from subscription entity if present
    let razorpay_subscription_id = payload["payload"]["subscription"]["entity"]["id"]
        .as_str()
        .map(|s| s.to_string());

    // Insert event — idempotent (ON CONFLICT DO NOTHING)
    let inserted = match db::webhooks::insert_webhook_event(
        &state.pool,
        &event_id,
        &event_type,
        None, // tenant resolved during processing
        &payload,
    )
    .await
    {
        Ok(inserted) => inserted,
        Err(e) => {
            tracing::error!(error = %e, "failed to store razorpay event");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if !inserted {
        tracing::debug!(event_id = %event_id, "duplicate razorpay event ignored");
        return StatusCode::OK.into_response();
    }

    // Spawn background processing (non-blocking response to Razorpay)
    let pool = Arc::clone(&state.pool);
    let event_id_bg = event_id.clone();
    let event_type_bg = event_type.clone();
    let payload_bg = payload.clone();
    let rz_sub_id = razorpay_subscription_id.clone();

    tokio::spawn(async move {
        if let Err(e) = process_webhook_event(
            &pool,
            &event_id_bg,
            &event_type_bg,
            &payload_bg,
            rz_sub_id.as_deref(),
        )
        .await
        {
            tracing::error!(
                event_id = %event_id_bg,
                event_type = %event_type_bg,
                error = %e,
                "razorpay event processing failed"
            );
            let _ = db::webhooks::mark_event_processed(
                &pool,
                &event_id_bg,
                Some(&e.to_string()),
            )
            .await;
        } else {
            let _ =
                db::webhooks::mark_event_processed(&pool, &event_id_bg, None).await;
        }
    });

    StatusCode::OK.into_response()
}

/// Process a verified Razorpay event by type.
async fn process_webhook_event(
    pool: &PgPool,
    event_id: &str,
    event_type: &str,
    payload: &serde_json::Value,
    razorpay_subscription_id: Option<&str>,
) -> Result<(), TenantError> {
    // `pool` is reserved for upcoming subscription upserts that need DB access;
    // current handlers only emit structured logs.
    let _ = pool;

    tracing::info!(
        event_id = %event_id,
        event_type = %event_type,
        "processing razorpay webhook event"
    );

    match event_type {
        "subscription.activated" | "subscription.charged" => {
            if let Some(rz_sub_id) = razorpay_subscription_id {
                let plan_id = payload["payload"]["subscription"]["entity"]["plan_id"]
                    .as_str()
                    .unwrap_or("");
                // We need tenant_id — look it up by razorpay_subscription_id.
                // For now, log; full upsert wires in once create_subscription
                // populates the tenant→subscription mapping.
                tracing::info!(
                    subscription_id = %rz_sub_id,
                    plan_id = %plan_id,
                    "subscription activated/charged"
                );
            }
        }
        "subscription.cancelled" | "subscription.expired" => {
            if let Some(rz_sub_id) = razorpay_subscription_id {
                tracing::info!(
                    subscription_id = %rz_sub_id,
                    event_type = %event_type,
                    "subscription cancelled or expired"
                );
            }
        }
        other => {
            tracing::debug!(event_type = %other, "unhandled razorpay event type");
        }
    }

    Ok(())
}
