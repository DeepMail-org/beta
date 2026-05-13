use sqlx::PgPool;
use uuid::Uuid;

pub async fn insert_request_log(
    pool: &PgPool,
    tenant_id: Option<Uuid>,
    user_id: Option<Uuid>,
    method: &str,
    path: &str,
    status_code: i32,
    latency_ms: i32,
    ip_address: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO gateway_request_log (tenant_id, user_id, method, path, status_code, latency_ms, ip_address) \
         VALUES ($1, $2, $3, $4, $5, $6, $7)"
    )
    .bind(tenant_id)
    .bind(user_id)
    .bind(method)
    .bind(path)
    .bind(status_code)
    .bind(latency_ms)
    .bind(ip_address)
    .execute(pool)
    .await?;
    Ok(())
}
