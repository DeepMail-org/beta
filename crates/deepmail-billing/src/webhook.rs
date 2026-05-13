use crate::db::invoices;
use crate::invoice::BillingCtx;
use crate::razorpay::RazorpayClient;
use axum::body::Bytes;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Extension;
use std::sync::Arc;

pub async fn razorpay_webhook(
    headers: HeaderMap,
    Extension(ctx): Extension<Arc<BillingCtx>>,
    body: Bytes,
) -> impl IntoResponse {
    let signature = match headers
        .get("X-Razorpay-Signature")
        .and_then(|v| v.to_str().ok())
    {
        Some(s) => s.to_string(),
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                "missing X-Razorpay-Signature header",
            )
                .into_response();
        }
    };

    if ctx.config.razorpay_webhook_secret.is_empty() {
        tracing::warn!("razorpay webhook secret not configured, rejecting");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "webhook secret not configured",
        )
            .into_response();
    }

    if !RazorpayClient::verify_webhook_signature(
        &body,
        &signature,
        &ctx.config.razorpay_webhook_secret,
    ) {
        tracing::warn!("invalid Razorpay webhook signature");
        return (StatusCode::UNAUTHORIZED, "invalid signature").into_response();
    }

    let payload: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(error = %e, "failed to parse webhook payload");
            return (StatusCode::BAD_REQUEST, "invalid JSON").into_response();
        }
    };

    let event_type = payload["event"].as_str().unwrap_or("").to_string();
    let event_id = payload["event_id"]
        .as_str()
        .or_else(|| payload["id"].as_str())
        .unwrap_or("")
        .to_string();

    if event_id.is_empty() {
        return (StatusCode::BAD_REQUEST, "missing event_id").into_response();
    }

    let is_new = match invoices::insert_razorpay_event(
        &ctx.pool,
        &event_id,
        &event_type,
        &payload,
    )
    .await
    {
        Ok(new) => new,
        Err(e) => {
            tracing::error!(error = %e, "failed to insert razorpay event");
            return (StatusCode::INTERNAL_SERVER_ERROR, "database error").into_response();
        }
    };

    if !is_new {
        return (StatusCode::OK, "already processed").into_response();
    }

    let razorpay_invoice_id = payload["payload"]["invoice"]["entity"]["id"]
        .as_str()
        .or_else(|| payload["payload"]["payment"]["entity"]["invoice_id"].as_str())
        .unwrap_or("");

    if !razorpay_invoice_id.is_empty() {
        let result = match event_type.as_str() {
            "invoice.paid" => {
                invoices::update_status_paid(&ctx.pool, razorpay_invoice_id).await
            }
            "invoice.cancelled" | "invoice.expired" => {
                invoices::update_status(&ctx.pool, razorpay_invoice_id, "cancelled").await
            }
            "payment.captured" => {
                invoices::update_status_paid(&ctx.pool, razorpay_invoice_id).await
            }
            _ => {
                tracing::debug!(event_type = event_type.as_str(), "unhandled webhook event type");
                Ok(())
            }
        };

        if let Err(e) = result {
            tracing::error!(
                event_type = event_type.as_str(),
                razorpay_id = razorpay_invoice_id,
                error = %e,
                "failed to update invoice status"
            );
        }
    }

    if let Err(e) = invoices::mark_razorpay_processed(&ctx.pool, &event_id).await {
        tracing::error!(event_id = event_id.as_str(), error = %e, "failed to mark event processed");
    }

    (StatusCode::OK, "ok").into_response()
}
