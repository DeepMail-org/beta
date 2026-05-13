use std::sync::Arc;
use std::time::Duration;

use axum::{extract::State, response::IntoResponse, Json};
use chrono::Utc;
use serde_json::json;
use tonic_health::pb::health_client::HealthClient;
use tonic_health::pb::HealthCheckRequest;
use tonic::transport::Channel;

use crate::GatewayCtx;

async fn check_service(url: &str) -> bool {
    let channel = match Channel::from_shared(url.to_string()) {
        Ok(ep) => {
            match tokio::time::timeout(
                Duration::from_secs(2),
                ep.connect_timeout(Duration::from_secs(2)).connect(),
            )
            .await
            {
                Ok(Ok(ch)) => ch,
                _ => return false,
            }
        }
        Err(_) => return false,
    };

    let mut client = HealthClient::new(channel);
    let req = HealthCheckRequest { service: String::new() };
    match tokio::time::timeout(Duration::from_secs(2), client.check(req)).await {
        Ok(Ok(_)) => true,
        _ => false,
    }
}

pub async fn health_check(
    State(ctx): State<Arc<GatewayCtx>>,
) -> impl IntoResponse {
    let cfg = &ctx.config;

    let checks = tokio::join!(
        check_service(&cfg.auth_grpc_url),
        check_service(&cfg.scoring_grpc_url),
        check_service(&cfg.report_grpc_url),
        check_service(&cfg.ioc_grpc_url),
        check_service(&cfg.graph_grpc_url),
        check_service(&cfg.tenant_grpc_url),
        check_service(&cfg.billing_grpc_url),
        check_service(&cfg.notify_grpc_url),
    );

    let services = json!({
        "auth": checks.0,
        "scoring": checks.1,
        "report": checks.2,
        "ioc": checks.3,
        "graph": checks.4,
        "tenant": checks.5,
        "billing": checks.6,
        "notify": checks.7,
    });

    let all: [bool; 8] = [
        checks.0, checks.1, checks.2, checks.3,
        checks.4, checks.5, checks.6, checks.7,
    ];
    let healthy_count = all.iter().filter(|&&v| v).count();

    let status = if healthy_count == 8 {
        "ok"
    } else if healthy_count > 0 {
        "degraded"
    } else {
        "down"
    };

    Json(json!({
        "status": status,
        "services": services,
        "timestamp": Utc::now().to_rfc3339()
    }))
}
