use crate::db::invoices;
use crate::invoice::{self, BillingCtx};
use crate::meter;
use chrono::Utc;
use deepmail_common::proto::billing::billing_service_server::BillingService;
use deepmail_common::proto::billing::{
    GetInvoiceRequest, GetUsageRequest, GetUsageResponse, InvoiceRequest, InvoiceResponse,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

pub struct BillingGrpcService {
    ctx: Arc<BillingCtx>,
}

impl BillingGrpcService {
    pub fn new(ctx: Arc<BillingCtx>) -> Self {
        Self { ctx }
    }
}

#[tonic::async_trait]
impl BillingService for BillingGrpcService {
    async fn get_usage(
        &self,
        request: Request<GetUsageRequest>,
    ) -> Result<Response<GetUsageResponse>, Status> {
        let req = request.into_inner();

        let tenant_id = Uuid::parse_str(&req.tenant_id)
            .map_err(|_| Status::invalid_argument("invalid tenant_id"))?;

        let period = if req.billing_period.is_empty() {
            Utc::now().format("%Y-%m").to_string()
        } else {
            req.billing_period
        };

        let usage = meter::get_usage(&self.ctx.pool, tenant_id, &period)
            .await
            .map_err(|e| Status::from(e))?;

        Ok(Response::new(GetUsageResponse {
            tenant_id: tenant_id.to_string(),
            billing_period: period,
            total_paise: usage.total_paise,
            event_count: usage.event_count,
            cost_by_event_type: usage.cost_by_event_type,
        }))
    }

    async fn generate_invoice(
        &self,
        request: Request<InvoiceRequest>,
    ) -> Result<Response<InvoiceResponse>, Status> {
        let req = request.into_inner();

        let tenant_id = Uuid::parse_str(&req.tenant_id)
            .map_err(|_| Status::invalid_argument("invalid tenant_id"))?;

        let period = if req.billing_period.is_empty() {
            Utc::now().format("%Y-%m").to_string()
        } else {
            req.billing_period
        };

        let row = invoice::generate_invoice_for_period(&self.ctx, tenant_id, &period)
            .await
            .map_err(|e| Status::from(e))?;

        Ok(Response::new(invoice_row_to_response(&row)))
    }

    async fn get_invoice(
        &self,
        request: Request<GetInvoiceRequest>,
    ) -> Result<Response<InvoiceResponse>, Status> {
        let req = request.into_inner();

        let tenant_id = Uuid::parse_str(&req.tenant_id)
            .map_err(|_| Status::invalid_argument("invalid tenant_id"))?;

        let period = if req.billing_period.is_empty() {
            Utc::now().format("%Y-%m").to_string()
        } else {
            req.billing_period
        };

        let row = invoices::get_by_period(&self.ctx.pool, tenant_id, &period)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| {
                Status::not_found(format!(
                    "no invoice for tenant {} period {}",
                    tenant_id, period
                ))
            })?;

        Ok(Response::new(invoice_row_to_response(&row)))
    }
}

fn invoice_row_to_response(row: &invoices::InvoiceRow) -> InvoiceResponse {
    InvoiceResponse {
        invoice_id: row.id.to_string(),
        tenant_id: row.tenant_id.to_string(),
        billing_period: row.billing_period.clone(),
        razorpay_id: row.razorpay_id.clone().unwrap_or_default(),
        status: row.status.clone(),
        total_paise: row.total_paise,
        line_items: row.line_items_json.to_string(),
        issued_at: row
            .issued_at
            .map(|t| t.to_rfc3339())
            .unwrap_or_default(),
        paid_at: row.paid_at.map(|t| t.to_rfc3339()).unwrap_or_default(),
        due_at: row.due_at.map(|t| t.to_rfc3339()).unwrap_or_default(),
        created_at: row.created_at.to_rfc3339(),
        updated_at: row.updated_at.to_rfc3339(),
    }
}
