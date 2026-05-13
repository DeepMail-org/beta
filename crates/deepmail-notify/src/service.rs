use crate::db::configs;
use crate::dispatcher::{self, NotifyCtx};
use crate::pipeline::NotifyEvent;
use deepmail_common::proto::notify::notify_service_server::NotifyService;
use deepmail_common::proto::notify::{
    GetConfigRequest, NotifyConfigResponse, SendNotificationRequest, SendNotificationResponse,
    UpsertConfigRequest,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

pub struct NotifyGrpcService {
    ctx: Arc<NotifyCtx>,
}

impl NotifyGrpcService {
    pub fn new(ctx: Arc<NotifyCtx>) -> Self {
        Self { ctx }
    }
}

#[tonic::async_trait]
impl NotifyService for NotifyGrpcService {
    async fn send_notification(
        &self,
        request: Request<SendNotificationRequest>,
    ) -> Result<Response<SendNotificationResponse>, Status> {
        let req = request.into_inner();

        let email_id = Uuid::parse_str(&req.email_id)
            .map_err(|_| Status::invalid_argument("invalid email_id"))?;
        let tenant_id = Uuid::parse_str(&req.tenant_id)
            .map_err(|_| Status::invalid_argument("invalid tenant_id"))?;

        let details: serde_json::Value = if req.details.is_empty() {
            serde_json::Value::Object(serde_json::Map::new())
        } else {
            serde_json::from_str(&req.details)
                .map_err(|e| Status::invalid_argument(format!("invalid details JSON: {}", e)))?
        };

        let event = NotifyEvent {
            email_id,
            tenant_id,
            event_type: req.event_type,
            verdict: req.verdict,
            score: req.score,
            details,
            timestamp: chrono::Utc::now(),
        };

        let result = dispatcher::dispatch_event(&self.ctx, &event).await;

        let mut channels: u32 = 0;
        if result.websocket_sent {
            channels += 1;
        }
        if result.email_sent {
            channels += 1;
        }
        if result.webhook_sent {
            channels += 1;
        }

        Ok(Response::new(SendNotificationResponse {
            delivered: channels > 0,
            channels_notified: channels,
            websocket_sent: result.websocket_sent,
            email_sent: result.email_sent,
            webhook_sent: result.webhook_sent,
        }))
    }

    async fn get_config(
        &self,
        request: Request<GetConfigRequest>,
    ) -> Result<Response<NotifyConfigResponse>, Status> {
        let req = request.into_inner();

        let tenant_id = Uuid::parse_str(&req.tenant_id)
            .map_err(|_| Status::invalid_argument("invalid tenant_id"))?;

        let row = configs::get_config(&self.ctx.own_pool, tenant_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        match row {
            Some(cfg) => Ok(Response::new(NotifyConfigResponse {
                config_id: cfg.id.to_string(),
                tenant_id: cfg.tenant_id.to_string(),
                webhook_url: cfg.webhook_url.unwrap_or_default(),
                webhook_active: cfg.webhook_active,
                smtp_enabled: cfg.smtp_enabled,
                min_severity: cfg.min_severity,
                created_at: cfg.created_at.to_rfc3339(),
                updated_at: cfg.updated_at.to_rfc3339(),
            })),
            None => Ok(Response::new(NotifyConfigResponse {
                config_id: String::new(),
                tenant_id: tenant_id.to_string(),
                webhook_url: String::new(),
                webhook_active: false,
                smtp_enabled: true,
                min_severity: self.ctx.config.min_severity_default.clone(),
                created_at: String::new(),
                updated_at: String::new(),
            })),
        }
    }

    async fn upsert_config(
        &self,
        request: Request<UpsertConfigRequest>,
    ) -> Result<Response<NotifyConfigResponse>, Status> {
        let req = request.into_inner();

        let tenant_id = Uuid::parse_str(&req.tenant_id)
            .map_err(|_| Status::invalid_argument("invalid tenant_id"))?;

        let valid_severities = ["CLEAN", "LOW_RISK", "SUSPICIOUS", "PHISHING", "MALICIOUS"];
        if !req.min_severity.is_empty() && !valid_severities.contains(&req.min_severity.as_str()) {
            return Err(Status::invalid_argument(format!(
                "min_severity must be one of: {}",
                valid_severities.join(", ")
            )));
        }

        let severity = if req.min_severity.is_empty() {
            &self.ctx.config.min_severity_default
        } else {
            &req.min_severity
        };

        let config_id = configs::upsert_config(
            &self.ctx.own_pool,
            tenant_id,
            &req.webhook_url,
            &req.webhook_secret,
            req.smtp_enabled,
            severity,
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        let row = configs::get_config(&self.ctx.own_pool, tenant_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::internal("config not found after upsert"))?;

        Ok(Response::new(NotifyConfigResponse {
            config_id: config_id.to_string(),
            tenant_id: row.tenant_id.to_string(),
            webhook_url: row.webhook_url.unwrap_or_default(),
            webhook_active: row.webhook_active,
            smtp_enabled: row.smtp_enabled,
            min_severity: row.min_severity,
            created_at: row.created_at.to_rfc3339(),
            updated_at: row.updated_at.to_rfc3339(),
        }))
    }
}
