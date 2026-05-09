/// gRPC service implementation for FileSandbox.

use std::sync::Arc;

use tonic::{Request, Response, Status};
use uuid::Uuid;

use deepmail_common::proto::sandbox_file::{
    file_sandbox_server::FileSandbox,
    FileAnalyzeRequest, FileAnalyzeResponse,
    FileReportRequest, FileReportResponse,
};

use crate::db;
use crate::pipeline::{IncomingJob, JobCtx};

pub struct FileSandboxService {
    pub ctx: Arc<JobCtx>,
}

#[tonic::async_trait]
impl FileSandbox for FileSandboxService {
    async fn analyze_file(
        &self,
        request: Request<FileAnalyzeRequest>,
    ) -> Result<Response<FileAnalyzeResponse>, Status> {
        let req = request.into_inner();

        let attachment_id = Uuid::parse_str(&req.attachment_id)
            .map_err(|_| Status::invalid_argument("invalid attachment_id"))?;
        let email_id = Uuid::parse_str(&req.email_id)
            .map_err(|_| Status::invalid_argument("invalid email_id"))?;
        let tenant_id = Uuid::parse_str(&req.tenant_id)
            .map_err(|_| Status::invalid_argument("invalid tenant_id"))?;

        let job = IncomingJob {
            attachment_id,
            email_id,
            tenant_id,
            s3_key: req.s3_key,
            filename: req.filename,
        };

        let report = crate::pipeline::run_file_job(Arc::clone(&self.ctx), job)
            .await
            .map_err(|e| Status::internal(format!("analysis failed: {}", e)))?;

        Ok(Response::new(FileAnalyzeResponse {
            report_id: report.id.to_string(),
            threat_verdict: report.threat_verdict,
            threat_score: report.threat_score,
            has_macros: report.has_macros,
            is_pe: report.is_pe,
            yara_matches: report.yara_matches,
        }))
    }

    async fn get_report(
        &self,
        request: Request<FileReportRequest>,
    ) -> Result<Response<FileReportResponse>, Status> {
        let req = request.into_inner();

        let attachment_id = Uuid::parse_str(&req.attachment_id)
            .map_err(|_| Status::invalid_argument("invalid attachment_id"))?;

        let report = db::reports::get_by_attachment_id(&self.ctx.pool, attachment_id)
            .await
            .map_err(|e| Status::internal(format!("db: {}", e)))?
            .ok_or_else(|| Status::not_found("report not found"))?;

        Ok(Response::new(FileReportResponse {
            report_id: report.id.to_string(),
            attachment_id: report.attachment_id.to_string(),
            threat_verdict: report.threat_verdict,
            threat_score: report.threat_score,
            yara_matches: report.yara_matches,
            analysis_notes: report.analysis_notes,
        }))
    }
}
