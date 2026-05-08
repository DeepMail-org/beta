/// gRPC IocExtractorService implementation.

use std::sync::Arc;

use tonic::{Request, Response, Status};
use uuid::Uuid;

use deepmail_common::proto::ioc::ioc_extractor_server::IocExtractor;
use deepmail_common::proto::ioc::{
    CampaignRequest, CampaignResponse, IocEmailRequest, IocEmailResponse,
    IocExtractRequest, IocExtractResponse, IocNodeProto, IocNodeRequest,
    IocNodeResponse,
};

use crate::db;
use crate::pipeline::{self, PipelineCtx};

pub struct IocExtractorService {
    ctx: Arc<PipelineCtx>,
}

impl IocExtractorService {
    pub fn new(ctx: Arc<PipelineCtx>) -> Self {
        Self { ctx }
    }
}

#[tonic::async_trait]
impl IocExtractor for IocExtractorService {
    async fn extract_and_enrich(
        &self,
        request: Request<IocExtractRequest>,
    ) -> Result<Response<IocExtractResponse>, Status> {
        let req = request.into_inner();

        let email_id = Uuid::parse_str(&req.email_id)
            .map_err(|_| Status::invalid_argument("invalid email_id UUID"))?;
        let tenant_id = Uuid::parse_str(&req.tenant_id)
            .map_err(|_| Status::invalid_argument("invalid tenant_id UUID"))?;

        let result = pipeline::run_pipeline(&self.ctx, email_id, tenant_id)
            .await
            .map_err(|e| -> Status { e.into() })?;

        Ok(Response::new(IocExtractResponse {
            email_id: req.email_id,
            ioc_count: result.ioc_count,
            malicious_count: result.malicious_count,
            campaign_id: result.campaign_id,
            campaign_status: result.campaign_status,
        }))
    }

    async fn get_email_iocs(
        &self,
        request: Request<IocEmailRequest>,
    ) -> Result<Response<IocEmailResponse>, Status> {
        let req = request.into_inner();

        let email_id = Uuid::parse_str(&req.email_id)
            .map_err(|_| Status::invalid_argument("invalid email_id UUID"))?;
        let tenant_id = Uuid::parse_str(&req.tenant_id)
            .map_err(|_| Status::invalid_argument("invalid tenant_id UUID"))?;

        let nodes = db::nodes::get_by_email(&self.ctx.pool, email_id, tenant_id)
            .await
            .map_err(|e| -> Status { e.into() })?;

        let iocs: Vec<IocNodeProto> = nodes
            .into_iter()
            .map(|n| IocNodeProto {
                id: n.id.to_string(),
                ioc_type: n.ioc_type,
                ioc_value: n.ioc_value,
                threat_level: n.threat_level,
                intel_score: n.intel_score,
                sighting_count: n.sighting_count,
            })
            .collect();

        Ok(Response::new(IocEmailResponse { iocs }))
    }

    async fn get_ioc_node(
        &self,
        request: Request<IocNodeRequest>,
    ) -> Result<Response<IocNodeResponse>, Status> {
        let req = request.into_inner();

        let node_id = Uuid::parse_str(&req.ioc_id)
            .map_err(|_| Status::invalid_argument("invalid ioc_id UUID"))?;

        let node = db::nodes::get_by_id(&self.ctx.pool, node_id)
            .await
            .map_err(|e| -> Status { e.into() })?
            .ok_or_else(|| Status::not_found("IOC node not found"))?;

        Ok(Response::new(IocNodeResponse {
            node: Some(IocNodeProto {
                id: node.id.to_string(),
                ioc_type: node.ioc_type,
                ioc_value: node.ioc_value,
                threat_level: node.threat_level,
                intel_score: node.intel_score,
                sighting_count: node.sighting_count,
            }),
        }))
    }

    async fn get_campaign(
        &self,
        request: Request<CampaignRequest>,
    ) -> Result<Response<CampaignResponse>, Status> {
        let req = request.into_inner();

        let campaign_id = Uuid::parse_str(&req.campaign_id)
            .map_err(|_| Status::invalid_argument("invalid campaign_id UUID"))?;

        let campaign = db::campaigns::get_by_id(&self.ctx.pool, campaign_id)
            .await
            .map_err(|e| -> Status { e.into() })?
            .ok_or_else(|| Status::not_found("campaign not found"))?;

        Ok(Response::new(CampaignResponse {
            campaign_id: campaign.id.to_string(),
            name: campaign.campaign_name,
            status: campaign.status,
            member_count: campaign.member_count,
            ioc_fingerprint: campaign.ioc_fingerprint,
        }))
    }
}
