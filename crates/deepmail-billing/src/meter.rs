use crate::db::{costs, events};
use crate::error::BillingError;
use chrono::Utc;
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

const NAMESPACE_OID: Uuid = Uuid::from_bytes([
    0x6b, 0xa7, 0xb8, 0x12, 0x9d, 0xad, 0x11, 0xd1, 0x80, 0xb4, 0x00, 0xc0, 0x4f, 0xd4, 0x30,
    0xc8,
]);

pub struct UsageSummary {
    pub total_paise: i64,
    pub event_count: i64,
    pub cost_by_event_type: HashMap<String, i64>,
}

pub async fn record_event(
    pool: &PgPool,
    tenant_id: Uuid,
    email_id: Uuid,
    event_type: &str,
) -> Result<bool, BillingError> {
    let cost_paise = costs::get_cost(pool, event_type)
        .await
        .map_err(BillingError::Db)?
        .unwrap_or(0);

    let idem_input = format!("{}:{}:{}", tenant_id, email_id, event_type);
    let idempotency_key = Uuid::new_v5(&NAMESPACE_OID, idem_input.as_bytes());

    let billing_period = Utc::now().format("%Y-%m").to_string();

    let inserted = events::insert_meter_event(
        pool,
        idempotency_key,
        tenant_id,
        email_id,
        event_type,
        cost_paise,
        &billing_period,
    )
    .await
    .map_err(BillingError::Db)?;

    Ok(inserted.is_some())
}

pub async fn get_usage(
    pool: &PgPool,
    tenant_id: Uuid,
    period: &str,
) -> Result<UsageSummary, BillingError> {
    let lines = events::get_usage_summary(pool, tenant_id, period)
        .await
        .map_err(BillingError::Db)?;

    let mut total_paise: i64 = 0;
    let mut event_count: i64 = 0;
    let mut cost_by_event_type = HashMap::new();

    for line in &lines {
        total_paise += line.total_paise;
        event_count += line.count;
        cost_by_event_type.insert(line.event_type.clone(), line.total_paise);
    }

    Ok(UsageSummary {
        total_paise,
        event_count,
        cost_by_event_type,
    })
}
