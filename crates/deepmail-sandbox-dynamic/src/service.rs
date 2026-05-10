/// gRPC service implementation.

use std::sync::Arc;

use tonic::{Request, Response, Status};
use uuid::Uuid;

use deepmail_common::proto::sandbox_dynamic::dynamic_sandbox_server::DynamicSandbox;
use deepmail_common::proto::sandbox_dynamic::{
    DynamicAnalyzeRequest, DynamicAnalyzeResponse, DynamicJobStatusRequest,
    DynamicJobStatusResponse, DynamicReportRequest, DynamicReportResponse,
};

use crate::db;
use crate::pipeline::JobCtx;

pub struct DynamicSandboxService {
    pub ctx: Arc<JobCtx>,
}

#[tonic::async_trait]
impl DynamicSandbox for DynamicSandboxService {
    /// AnalyzeFile: idempotent. Returns existing results or starts new job.
    async fn analyze_file(
        &self,
        request: Request<DynamicAnalyzeRequest>,
    ) -> Result<Response<DynamicAnalyzeResponse>, Status> {
        let req = request.into_inner();

        let attachment_id = Uuid::parse_str(&req.attachment_id)
            .map_err(|e| Status::invalid_argument(format!("bad attachment_id: {}", e)))?;
        let email_id = Uuid::parse_str(&req.email_id)
            .map_err(|e| Status::invalid_argument(format!("bad email_id: {}", e)))?;
        let tenant_id = Uuid::parse_str(&req.tenant_id)
            .map_err(|e| Status::invalid_argument(format!("bad tenant_id: {}", e)))?;

        // Check existing job
        if let Some(existing) = db::jobs::get_by_attachment_id(&self.ctx.pool, attachment_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
        {
            if existing.status == "completed" {
                // Fetch report
                if let Some(report) =
                    db::reports::get_by_attachment_id(&self.ctx.pool, attachment_id)
                        .await
                        .map_err(|e| Status::internal(e.to_string()))?
                {
                    return Ok(Response::new(DynamicAnalyzeResponse {
                        job_id: existing.id.to_string(),
                        status: existing.status,
                        dynamic_verdict: report.dynamic_verdict,
                        dynamic_score: report.dynamic_score,
                        cape_unavailable: report.cape_unavailable,
                    }));
                }
            }
            // Pending/running/failed — return current status
            return Ok(Response::new(DynamicAnalyzeResponse {
                job_id: existing.id.to_string(),
                status: existing.status,
                dynamic_verdict: "UNKNOWN".into(),
                dynamic_score: 0.0,
                cape_unavailable: existing.cape_unavailable,
            }));
        }

        // New job: create + publish to NATS
        let job_id = db::jobs::create_job(
            &self.ctx.pool,
            email_id,
            tenant_id,
            attachment_id,
            &req.s3_key,
            &req.filename,
            if req.sha256_hash.is_empty() {
                None
            } else {
                Some(req.sha256_hash.as_str())
            },
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        // Publish NATS job
        let payload = serde_json::json!({
            "attachment_id": req.attachment_id,
            "email_id": req.email_id,
            "tenant_id": req.tenant_id,
            "s3_key": req.s3_key,
            "filename": req.filename,
            "sha256_hash": req.sha256_hash,
        });

        if let Err(e) = self
            .ctx
            .nats
            .publish(
                "deepmail.jobs.sandbox.dynamic",
                payload.to_string().into(),
            )
            .await
        {
            tracing::warn!("failed to publish dynamic job: {}", e);
        }

        Ok(Response::new(DynamicAnalyzeResponse {
            job_id: job_id.to_string(),
            status: "pending".into(),
            dynamic_verdict: "UNKNOWN".into(),
            dynamic_score: 0.0,
            cape_unavailable: false,
        }))
    }

    /// GetReport: returns completed report for an attachment.
    async fn get_report(
        &self,
        request: Request<DynamicReportRequest>,
    ) -> Result<Response<DynamicReportResponse>, Status> {
        let req = request.into_inner();
        let attachment_id = Uuid::parse_str(&req.attachment_id)
            .map_err(|e| Status::invalid_argument(format!("bad attachment_id: {}", e)))?;

        let report = db::reports::get_by_attachment_id(&self.ctx.pool, attachment_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("no report for attachment"))?;

        Ok(Response::new(DynamicReportResponse {
            report_id: report.id.to_string(),
            attachment_id: req.attachment_id,
            dynamic_verdict: report.dynamic_verdict,
            dynamic_score: report.dynamic_score,
            network_hosts: report.network_hosts,
            dns_requests: report.dns_requests,
            persistence_indicators: report.persistence_indicators,
            c2_indicators: report.c2_indicators,
            analysis_notes: report.analysis_notes,
            cape_unavailable: report.cape_unavailable,
        }))
    }

    /// GetJobStatus: returns the current status of a dynamic analysis job.
    async fn get_job_status(
        &self,
        request: Request<DynamicJobStatusRequest>,
    ) -> Result<Response<DynamicJobStatusResponse>, Status> {
        let req = request.into_inner();
        let job_id = Uuid::parse_str(&req.job_id)
            .map_err(|e| Status::invalid_argument(format!("bad job_id: {}", e)))?;

        let job = db::jobs::get_job(&self.ctx.pool, job_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("job not found"))?;

        Ok(Response::new(DynamicJobStatusResponse {
            job_id: req.job_id,
            status: job.status,
        }))
    }
}
