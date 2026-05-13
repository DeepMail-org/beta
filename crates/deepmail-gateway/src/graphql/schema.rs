use std::sync::Arc;

use async_graphql::{Context, EmptyMutation, EmptySubscription, Object, Schema, SimpleObject, ID};
use chrono::Utc;

use crate::auth_middleware::AuthClaims;
use crate::GatewayCtx;

#[derive(SimpleObject)]
pub struct EmailScore {
    pub email_id: ID,
    pub final_score: f64,
    pub final_verdict: String,
    pub signals_available: i32,
}

#[derive(SimpleObject)]
pub struct EmailReportInfo {
    pub email_id: ID,
    pub json_s3_key: Option<String>,
    pub html_s3_key: Option<String>,
    pub status: String,
    pub final_verdict: Option<String>,
}

#[derive(SimpleObject)]
pub struct TenantUsage {
    pub billing_period: String,
    pub total_paise: i64,
    pub event_count: i64,
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn email_score(
        &self,
        ctx: &Context<'_>,
        email_id: ID,
    ) -> async_graphql::Result<Option<EmailScore>> {
        let claims = ctx.data::<AuthClaims>()?;
        let gateway = ctx.data::<Arc<GatewayCtx>>()?;

        match gateway
            .scoring_client
            .get_score(email_id.as_str(), &claims.tenant_id.to_string())
            .await
        {
            Ok(resp) => Ok(Some(EmailScore {
                email_id: ID::from(resp.email_id),
                final_score: resp.final_score as f64,
                final_verdict: resp.final_verdict,
                signals_available: resp.signals_available,
            })),
            Err(crate::error::GatewayError::Grpc(s))
                if s.code() == tonic::Code::NotFound =>
            {
                Ok(None)
            }
            Err(e) => Err(async_graphql::Error::new(e.to_string())),
        }
    }

    async fn email_report(
        &self,
        ctx: &Context<'_>,
        email_id: ID,
    ) -> async_graphql::Result<Option<EmailReportInfo>> {
        let claims = ctx.data::<AuthClaims>()?;
        let gateway = ctx.data::<Arc<GatewayCtx>>()?;

        match gateway
            .report_client
            .get_report(email_id.as_str(), &claims.tenant_id.to_string(), "json")
            .await
        {
            Ok(resp) => Ok(Some(EmailReportInfo {
                email_id: ID::from(resp.email_id),
                json_s3_key: if resp.json_s3_key.is_empty() { None } else { Some(resp.json_s3_key) },
                html_s3_key: if resp.html_s3_key.is_empty() { None } else { Some(resp.html_s3_key) },
                status: resp.status,
                final_verdict: if resp.final_verdict.is_empty() { None } else { Some(resp.final_verdict) },
            })),
            Err(crate::error::GatewayError::Grpc(s))
                if s.code() == tonic::Code::NotFound =>
            {
                Ok(None)
            }
            Err(e) => Err(async_graphql::Error::new(e.to_string())),
        }
    }

    async fn tenant_usage(
        &self,
        ctx: &Context<'_>,
        period: Option<String>,
    ) -> async_graphql::Result<TenantUsage> {
        let claims = ctx.data::<AuthClaims>()?;
        let gateway = ctx.data::<Arc<GatewayCtx>>()?;

        let period = period.unwrap_or_else(|| Utc::now().format("%Y-%m").to_string());

        let resp = gateway
            .billing_client
            .get_usage(&claims.tenant_id.to_string(), &period)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(TenantUsage {
            billing_period: resp.billing_period,
            total_paise: resp.total_paise,
            event_count: resp.event_count,
        })
    }
}

pub type AppSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

pub fn build_schema() -> AppSchema {
    Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish()
}
