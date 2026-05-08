/// gRPC service implementation for BodyAnalyzer.

use std::sync::Arc;

use tonic::{Request, Response, Status};
use uuid::Uuid;

use deepmail_common::proto::body::{
    body_analyzer_server::BodyAnalyzer,
    BodyAnalyzeRequest, BodyAnalyzeResponse, BodyGetRequest,
};

use crate::db;
use crate::error::BodyError;
use crate::pipeline::PipelineCtx;

/// gRPC service for body analysis.
pub struct BodyAnalyzerService {
    pub ctx: Arc<PipelineCtx>,
}

#[tonic::async_trait]
impl BodyAnalyzer for BodyAnalyzerService {
    /// Analyze an email body (idempotent).
    async fn analyze_email(
        &self,
        request: Request<BodyAnalyzeRequest>,
    ) -> Result<Response<BodyAnalyzeResponse>, Status> {
        let req = request.into_inner();

        let email_id: Uuid = req.email_id.parse()
            .map_err(|_| Status::invalid_argument("invalid email_id"))?;
        let tenant_id: Uuid = req.tenant_id.parse()
            .map_err(|_| Status::invalid_argument("invalid tenant_id"))?;

        let result = crate::pipeline::run_pipeline(
            Arc::clone(&self.ctx),
            email_id,
            tenant_id,
        )
        .await
        .map_err(|e: BodyError| -> Status { e.into() })?;

        Ok(Response::new(BodyAnalyzeResponse {
            email_id: result.email_id.to_string(),
            analysis_id: result.analysis_id.to_string(),
            url_count: result.url_count,
            qr_code_count: result.qr_code_count,
            final_phishing_score: result.final_phishing_score,
            verdict: result.verdict,
            has_obfuscation: result.has_obfuscation,
            has_tracking_pixels: result.has_tracking_pixels,
        }))
    }

    /// Get existing body analysis by email_id.
    async fn get_analysis(
        &self,
        request: Request<BodyGetRequest>,
    ) -> Result<Response<BodyAnalyzeResponse>, Status> {
        let req = request.into_inner();

        let email_id: Uuid = req.email_id.parse()
            .map_err(|_| Status::invalid_argument("invalid email_id"))?;

        let row = db::analyses::get_by_email_id(&self.ctx.pool, email_id)
            .await
            .map_err(|e: BodyError| -> Status { e.into() })?
            .ok_or_else(|| Status::not_found(format!("no body analysis for {}", email_id)))?;

        Ok(Response::new(BodyAnalyzeResponse {
            email_id: row.email_id.to_string(),
            analysis_id: row.id.to_string(),
            url_count: row.url_count,
            qr_code_count: row.qr_code_count,
            final_phishing_score: row.final_phishing_score,
            verdict: row.verdict,
            has_obfuscation: row.has_obfuscation,
            has_tracking_pixels: row.has_tracking_pixels,
        }))
    }
}
