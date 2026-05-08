/// gRPC service implementation for UrlSandbox.

use std::sync::Arc;

use tonic::{Request, Response, Status};
use uuid::Uuid;

use deepmail_common::proto::sandbox_url::url_sandbox_server::UrlSandbox;
use deepmail_common::proto::sandbox_url::{
    JobStatusRequest, JobStatusResponse, UrlReportRequest, UrlReportResponse,
    UrlSubmitRequest, UrlSubmitResponse,
};

use crate::db;
use crate::pipeline::JobCtx;

pub struct UrlSandboxService {
    pub ctx: Arc<JobCtx>,
}

#[tonic::async_trait]
impl UrlSandbox for UrlSandboxService {
    /// Submit a URL for sandbox analysis.
    async fn submit_url(
        &self,
        request: Request<UrlSubmitRequest>,
    ) -> Result<Response<UrlSubmitResponse>, Status> {
        let req = request.into_inner();

        let email_id: Uuid = req.email_id.parse()
            .map_err(|_| Status::invalid_argument("invalid email_id UUID"))?;
        let tenant_id: Uuid = req.tenant_id.parse()
            .map_err(|_| Status::invalid_argument("invalid tenant_id UUID"))?;

        if req.url.is_empty() {
            return Err(Status::invalid_argument("url is required"));
        }

        let url_type = if req.url_type.is_empty() { "href" } else { &req.url_type };

        // Create job record
        let job_id = db::jobs::create_job(
            &self.ctx.pool,
            email_id,
            tenant_id,
            &req.url,
            url_type,
        )
        .await
        .map_err(|e| Status::internal(format!("db error: {}", e)))?;

        // Publish to NATS for async processing
        let payload = serde_json::json!({
            "email_id": email_id.to_string(),
            "tenant_id": tenant_id.to_string(),
            "url": req.url,
            "url_type": url_type,
            "job_id": job_id.to_string(),
        });

        self.ctx
            .nats
            .publish(
                "deepmail.jobs.sandbox.url",
                payload.to_string().into(),
            )
            .await
            .map_err(|e| Status::internal(format!("nats publish: {}", e)))?;

        Ok(Response::new(UrlSubmitResponse {
            job_id: job_id.to_string(),
            status: "pending".into(),
        }))
    }

    /// Get the analysis report for a completed sandbox job.
    async fn get_report(
        &self,
        request: Request<UrlReportRequest>,
    ) -> Result<Response<UrlReportResponse>, Status> {
        let req = request.into_inner();
        let job_id: Uuid = req.job_id.parse()
            .map_err(|_| Status::invalid_argument("invalid job_id UUID"))?;

        let report = db::reports::get_by_job_id(&self.ctx.pool, job_id)
            .await
            .map_err(|e| Status::internal(format!("db error: {}", e)))?
            .ok_or_else(|| Status::not_found("report not found"))?;

        Ok(Response::new(UrlReportResponse {
            job_id: report.job_id.to_string(),
            original_url: report.original_url,
            final_url: report.final_url.unwrap_or_default(),
            threat_class: report.threat_class,
            threat_score: report.threat_score,
            has_login_form: report.has_login_form,
            redirect_count: report.redirect_count,
        }))
    }

    /// Get the status of a sandbox job.
    async fn get_job_status(
        &self,
        request: Request<JobStatusRequest>,
    ) -> Result<Response<JobStatusResponse>, Status> {
        let req = request.into_inner();
        let job_id: Uuid = req.job_id.parse()
            .map_err(|_| Status::invalid_argument("invalid job_id UUID"))?;

        let job = db::jobs::get_job(&self.ctx.pool, job_id)
            .await
            .map_err(|e| Status::internal(format!("db error: {}", e)))?
            .ok_or_else(|| Status::not_found("job not found"))?;

        Ok(Response::new(JobStatusResponse {
            job_id: job.id.to_string(),
            status: job.status,
        }))
    }
}
