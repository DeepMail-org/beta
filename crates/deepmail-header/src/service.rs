//! gRPC service implementation for HeaderAnalyzer.
//!
//! Exposes:
//!   - AnalyzeHeaders: run full analysis on a set of raw headers.
//!     In production, analysis is triggered via NATS; this RPC is for
//!     ad-hoc requests and integration tests.

use std::sync::Arc;

use sqlx::PgPool;
use tonic::{Request, Response, Status};

use deepmail_common::proto::header::{
    header_analyzer_server::HeaderAnalyzer,
    DkimResult as ProtoDkimResult,
    DmarcResult as ProtoDmarcResult,
    HeaderAnalysisRequest,
    HeaderAnalysisResult,
    HeaderFinding as ProtoHeaderFinding,
    SpfResult as ProtoSpfResult,
    TimeAnomalyResult as ProtoTimeAnomalyResult,
};

use crate::dns::Resolver;
use crate::pipeline;

/// gRPC service handler.
pub struct HeaderService {
    pub header_pool: PgPool,
    pub ingest_pool: PgPool,
    pub parser_pool: PgPool,
    pub resolver: Arc<Resolver>,
}

#[tonic::async_trait]
impl HeaderAnalyzer for HeaderService {
    async fn analyze_headers(
        &self,
        request: Request<HeaderAnalysisRequest>,
    ) -> Result<Response<HeaderAnalysisResult>, Status> {
        let req = request.into_inner();

        let email_id: uuid::Uuid = req
            .email_id
            .parse()
            .map_err(|_| Status::invalid_argument("invalid email_id UUID"))?;

        let tenant_id: uuid::Uuid = req
            .tenant_id
            .parse()
            .map_err(|_| Status::invalid_argument("invalid tenant_id UUID"))?;

        let state = pipeline::PipelineState {
            header_pool: self.header_pool.clone(),
            ingest_pool: self.ingest_pool.clone(),
            parser_pool: self.parser_pool.clone(),
            resolver: self.resolver.clone(),
        };

        let output = pipeline::analyze(&state, email_id, tenant_id)
            .await
            .map_err(|e| -> Status { e.into() })?;

        let proto_result = build_proto_result(&self.header_pool, email_id, &output).await?;

        Ok(Response::new(proto_result))
    }
}

/// Fetch the persisted analysis result from the header DB and map to proto.
async fn build_proto_result(
    pool: &PgPool,
    email_id: uuid::Uuid,
    output: &pipeline::AnalysisOutput,
) -> Result<HeaderAnalysisResult, Status> {
    // Fetch time anomaly if present
    let time_anomaly = fetch_time_anomaly(pool, email_id).await?;

    // Fetch findings
    let findings = fetch_findings(pool, email_id).await?;

    Ok(HeaderAnalysisResult {
        email_id: email_id.to_string(),
        overall_risk_score: output.risk_score,
        spf_result: spf_str_to_proto(&output.spf_result).into(),
        spf_domain: output.spf_domain.clone(),
        dkim_result: dkim_str_to_proto(&output.dkim_result).into(),
        dkim_domain: output.dkim_domain.clone(),
        dmarc_result: dmarc_str_to_proto(&output.dmarc_result).into(),
        dmarc_policy: output.dmarc_policy.clone(),
        return_path_mismatch: output.checks.return_path_mismatch,
        reply_to_mismatch: output.checks.reply_to_mismatch,
        message_id_fake_pattern: output.checks.message_id_fake_pattern,
        x_mailer: output.checks.x_mailer.clone().unwrap_or_default(),
        x_mailer_known_kit: output.checks.x_mailer_known_kit.clone().unwrap_or_default(),
        sender_spoofing_likely: output.checks.sender_spoofing_likely,
        findings,
        time_anomaly,
    })
}

/// Fetch time anomaly from DB.
async fn fetch_time_anomaly(
    pool: &PgPool,
    email_id: uuid::Uuid,
) -> Result<Option<ProtoTimeAnomalyResult>, Status> {
    let row = sqlx::query!(
        r#"SELECT future_dated, date_predates_recv,
                  suspicious_delay, max_hop_delay_sec,
                  total_transit_sec, hop_count
           FROM time_anomalies
           WHERE email_id = $1"#,
        email_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "failed to fetch time_anomalies");
        Status::internal("database error")
    })?;

    Ok(row.map(|r| ProtoTimeAnomalyResult {
        future_dated: r.future_dated,
        date_predates_received: r.date_predates_recv,
        suspicious_delay: r.suspicious_delay,
        max_hop_delay_sec: r.max_hop_delay_sec.unwrap_or(0),
        total_transit_sec: r.total_transit_sec.unwrap_or(0),
        hop_count: r.hop_count.unwrap_or(0),
    }))
}

/// Fetch header findings from DB.
async fn fetch_findings(
    pool: &PgPool,
    email_id: uuid::Uuid,
) -> Result<Vec<ProtoHeaderFinding>, Status> {
    let rows = sqlx::query!(
        r#"SELECT check_name, severity, passed, evidence, raw_value, risk_weight
           FROM header_findings
           WHERE email_id = $1
           ORDER BY created_at"#,
        email_id,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "failed to fetch header_findings");
        Status::internal("database error")
    })?;

    Ok(rows
        .into_iter()
        .map(|r| ProtoHeaderFinding {
            check_name: r.check_name,
            severity: r.severity,
            passed: r.passed,
            evidence: r.evidence,
            raw_value: r.raw_value.unwrap_or_default(),
            risk_weight: r.risk_weight,
        })
        .collect())
}

// ── Proto enum mapping ──────────────────────────────────────────

fn spf_str_to_proto(s: &str) -> i32 {
    match s {
        "pass"      => ProtoSpfResult::SpfPass as i32,
        "fail"      => ProtoSpfResult::SpfFail as i32,
        "softfail"  => ProtoSpfResult::SpfSoftfail as i32,
        "neutral"   => ProtoSpfResult::SpfNeutral as i32,
        "permerror" => ProtoSpfResult::SpfPermerror as i32,
        "temperror" => ProtoSpfResult::SpfTemperror as i32,
        _           => ProtoSpfResult::SpfNone as i32,
    }
}

fn dkim_str_to_proto(s: &str) -> i32 {
    match s {
        "pass"      => ProtoDkimResult::DkimPass as i32,
        "fail"      => ProtoDkimResult::DkimFail as i32,
        "neutral"   => ProtoDkimResult::DkimNeutral as i32,
        "policy"    => ProtoDkimResult::DkimPolicy as i32,
        "temperror" => ProtoDkimResult::DkimTemperror as i32,
        "permerror" => ProtoDkimResult::DkimPermerror as i32,
        _           => ProtoDkimResult::DkimNone as i32,
    }
}

fn dmarc_str_to_proto(s: &str) -> i32 {
    match s {
        "pass"      => ProtoDmarcResult::DmarcPass as i32,
        "fail"      => ProtoDmarcResult::DmarcFail as i32,
        "temperror" => ProtoDmarcResult::DmarcTemperror as i32,
        "permerror" => ProtoDmarcResult::DmarcPermerror as i32,
        _           => ProtoDmarcResult::DmarcNone as i32,
    }
}
