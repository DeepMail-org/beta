use crate::config::BillingConfig;
use crate::db::invoices::{self, InvoiceRow};
use crate::error::BillingError;
use crate::meter;
use crate::razorpay::{LineItem, RazorpayClient};
use chrono::{Duration, Utc};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

pub struct BillingCtx {
    pub pool: Arc<PgPool>,
    pub auth_pool: Arc<PgPool>,
    pub tenant_pool: Arc<PgPool>,
    pub razorpay: Arc<RazorpayClient>,
    pub config: BillingConfig,
}

pub async fn generate_invoice_for_period(
    ctx: &BillingCtx,
    tenant_id: Uuid,
    period: &str,
) -> Result<InvoiceRow, BillingError> {
    if let Some(existing) = invoices::get_by_period(&ctx.pool, tenant_id, period)
        .await
        .map_err(BillingError::Db)?
    {
        if existing.status != "draft" {
            return Ok(existing);
        }
    }

    let usage = meter::get_usage(&ctx.pool, tenant_id, period).await?;

    if usage.event_count == 0 {
        return Err(BillingError::NotFound(format!(
            "no usage for tenant {} in period {}",
            tenant_id, period
        )));
    }

    let line_items: Vec<LineItem> = usage
        .cost_by_event_type
        .iter()
        .filter(|(_, &paise)| paise > 0)
        .map(|(event_type, &paise)| LineItem {
            name: event_type.clone(),
            amount_paise: paise,
            quantity: 1,
        })
        .collect();

    let line_items_json = serde_json::to_value(
        line_items
            .iter()
            .map(|li| {
                serde_json::json!({
                    "name": li.name,
                    "amount": li.amount_paise,
                    "quantity": li.quantity,
                })
            })
            .collect::<Vec<_>>(),
    )
    .unwrap_or_default();

    let mut razorpay_id: Option<String> = None;
    let mut status = "draft";
    let mut due_at = None;

    if ctx.razorpay.is_configured() {
        let (tenant_name, billing_email) =
            fetch_tenant_owner(ctx, tenant_id).await.unwrap_or_else(|e| {
                tracing::warn!(
                    tenant_id = %tenant_id,
                    error = %e,
                    "could not fetch tenant owner, using fallback"
                );
                (
                    format!("tenant-{}", tenant_id),
                    "billing@deepmail.io".to_string(),
                )
            });

        match ctx
            .razorpay
            .create_invoice(&tenant_name, &billing_email, &line_items, period)
            .await
        {
            Ok(rp_id) => {
                if let Err(e) = ctx.razorpay.issue_invoice(&rp_id).await {
                    tracing::warn!(razorpay_id = rp_id.as_str(), error = %e, "failed to issue invoice");
                } else {
                    status = "issued";
                }
                due_at = Some(Utc::now() + Duration::days(30));
                razorpay_id = Some(rp_id);
            }
            Err(e) => {
                tracing::error!(error = %e, "failed to create Razorpay invoice");
            }
        }
    }

    let invoice_id = invoices::upsert_invoice(
        &ctx.pool,
        tenant_id,
        period,
        razorpay_id.as_deref(),
        status,
        usage.total_paise,
        &line_items_json,
        due_at,
    )
    .await
    .map_err(BillingError::Db)?;

    invoices::get_by_period(&ctx.pool, tenant_id, period)
        .await
        .map_err(BillingError::Db)?
        .ok_or_else(|| {
            BillingError::Internal(format!(
                "invoice {} not found after upsert",
                invoice_id
            ))
        })
}

async fn fetch_tenant_owner(
    ctx: &BillingCtx,
    tenant_id: Uuid,
) -> Result<(String, String), BillingError> {
    let owner_row: Option<(Uuid,)> = sqlx::query_as(
        "SELECT user_id FROM tenant_members WHERE tenant_id = $1 AND role = 'owner' LIMIT 1",
    )
    .bind(tenant_id)
    .fetch_optional(ctx.tenant_pool.as_ref())
    .await
    .map_err(BillingError::Db)?;

    let owner_id = owner_row
        .map(|(id,)| id)
        .ok_or_else(|| BillingError::NotFound(format!("no owner for tenant {}", tenant_id)))?;

    let user_row: Option<(String, String)> = sqlx::query_as(
        "SELECT username, CAST(email AS TEXT) AS email FROM users WHERE id = $1 AND is_active = true AND deleted_at IS NULL",
    )
    .bind(owner_id)
    .fetch_optional(ctx.auth_pool.as_ref())
    .await
    .map_err(BillingError::Db)?;

    user_row.ok_or_else(|| {
        BillingError::NotFound(format!("owner user {} not found or inactive", owner_id))
    })
}
