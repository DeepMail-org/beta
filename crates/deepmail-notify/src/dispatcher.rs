use crate::config::NotifyConfig;
use crate::db::{configs, logs};
use crate::pipeline::{self, NotifyEvent};
use crate::smtp;
use crate::webhook;
use crate::ws::WsHub;
use lettre::{AsyncSmtpTransport, Tokio1Executor};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

pub struct NotifyCtx {
    pub own_pool: Arc<PgPool>,
    pub auth_pool: Arc<PgPool>,
    pub tenant_pool: Arc<PgPool>,
    pub report_pool: Arc<PgPool>,
    pub hub: WsHub,
    pub smtp_transport: Option<Arc<AsyncSmtpTransport<Tokio1Executor>>>,
    pub http_client: reqwest::Client,
    pub config: NotifyConfig,
}

pub struct DispatchResult {
    pub websocket_sent: bool,
    pub ws_recipients: usize,
    pub email_sent: bool,
    pub webhook_sent: bool,
}

pub async fn dispatch_event(ctx: &NotifyCtx, event: &NotifyEvent) -> DispatchResult {
    let notify_config = configs::get_config(&ctx.own_pool, event.tenant_id)
        .await
        .ok()
        .flatten();

    let min_severity = notify_config
        .as_ref()
        .map(|c| c.min_severity.as_str())
        .unwrap_or(&ctx.config.min_severity_default);

    let event_rank = pipeline::severity_rank(&event.verdict);
    let threshold_rank = pipeline::severity_rank(min_severity);

    if event_rank < threshold_rank {
        tracing::debug!(
            email_id = %event.email_id,
            verdict = event.verdict.as_str(),
            min_severity = min_severity,
            "below severity threshold, skipping"
        );
        return DispatchResult {
            websocket_sent: false,
            ws_recipients: 0,
            email_sent: false,
            webhook_sent: false,
        };
    }

    let (ws_result, email_result, webhook_result) = tokio::join!(
        dispatch_websocket(ctx, event),
        dispatch_email(ctx, event, &notify_config),
        dispatch_webhook(ctx, event, &notify_config),
    );

    DispatchResult {
        websocket_sent: ws_result.0,
        ws_recipients: ws_result.1,
        email_sent: email_result,
        webhook_sent: webhook_result,
    }
}

async fn dispatch_websocket(ctx: &NotifyCtx, event: &NotifyEvent) -> (bool, usize) {
    let ws_msg = serde_json::json!({
        "type": "event",
        "data": event,
    })
    .to_string();

    let count = ctx
        .hub
        .broadcast_to_tenant(event.tenant_id, &ws_msg)
        .await;

    if count > 0 {
        let payload_val = serde_json::to_value(event).ok();
        let _ = logs::insert_log(
            &ctx.own_pool,
            event.tenant_id,
            event.email_id,
            &event.event_type,
            "websocket",
            "sent",
            None,
            payload_val.as_ref(),
            None,
        )
        .await;
    }

    (count > 0, count)
}

async fn dispatch_email(
    ctx: &NotifyCtx,
    event: &NotifyEvent,
    notify_config: &Option<configs::NotifyConfigRow>,
) -> bool {
    let smtp_enabled = notify_config
        .as_ref()
        .map(|c| c.smtp_enabled)
        .unwrap_or(true);

    if !smtp_enabled {
        return false;
    }

    let transport = match &ctx.smtp_transport {
        Some(t) => t,
        None => return false,
    };

    let should_send = match should_send_immediate_email(ctx, event.tenant_id).await {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(error = %e, "failed to check digest schedule, sending email anyway");
            true
        }
    };

    if !should_send {
        return false;
    }

    let emails = match get_tenant_member_emails(ctx, event.tenant_id).await {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!(
                tenant_id = %event.tenant_id,
                error = %e,
                "failed to get tenant member emails"
            );
            return false;
        }
    };

    if emails.is_empty() {
        return false;
    }

    let subject = format!(
        "[DeepMail] {} — {} detected",
        event.event_type, event.verdict
    );
    let html = smtp::render_alert_html(event, &ctx.config.dashboard_url);
    let mut any_sent = false;

    for email_addr in &emails {
        match smtp::send_alert_email(transport, email_addr, &subject, &html, &ctx.config.smtp_from)
            .await
        {
            Ok(()) => {
                any_sent = true;
                let _ = logs::insert_log(
                    &ctx.own_pool,
                    event.tenant_id,
                    event.email_id,
                    &event.event_type,
                    "email",
                    "sent",
                    Some(email_addr),
                    None,
                    None,
                )
                .await;
            }
            Err(e) => {
                tracing::warn!(
                    to = email_addr.as_str(),
                    error = %e,
                    "failed to send alert email"
                );
                let _ = logs::insert_log(
                    &ctx.own_pool,
                    event.tenant_id,
                    event.email_id,
                    &event.event_type,
                    "email",
                    "failed",
                    Some(email_addr),
                    None,
                    Some(&e.to_string()),
                )
                .await;
            }
        }
    }

    any_sent
}

async fn dispatch_webhook(
    ctx: &NotifyCtx,
    event: &NotifyEvent,
    notify_config: &Option<configs::NotifyConfigRow>,
) -> bool {
    let cfg = match notify_config.as_ref() {
        Some(c) if c.webhook_active => c,
        _ => return false,
    };

    let url = match &cfg.webhook_url {
        Some(u) if !u.is_empty() => u.as_str(),
        _ => return false,
    };

    let secret = cfg.webhook_secret.as_deref().unwrap_or("");
    let delivery_id = Uuid::new_v4();

    match webhook::deliver_with_retry(
        &ctx.http_client,
        url,
        secret,
        event,
        delivery_id,
        ctx.config.webhook_timeout_secs,
    )
    .await
    {
        Ok(attempts) => {
            let _ = logs::insert_log(
                &ctx.own_pool,
                event.tenant_id,
                event.email_id,
                &event.event_type,
                "webhook",
                "sent",
                Some(url),
                None,
                None,
            )
            .await;
            tracing::info!(
                url = url,
                delivery_id = %delivery_id,
                attempts = attempts,
                "webhook delivered"
            );
            true
        }
        Err(e) => {
            let _ = logs::insert_log(
                &ctx.own_pool,
                event.tenant_id,
                event.email_id,
                &event.event_type,
                "webhook",
                "failed",
                Some(url),
                None,
                Some(&e.to_string()),
            )
            .await;
            tracing::error!(
                url = url,
                delivery_id = %delivery_id,
                error = %e,
                "webhook delivery failed"
            );
            false
        }
    }
}

async fn get_tenant_member_emails(
    ctx: &NotifyCtx,
    tenant_id: Uuid,
) -> Result<Vec<String>, crate::error::NotifyError> {
    let user_ids: Vec<(Uuid,)> = sqlx::query_as(
        "SELECT user_id FROM tenant_members WHERE tenant_id = $1",
    )
    .bind(tenant_id)
    .fetch_all(ctx.tenant_pool.as_ref())
    .await
    .map_err(crate::error::NotifyError::Db)?;

    if user_ids.is_empty() {
        return Ok(Vec::new());
    }

    let ids: Vec<Uuid> = user_ids.into_iter().map(|(id,)| id).collect();

    let emails: Vec<(String,)> = sqlx::query_as(
        "SELECT CAST(email AS TEXT) FROM users WHERE id = ANY($1) AND is_active = true AND deleted_at IS NULL",
    )
    .bind(&ids)
    .fetch_all(ctx.auth_pool.as_ref())
    .await
    .map_err(crate::error::NotifyError::Db)?;

    Ok(emails.into_iter().map(|(e,)| e).collect())
}

async fn should_send_immediate_email(
    ctx: &NotifyCtx,
    tenant_id: Uuid,
) -> Result<bool, crate::error::NotifyError> {
    let schedule: Option<(String, bool)> = sqlx::query_as(
        "SELECT frequency, is_active FROM digest_schedules WHERE tenant_id = $1 LIMIT 1",
    )
    .bind(tenant_id)
    .fetch_optional(ctx.report_pool.as_ref())
    .await
    .map_err(crate::error::NotifyError::Db)?;

    match schedule {
        None => Ok(true),
        Some((frequency, is_active)) => {
            if !is_active {
                return Ok(true);
            }
            match frequency.as_str() {
                "never" => Ok(false),
                "daily" | "weekly" | "monthly" => Ok(false),
                _ => Ok(true),
            }
        }
    }
}
