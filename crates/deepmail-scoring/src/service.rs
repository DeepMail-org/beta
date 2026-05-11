use crate::db;
use crate::error::ScoringError;
use crate::scorer::ScoringCtx;
use deepmail_common::proto::scoring::threat_scorer_server::ThreatScorer;
use deepmail_common::proto::scoring::{
    ComputeScoreRequest, ComputeScoreResponse, FeedbackRequest, FeedbackResponse, GetScoreRequest,
};
use std::collections::HashMap;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

pub struct ScoringService {
    ctx: Arc<ScoringCtx>,
}

impl ScoringService {
    pub fn new(ctx: Arc<ScoringCtx>) -> Self {
        Self { ctx }
    }
}

#[tonic::async_trait]
impl ThreatScorer for ScoringService {
    async fn compute_score(
        &self,
        request: Request<ComputeScoreRequest>,
    ) -> Result<Response<ComputeScoreResponse>, Status> {
        let req = request.into_inner();

        let email_id = Uuid::parse_str(&req.email_id)
            .map_err(|_| Status::invalid_argument("invalid email_id"))?;
        let tenant_id = Uuid::parse_str(&req.tenant_id)
            .map_err(|_| Status::invalid_argument("invalid tenant_id"))?;

        let result =
            crate::scorer::compute_and_store(&self.ctx, email_id, tenant_id, req.force_recompute)
                .await
                .map_err(|e| -> Status { e.into() })?;

        let signal_scores: HashMap<String, f32> = result
            .signals
            .iter()
            .map(|(k, v)| (k.as_str().to_string(), *v))
            .collect();

        Ok(Response::new(ComputeScoreResponse {
            email_id: email_id.to_string(),
            final_score: result.final_score,
            final_verdict: result.final_verdict.as_str().to_string(),
            signals_available: result.signals_available,
            signal_scores,
            is_final: result.signals_available as usize >= self.ctx.final_min_signals,
        }))
    }

    async fn get_score(
        &self,
        request: Request<GetScoreRequest>,
    ) -> Result<Response<ComputeScoreResponse>, Status> {
        let req = request.into_inner();

        let email_id = Uuid::parse_str(&req.email_id)
            .map_err(|_| Status::invalid_argument("invalid email_id"))?;

        let row = db::scores::get_by_email_id(&self.ctx.pool, email_id)
            .await
            .map_err(|e| -> Status { e.into() })?
            .ok_or_else(|| {
                Status::not_found(format!("no score found for email {}", email_id))
            })?;

        let mut signal_scores = HashMap::new();
        if let Some(v) = row.header_score {
            signal_scores.insert("header".to_string(), v);
        }
        if let Some(v) = row.dkim_score {
            signal_scores.insert("dkim".to_string(), v);
        }
        if let Some(v) = row.geo_score {
            signal_scores.insert("geo".to_string(), v);
        }
        if let Some(v) = row.ip_score {
            signal_scores.insert("ip".to_string(), v);
        }
        if let Some(v) = row.intel_score {
            signal_scores.insert("intel".to_string(), v);
        }
        if let Some(v) = row.ioc_score {
            signal_scores.insert("ioc".to_string(), v);
        }
        if let Some(v) = row.homograph_score {
            signal_scores.insert("homograph".to_string(), v);
        }
        if let Some(v) = row.body_score {
            signal_scores.insert("body".to_string(), v);
        }
        if let Some(v) = row.sandbox_url_score {
            signal_scores.insert("sandbox_url".to_string(), v);
        }
        if let Some(v) = row.sandbox_file_score {
            signal_scores.insert("sandbox_file".to_string(), v);
        }
        if let Some(v) = row.sandbox_dynamic_score {
            signal_scores.insert("sandbox_dynamic".to_string(), v);
        }

        Ok(Response::new(ComputeScoreResponse {
            email_id: row.email_id.to_string(),
            final_score: row.final_score,
            final_verdict: row.final_verdict,
            signals_available: row.signals_available,
            signal_scores,
            is_final: row.is_final,
        }))
    }

    async fn submit_feedback(
        &self,
        request: Request<FeedbackRequest>,
    ) -> Result<Response<FeedbackResponse>, Status> {
        let req = request.into_inner();

        let email_id = Uuid::parse_str(&req.email_id)
            .map_err(|_| Status::invalid_argument("invalid email_id"))?;
        let tenant_id = Uuid::parse_str(&req.tenant_id)
            .map_err(|_| Status::invalid_argument("invalid tenant_id"))?;
        let analyst_id = Uuid::parse_str(&req.analyst_id)
            .map_err(|_| Status::invalid_argument("invalid analyst_id"))?;

        let valid_verdicts = ["CLEAN", "LOW_RISK", "SUSPICIOUS", "PHISHING", "MALICIOUS"];
        if !valid_verdicts.contains(&req.correct_verdict.as_str()) {
            return Err(Status::invalid_argument(format!(
                "invalid verdict: {}",
                req.correct_verdict
            )));
        }

        let existing = db::scores::get_by_email_id(&self.ctx.pool, email_id)
            .await
            .map_err(|e| -> Status { e.into() })?;

        let predicted_verdict = existing
            .map(|r| r.final_verdict)
            .unwrap_or_else(|| "CLEAN".to_string());

        let notes = if req.feedback_notes.is_empty() {
            None
        } else {
            Some(req.feedback_notes.as_str())
        };

        let feedback_id = db::feedback::insert_feedback(
            &self.ctx.pool,
            email_id,
            tenant_id,
            analyst_id,
            &predicted_verdict,
            &req.correct_verdict,
            notes,
            &serde_json::json!({}),
        )
        .await
        .map_err(|e| -> Status { ScoringError::from(e).into() })?;

        let verdict_levels = |v: &str| -> i32 {
            match v {
                "CLEAN" => 0,
                "LOW_RISK" => 1,
                "SUSPICIOUS" => 2,
                "PHISHING" => 3,
                "MALICIOUS" => 4,
                _ => -1,
            }
        };
        let diff =
            (verdict_levels(&predicted_verdict) - verdict_levels(&req.correct_verdict)).abs();
        if diff >= 2 {
            tracing::info!(
                %email_id,
                predicted = predicted_verdict,
                correct = req.correct_verdict,
                "analyst correction: significant verdict change"
            );
        }

        Ok(Response::new(FeedbackResponse {
            feedback_id: feedback_id.to_string(),
            accepted: true,
        }))
    }
}
