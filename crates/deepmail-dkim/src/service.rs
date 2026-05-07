use std::sync::Arc;

use sqlx::PgPool;
use tonic::{Request, Response, Status};

use deepmail_common::proto::dkim::{
    dkim_analyzer_server::DkimAnalyzer,
    DkimAnalysisRequest,
    DkimAnalysisResult,
    DkimKeyStatus,
    DkimSignatureResult,
    KeyRotationRequest,
    KeyRotationResult,
};

use crate::db::{analyses, key_cache, signatures};
use crate::dns::Resolver;
use crate::pipeline;

pub struct DkimService {
    pub dkim_pool: PgPool,
    pub ingest_pool: PgPool,
    pub resolver: Arc<Resolver>,
}

#[tonic::async_trait]
impl DkimAnalyzer for DkimService {
    async fn analyze_dkim(
        &self,
        request: Request<DkimAnalysisRequest>,
    ) -> Result<Response<DkimAnalysisResult>, Status> {
        let req = request.into_inner();

        let email_id: uuid::Uuid = req
            .email_id
            .parse()
            .map_err(|_| Status::invalid_argument("invalid email_id UUID"))?;

        let tenant_id: uuid::Uuid = req
            .tenant_id
            .parse()
            .map_err(|_| Status::invalid_argument("invalid tenant_id UUID"))?;

        if let Ok(Some(existing)) = analyses::get_by_email_id(&self.dkim_pool, email_id).await {
            let sigs = signatures::list_by_analysis(&self.dkim_pool, existing.id)
                .await
                .unwrap_or_default();
            let proto_result = build_result_from_db(&existing, &sigs);
            return Ok(Response::new(proto_result));
        }

        let state = pipeline::PipelineState {
            dkim_pool: self.dkim_pool.clone(),
            ingest_pool: self.ingest_pool.clone(),
            resolver: self.resolver.clone(),
        };

        let output = pipeline::analyze(&state, email_id, tenant_id)
            .await
            .map_err(|e| -> Status { e.into() })?;

        let existing = analyses::get_by_email_id(&self.dkim_pool, email_id)
            .await
            .map_err(|e| -> Status { e.into() })?
            .ok_or_else(|| Status::internal("analysis not persisted"))?;

        let sigs = signatures::list_by_analysis(&self.dkim_pool, existing.id)
            .await
            .map_err(|e| -> Status { e.into() })?;

        let mut proto_result = build_result_from_db(&existing, &sigs);
        proto_result.analysis_ms = output.duration_ms as u64;

        Ok(Response::new(proto_result))
    }

    async fn check_key_rotation(
        &self,
        request: Request<KeyRotationRequest>,
    ) -> Result<Response<KeyRotationResult>, Status> {
        let req = request.into_inner();

        if req.selector.is_empty() || req.domain.is_empty() {
            return Err(Status::invalid_argument("selector and domain required"));
        }

        let dns_result = self.resolver.lookup_dkim_key(&req.selector, &req.domain).await;

        let was_rotated = if let Some(ref txt) = dns_result.raw_txt {
            if let Ok(Some(cached)) =
                key_cache::get_cached_key(&self.dkim_pool, &req.selector, &req.domain).await
            {
                cached.public_key_pem.as_deref() != Some(txt.as_str())
            } else {
                false
            }
        } else {
            true
        };

        let current_key_tag = dns_result.raw_txt.unwrap_or_default();

        let ttl = dns_result.ttl_seconds.unwrap_or(300);
        let _ = key_cache::upsert_key(
            &self.dkim_pool,
            &req.selector,
            &req.domain,
            if current_key_tag.is_empty() {
                None
            } else {
                Some(current_key_tag.as_str())
            },
            None,
            ttl,
        )
        .await;

        Ok(Response::new(KeyRotationResult {
            was_rotated,
            current_key_tag,
        }))
    }
}

fn build_result_from_db(
    analysis: &analyses::AnalysisRow,
    sigs: &[signatures::SignatureRow],
) -> DkimAnalysisResult {
    let confidence = analysis.replay_confidence;
    let replay_detected = confidence >= 0.4;

    let replay_reasons = build_replay_reasons(&analysis.signals_json, confidence);

    let proto_sigs: Vec<DkimSignatureResult> = sigs
        .iter()
        .map(|s| {
            let key_status = if !s.key_found_in_dns {
                DkimKeyStatus::KeyNotFound as i32
            } else if s.signature_valid {
                DkimKeyStatus::KeyValid as i32
            } else {
                DkimKeyStatus::KeyRotated as i32
            };

            let timestamp_signed = s.signature_timestamp.map(|ts| prost_types::Timestamp {
                seconds: ts.timestamp(),
                nanos: 0,
            });

            DkimSignatureResult {
                selector: s.selector.clone(),
                domain: s.domain.clone(),
                algorithm: s.algorithm.clone(),
                body_hash_claimed: s.body_hash_claimed.clone(),
                body_hash_computed: s.body_hash_computed.clone(),
                body_hash_match: s.body_hash_match,
                header_delta_seconds: 0,
                receive_delta_seconds: 0,
                key_status,
                key_rotated_since_signing: !s.key_found_in_dns || !s.signature_valid,
                signature_valid: s.signature_valid,
                timestamp_signed,
            }
        })
        .collect();

    DkimAnalysisResult {
        email_id: analysis.email_id.to_string(),
        replay_detected,
        replay_confidence: confidence,
        replay_reasons,
        signatures: proto_sigs,
        analysis_ms: 0,
    }
}

fn build_replay_reasons(signals_json: &serde_json::Value, confidence: f32) -> Vec<String> {
    let mut reasons = Vec::new();

    if confidence < 0.2 {
        return reasons;
    }

    if let Some(v) = signals_json.get("body_hash_mismatch").and_then(|v| v.as_f64()) {
        if v > 0.5 {
            reasons.push("body hash mismatch detected".to_string());
        }
    }
    if let Some(v) = signals_json.get("timestamp_drift").and_then(|v| v.as_f64()) {
        if v > 0.5 {
            reasons.push("significant timestamp drift between Date and signature".to_string());
        }
    }
    if let Some(v) = signals_json.get("received_before_signed").and_then(|v| v.as_f64()) {
        if v > 0.5 {
            reasons.push("email received before DKIM signing timestamp".to_string());
        }
    }
    if let Some(v) = signals_json.get("key_rotated_since").and_then(|v| v.as_f64()) {
        if v > 0.5 {
            reasons.push("signing key no longer present in DNS".to_string());
        }
    }
    if let Some(v) = signals_json.get("excessive_replay_gap").and_then(|v| v.as_f64()) {
        if v > 0.5 {
            reasons.push("excessive gap between signing and delivery".to_string());
        }
    }
    if let Some(v) = signals_json.get("header_order_tampered").and_then(|v| v.as_f64()) {
        if v > 0.5 {
            reasons.push("signed header order inconsistent with actual headers".to_string());
        }
    }

    reasons
}
