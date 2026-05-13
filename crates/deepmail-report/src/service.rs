use crate::pipeline::PipelineCtx;
use deepmail_common::proto::report::report_service_server::ReportService;
use deepmail_common::proto::report::{
    DigestScheduleRequest, DigestScheduleResponse, GetReportRequest, ReportRequest, ReportResponse,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

pub struct ReportGrpcService {
    ctx: Arc<PipelineCtx>,
}

impl ReportGrpcService {
    pub fn new(ctx: Arc<PipelineCtx>) -> Self {
        Self { ctx }
    }
}

#[tonic::async_trait]
impl ReportService for ReportGrpcService {
    async fn generate_report(
        &self,
        request: Request<ReportRequest>,
    ) -> Result<Response<ReportResponse>, Status> {
        let req = request.into_inner();

        let email_id = Uuid::parse_str(&req.email_id)
            .map_err(|_| Status::invalid_argument("invalid email_id"))?;
        let tenant_id = Uuid::parse_str(&req.tenant_id)
            .map_err(|_| Status::invalid_argument("invalid tenant_id"))?;

        let result =
            crate::pipeline::generate_report(&self.ctx, email_id, tenant_id, req.force_regenerate)
                .await
                .map_err(|e| -> Status { e.into() })?;

        Ok(Response::new(ReportResponse {
            report_id: result.report_id.to_string(),
            email_id: result.email_id.to_string(),
            json_s3_key: result.json_s3_key,
            html_s3_key: result.html_s3_key,
            status: "generated".to_string(),
            final_verdict: result.final_verdict,
            final_score: result.final_score,
        }))
    }

    async fn get_report(
        &self,
        request: Request<GetReportRequest>,
    ) -> Result<Response<ReportResponse>, Status> {
        let req = request.into_inner();

        let email_id = Uuid::parse_str(&req.email_id)
            .map_err(|_| Status::invalid_argument("invalid email_id"))?;
        let tenant_id = Uuid::parse_str(&req.tenant_id)
            .map_err(|_| Status::invalid_argument("invalid tenant_id"))?;

        let format = if req.format.is_empty() { "json" } else { &req.format };
        if format != "json" && format != "html" {
            return Err(Status::invalid_argument("format must be 'json' or 'html'"));
        }

        let existing =
            crate::db::exports::get_by_email_format(&self.ctx.own_pool, email_id, format)
                .await
                .map_err(|e| -> Status { crate::error::ReportError::from(e).into() })?;

        if let Some(row) = existing {
            let scoring: Option<(f32, String)> = sqlx::query_as(
                "SELECT final_score, final_verdict FROM threat_scores WHERE email_id = $1 LIMIT 1",
            )
            .bind(email_id)
            .fetch_optional(self.ctx.pools.scoring.as_ref())
            .await
            .unwrap_or_else(|e| { tracing::warn!(error = %e, "scoring query failed"); None });

            let (score, verdict) = scoring.unwrap_or((0.0, "CLEAN".to_string()));

            let other_format = if format == "json" { "html" } else { "json" };
            let other_key = crate::db::exports::get_by_email_format(
                &self.ctx.own_pool,
                email_id,
                other_format,
            )
            .await
            .ok()
            .flatten()
            .map(|r| r.s3_key)
            .unwrap_or_default();

            let (json_key, html_key) = if format == "json" {
                (row.s3_key, other_key)
            } else {
                (other_key, row.s3_key)
            };

            return Ok(Response::new(ReportResponse {
                report_id: row.id.to_string(),
                email_id: email_id.to_string(),
                json_s3_key: json_key,
                html_s3_key: html_key,
                status: "cached".to_string(),
                final_verdict: verdict,
                final_score: score,
            }));
        }

        let result = crate::pipeline::generate_report(&self.ctx, email_id, tenant_id, false)
            .await
            .map_err(|e| -> Status { e.into() })?;

        Ok(Response::new(ReportResponse {
            report_id: result.report_id.to_string(),
            email_id: result.email_id.to_string(),
            json_s3_key: result.json_s3_key,
            html_s3_key: result.html_s3_key,
            status: "generated".to_string(),
            final_verdict: result.final_verdict,
            final_score: result.final_score,
        }))
    }

    async fn schedule_digest(
        &self,
        request: Request<DigestScheduleRequest>,
    ) -> Result<Response<DigestScheduleResponse>, Status> {
        let req = request.into_inner();

        let tenant_id = Uuid::parse_str(&req.tenant_id)
            .map_err(|_| Status::invalid_argument("invalid tenant_id"))?;

        let row = crate::pipeline::schedule_digest(
            &self.ctx.own_pool,
            tenant_id,
            &req.frequency,
            &req.recipient_emails,
        )
        .await
        .map_err(|e| -> Status { e.into() })?;

        Ok(Response::new(DigestScheduleResponse {
            schedule_id: row.id.to_string(),
            accepted: true,
        }))
    }
}
