use crate::error::GraphError;
use crate::neo4j::{GraphEdgeData, GraphNodeData};
use crate::sync::SyncCtx;
use deepmail_common::proto::graph::graph_service_server::GraphService;
use deepmail_common::proto::graph::{
    C2PathRequest, C2PathResponse, CampaignGraphRequest, CampaignGraphResponse, GraphEdge,
    GraphNode, GraphPath, GraphQueryRequest, GraphQueryResponse, GraphSyncRequest,
    GraphSyncResponse, ShadowInfraRequest, ShadowInfraResponse,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

pub struct GraphServiceHandler {
    ctx: Arc<SyncCtx>,
}

impl GraphServiceHandler {
    pub fn new(ctx: Arc<SyncCtx>) -> Self {
        Self { ctx }
    }
}

fn node_data_to_proto(n: &GraphNodeData) -> GraphNode {
    GraphNode {
        id: n.neo4j_id.clone(),
        node_type: n.node_type.clone(),
        value: n.value.clone(),
        threat_score: n.threat_score,
        properties: n.properties.clone(),
    }
}

fn edge_data_to_proto(e: &GraphEdgeData) -> GraphEdge {
    GraphEdge {
        source_id: e.source_id.clone(),
        target_id: e.target_id.clone(),
        relation_type: e.relation_type.clone(),
        confidence: e.confidence,
    }
}

fn ioc_type_to_label(ioc_type: &str) -> Result<&'static str, GraphError> {
    match ioc_type {
        "ip" => Ok("IpAddress"),
        "domain" => Ok("Domain"),
        "url" => Ok("Url"),
        "hash" => Ok("FileHash"),
        "email" => Ok("Sender"),
        _ => Err(GraphError::InvalidArgument(format!(
            "unknown ioc_type: {ioc_type}"
        ))),
    }
}

#[tonic::async_trait]
impl GraphService for GraphServiceHandler {
    async fn sync_email(
        &self,
        request: Request<GraphSyncRequest>,
    ) -> Result<Response<GraphSyncResponse>, Status> {
        let req = request.into_inner();

        let email_id = Uuid::parse_str(&req.email_id)
            .map_err(|_| Status::invalid_argument("invalid email_id"))?;
        let tenant_id = Uuid::parse_str(&req.tenant_id)
            .map_err(|_| Status::invalid_argument("invalid tenant_id"))?;

        let (nodes, edges) = crate::sync::sync_email(&self.ctx, email_id, tenant_id)
            .await
            .map_err(|e| -> Status { e.into() })?;

        Ok(Response::new(GraphSyncResponse {
            email_id: email_id.to_string(),
            nodes_created: nodes,
            edges_created: edges,
            status: "completed".to_string(),
        }))
    }

    async fn query_related(
        &self,
        request: Request<GraphQueryRequest>,
    ) -> Result<Response<GraphQueryResponse>, Status> {
        let req = request.into_inner();

        let label = ioc_type_to_label(&req.ioc_type).map_err(|e| -> Status { e.into() })?;
        let depth = if req.depth > 0 { req.depth } else { 2 };

        let (nodes_data, edges_data) = self
            .ctx
            .neo4j
            .query_related(&req.ioc_value, label, depth)
            .await
            .map_err(|e| -> Status { e.into() })?;

        Ok(Response::new(GraphQueryResponse {
            nodes: nodes_data.iter().map(node_data_to_proto).collect(),
            edges: edges_data.iter().map(edge_data_to_proto).collect(),
        }))
    }

    async fn find_c2_paths(
        &self,
        request: Request<C2PathRequest>,
    ) -> Result<Response<C2PathResponse>, Status> {
        let req = request.into_inner();

        Uuid::parse_str(&req.email_id)
            .map_err(|_| Status::invalid_argument("invalid email_id"))?;

        let paths_data = self
            .ctx
            .neo4j
            .find_c2_paths(&req.email_id)
            .await
            .map_err(|e| -> Status { e.into() })?;

        let paths = paths_data
            .into_iter()
            .map(|path_nodes| GraphPath {
                nodes: path_nodes.iter().map(node_data_to_proto).collect(),
                path_type: "c2_chain".to_string(),
            })
            .collect();

        Ok(Response::new(C2PathResponse { paths }))
    }

    async fn find_shadow_infra(
        &self,
        request: Request<ShadowInfraRequest>,
    ) -> Result<Response<ShadowInfraResponse>, Status> {
        let req = request.into_inner();

        if req.ip_value.is_empty() {
            return Err(Status::invalid_argument("ip_value is required"));
        }

        let nodes_data = self
            .ctx
            .neo4j
            .find_shadow_infra(&req.ip_value)
            .await
            .map_err(|e| -> Status { e.into() })?;

        let summary = if nodes_data.is_empty() {
            "No related infrastructure found".to_string()
        } else {
            let unique_types: std::collections::HashSet<&str> =
                nodes_data.iter().map(|n| n.node_type.as_str()).collect();
            format!(
                "Found {} related nodes ({} unique types) for IP {}",
                nodes_data.len(),
                unique_types.len(),
                req.ip_value
            )
        };

        Ok(Response::new(ShadowInfraResponse {
            related_nodes: nodes_data.iter().map(node_data_to_proto).collect(),
            summary,
        }))
    }

    async fn get_campaign_graph(
        &self,
        request: Request<CampaignGraphRequest>,
    ) -> Result<Response<CampaignGraphResponse>, Status> {
        let req = request.into_inner();

        if req.campaign_id.is_empty() {
            return Err(Status::invalid_argument("campaign_id is required"));
        }

        let (nodes_data, edges_data, email_count) = self
            .ctx
            .neo4j
            .get_campaign_graph(&req.campaign_id)
            .await
            .map_err(|e| -> Status { e.into() })?;

        Ok(Response::new(CampaignGraphResponse {
            nodes: nodes_data.iter().map(node_data_to_proto).collect(),
            edges: edges_data.iter().map(edge_data_to_proto).collect(),
            email_count,
        }))
    }
}
