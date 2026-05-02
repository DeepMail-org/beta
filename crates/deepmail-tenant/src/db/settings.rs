//! CRUD for tenant_settings table.

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::TenantError;

#[derive(Debug, Clone)]
pub struct SettingsRow {
    pub tenant_id: Uuid,
    pub max_upload_size_mb: i32,
    pub sandbox_url_enabled: bool,
    pub sandbox_file_enabled: bool,
    pub sandbox_dynamic_enabled: bool,
    pub auto_report_on_critical: bool,
    pub retention_days: i32,
    pub notify_email: Option<String>,
    pub notify_slack_webhook: Option<String>,
    pub notify_custom_webhook: Option<String>,
    pub notify_on_critical: bool,
    pub notify_on_high: bool,
    pub notify_on_medium: bool,
}

/// Fetch settings for a tenant.
pub async fn get_settings(
    pool: &PgPool,
    tenant_id: Uuid,
) -> Result<Option<SettingsRow>, TenantError> {
    let row = sqlx::query!(
        r#"
        SELECT tenant_id, max_upload_size_mb,
               sandbox_url_enabled, sandbox_file_enabled,
               sandbox_dynamic_enabled, auto_report_on_critical,
               retention_days,
               notify_email AS "notify_email: String",
               notify_slack_webhook, notify_custom_webhook,
               notify_on_critical, notify_on_high, notify_on_medium
        FROM tenant_settings
        WHERE tenant_id = $1
        "#,
        tenant_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(TenantError::Database)?;

    Ok(row.map(|r| SettingsRow {
        tenant_id: r.tenant_id,
        max_upload_size_mb: r.max_upload_size_mb,
        sandbox_url_enabled: r.sandbox_url_enabled,
        sandbox_file_enabled: r.sandbox_file_enabled,
        sandbox_dynamic_enabled: r.sandbox_dynamic_enabled,
        auto_report_on_critical: r.auto_report_on_critical,
        retention_days: r.retention_days,
        notify_email: r.notify_email,
        notify_slack_webhook: r.notify_slack_webhook,
        notify_custom_webhook: r.notify_custom_webhook,
        notify_on_critical: r.notify_on_critical,
        notify_on_high: r.notify_on_high,
        notify_on_medium: r.notify_on_medium,
    }))
}

/// Upsert sandbox toggle settings for a tenant.
pub async fn update_sandbox_settings(
    pool: &PgPool,
    tenant_id: Uuid,
    sandbox_url_enabled: bool,
    sandbox_file_enabled: bool,
    sandbox_dynamic_enabled: bool,
) -> Result<(), TenantError> {
    sqlx::query!(
        r#"
        UPDATE tenant_settings
        SET sandbox_url_enabled     = $2,
            sandbox_file_enabled    = $3,
            sandbox_dynamic_enabled = $4,
            updated_at              = now()
        WHERE tenant_id = $1
        "#,
        tenant_id,
        sandbox_url_enabled,
        sandbox_file_enabled,
        sandbox_dynamic_enabled,
    )
    .execute(pool)
    .await
    .map_err(TenantError::Database)?;
    Ok(())
}
