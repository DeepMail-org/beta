/// gRPC HomographDetectorService implementation.

use std::sync::Arc;

use tonic::{Request, Response, Status};
use uuid::Uuid;

use deepmail_common::proto::homograph::homograph_detector_server::HomographDetector;
use deepmail_common::proto::homograph::{
    DomainScore as DomainScoreProto, HomographAnalyzeRequest, HomographAnalyzeResponse,
    HomographGetRequest,
};

use crate::db;
use crate::pipeline::{self, PipelineCtx};
use crate::similarity::RiskLevel;

pub struct HomographDetectorService {
    ctx: Arc<PipelineCtx>,
}

impl HomographDetectorService {
    pub fn new(ctx: Arc<PipelineCtx>) -> Self {
        Self { ctx }
    }
}

/// Convert risk string to proto enum i32.
fn risk_str_to_proto(risk: &str) -> i32 {
    match risk {
        "NONE" => 0,
        "LOW" => 1,
        "MEDIUM" => 2,
        "HIGH" => 3,
        "CRITICAL" => 4,
        _ => 0,
    }
}

#[tonic::async_trait]
impl HomographDetector for HomographDetectorService {
    async fn analyze_email(
        &self,
        request: Request<HomographAnalyzeRequest>,
    ) -> Result<Response<HomographAnalyzeResponse>, Status> {
        let req = request.into_inner();

        let email_id = Uuid::parse_str(&req.email_id)
            .map_err(|_| Status::invalid_argument("invalid email_id UUID"))?;
        let tenant_id = Uuid::parse_str(&req.tenant_id)
            .map_err(|_| Status::invalid_argument("invalid tenant_id UUID"))?;

        let result = pipeline::run_pipeline(&self.ctx, email_id, tenant_id)
            .await
            .map_err(|e| -> Status { e.into() })?;

        // Fetch domain scores for response
        let scores = db::scores::list_by_analysis(&self.ctx.pool, result.analysis_id)
            .await
            .unwrap_or_default();

        let proto_scores: Vec<DomainScoreProto> = scores
            .into_iter()
            .map(|s| DomainScoreProto {
                original_domain: s.domain,
                decoded_unicode: s.decoded_domain,
                skeleton: s.skeleton,
                best_brand_match: s.best_brand_match,
                raw_similarity: s.raw_similarity,
                final_score: s.final_score,
                edit_distance: s.edit_distance as u32,
                mixed_script: s.mixed_script,
                punycode_abuse: s.punycode_abuse,
                risk_level: risk_str_to_proto(&s.risk_level),
            })
            .collect();

        Ok(Response::new(HomographAnalyzeResponse {
            email_id: req.email_id,
            domains_checked: result.domains_checked,
            high_risk_count: result.high_risk_count,
            overall_risk: result.overall_risk,
            analysis_id: result.analysis_id.to_string(),
            scores: proto_scores,
        }))
    }

    async fn get_analysis(
        &self,
        request: Request<HomographGetRequest>,
    ) -> Result<Response<HomographAnalyzeResponse>, Status> {
        let req = request.into_inner();

        let email_id = Uuid::parse_str(&req.email_id)
            .map_err(|_| Status::invalid_argument("invalid email_id UUID"))?;

        let analysis = db::analyses::get_by_email(&self.ctx.pool, email_id)
            .await
            .map_err(|e| -> Status { e.into() })?
            .ok_or_else(|| Status::not_found("homograph analysis not found"))?;

        let scores = db::scores::list_by_analysis(&self.ctx.pool, analysis.id)
            .await
            .unwrap_or_default();

        let proto_scores: Vec<DomainScoreProto> = scores
            .into_iter()
            .map(|s| DomainScoreProto {
                original_domain: s.domain,
                decoded_unicode: s.decoded_domain,
                skeleton: s.skeleton,
                best_brand_match: s.best_brand_match,
                raw_similarity: s.raw_similarity,
                final_score: s.final_score,
                edit_distance: s.edit_distance as u32,
                mixed_script: s.mixed_script,
                punycode_abuse: s.punycode_abuse,
                risk_level: risk_str_to_proto(&s.risk_level),
            })
            .collect();

        Ok(Response::new(HomographAnalyzeResponse {
            email_id: req.email_id,
            domains_checked: analysis.domains_checked,
            high_risk_count: analysis.high_risk_count,
            overall_risk: analysis.overall_risk,
            analysis_id: analysis.id.to_string(),
            scores: proto_scores,
        }))
    }
}
