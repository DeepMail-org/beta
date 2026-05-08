/// gRPC IntelEnricherService implementation.

use std::collections::HashMap;
use std::sync::Arc;

use tonic::{Request, Response, Status};

use deepmail_common::proto::intel::intel_enricher_server::IntelEnricher;
use deepmail_common::proto::intel::{
    EnrichBatchRequest, EnrichBatchResponse, EnrichIocRequest, EnrichIocResponse,
    GetCachedRequest, ProviderStatusRequest, ProviderStatusResponse,
};

use crate::cache;
use crate::circuit::CircuitRegistry;
use crate::enricher::{enrich_ioc, EnrichCtx};

pub struct IntelEnricherService {
    ctx: Arc<EnrichCtx>,
    circuits: Arc<CircuitRegistry>,
}

impl IntelEnricherService {
    pub fn new(ctx: Arc<EnrichCtx>, circuits: Arc<CircuitRegistry>) -> Self {
        Self { ctx, circuits }
    }
}

#[tonic::async_trait]
impl IntelEnricher for IntelEnricherService {
    async fn enrich_ioc(
        &self,
        request: Request<EnrichIocRequest>,
    ) -> Result<Response<EnrichIocResponse>, Status> {
        let req = request.into_inner();

        if req.ioc_value.is_empty() || req.ioc_type.is_empty() {
            return Err(Status::invalid_argument("ioc_value and ioc_type required"));
        }

        let valid_types = ["ip", "domain", "url", "hash"];
        if !valid_types.contains(&req.ioc_type.as_str()) {
            return Err(Status::invalid_argument(format!(
                "invalid ioc_type: {}, must be one of: ip, domain, url, hash",
                req.ioc_type
            )));
        }

        let result = enrich_ioc(
            &req.ioc_value,
            &req.ioc_type,
            &req.providers,
            req.force_refresh,
            &self.ctx,
        )
        .await
        .map_err(|e| -> Status { e.into() })?;

        let provider_results_json: HashMap<String, String> = result
            .provider_results
            .iter()
            .map(|(k, v)| (k.clone(), v.to_string()))
            .collect();

        Ok(Response::new(EnrichIocResponse {
            ioc_value: result.ioc_value,
            ioc_type: result.ioc_type,
            provider_results_json,
            max_score: result.max_score,
            is_malicious: result.is_malicious,
            cached_at: result.fetched_at.timestamp(),
        }))
    }

    async fn enrich_batch(
        &self,
        request: Request<EnrichBatchRequest>,
    ) -> Result<Response<EnrichBatchResponse>, Status> {
        let req = request.into_inner();

        if req.iocs.len() > 100 {
            return Err(Status::invalid_argument(
                "batch size must not exceed 100 IOCs",
            ));
        }

        let mut handles = Vec::new();
        for ioc_req in req.iocs {
            let ctx = self.ctx.clone();
            let handle = tokio::spawn(async move {
                enrich_ioc(
                    &ioc_req.ioc_value,
                    &ioc_req.ioc_type,
                    &ioc_req.providers,
                    ioc_req.force_refresh,
                    &ctx,
                )
                .await
            });
            handles.push(handle);
        }

        let mut responses = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(Ok(result)) => {
                    let provider_results_json: HashMap<String, String> = result
                        .provider_results
                        .iter()
                        .map(|(k, v)| (k.clone(), v.to_string()))
                        .collect();

                    responses.push(EnrichIocResponse {
                        ioc_value: result.ioc_value,
                        ioc_type: result.ioc_type,
                        provider_results_json,
                        max_score: result.max_score,
                        is_malicious: result.is_malicious,
                        cached_at: result.fetched_at.timestamp(),
                    });
                }
                Ok(Err(e)) => {
                    tracing::warn!(error = %e, "batch enrichment item failed");
                    // Partial failure — skip this IOC in results
                }
                Err(e) => {
                    tracing::warn!(error = %e, "batch enrichment task panicked");
                }
            }
        }

        Ok(Response::new(EnrichBatchResponse { results: responses }))
    }

    async fn get_cached_result(
        &self,
        request: Request<GetCachedRequest>,
    ) -> Result<Response<EnrichIocResponse>, Status> {
        let req = request.into_inner();

        if req.ioc_value.is_empty() || req.ioc_type.is_empty() || req.provider.is_empty() {
            return Err(Status::invalid_argument(
                "ioc_value, ioc_type, and provider required",
            ));
        }

        let cached = cache::get_cached_db(&self.ctx.pool, &req.ioc_type, &req.ioc_value, &req.provider)
            .await
            .map_err(|e| -> Status { e.into() })?;

        match cached {
            Some(row) => {
                let mut provider_results: HashMap<String, String> = HashMap::new();
                provider_results.insert(req.provider, row.result_json.to_string());

                Ok(Response::new(EnrichIocResponse {
                    ioc_value: row.ioc_value,
                    ioc_type: row.ioc_type,
                    provider_results_json: provider_results,
                    max_score: row.vt_score.unwrap_or(0.0),
                    is_malicious: row.vt_score.unwrap_or(0.0) > 0.5
                        || row.abuse_score.unwrap_or(0) > 80
                        || row.pulse_count.unwrap_or(0) > 5,
                    cached_at: row.fetched_at.timestamp(),
                }))
            }
            None => Err(Status::not_found("no cached result found")),
        }
    }

    async fn get_provider_status(
        &self,
        request: Request<ProviderStatusRequest>,
    ) -> Result<Response<ProviderStatusResponse>, Status> {
        let req = request.into_inner();

        let statuses = if req.provider.is_empty() {
            self.circuits.all_statuses().await
        } else {
            let mut map = HashMap::new();
            match self.circuits.get(&req.provider) {
                Some(cb) => {
                    map.insert(req.provider, cb.state_str().await.to_string());
                }
                None => {
                    return Err(Status::not_found(format!(
                        "unknown provider: {}",
                        req.provider
                    )));
                }
            }
            map
        };

        Ok(Response::new(ProviderStatusResponse { statuses }))
    }
}
