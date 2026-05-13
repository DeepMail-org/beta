use tonic::transport::Channel;

use deepmail_common::proto::auth::{
    self, auth_service_client::AuthServiceClient,
};
use deepmail_common::proto::billing::{
    self, billing_service_client::BillingServiceClient,
};
use deepmail_common::proto::graph::{
    self, graph_service_client::GraphServiceClient,
};
use deepmail_common::proto::ioc::{
    self, ioc_extractor_client::IocExtractorClient,
};
use deepmail_common::proto::notify::{
    self, notify_service_client::NotifyServiceClient,
};
use deepmail_common::proto::report::{
    self, report_service_client::ReportServiceClient,
};
use deepmail_common::proto::scoring::{
    self, threat_scorer_client::ThreatScorerClient,
};
use deepmail_common::proto::tenant::{
    self, tenant_service_client::TenantServiceClient,
};

use crate::error::GatewayError;

fn lazy_channel(url: &str) -> Result<Channel, GatewayError> {
    Channel::from_shared(url.to_string())
        .map_err(|e| GatewayError::Validation(format!("invalid gRPC URL: {e}")))
        .map(|endpoint| endpoint.connect_lazy())
}

#[derive(Clone)]
pub struct AuthClient {
    inner: AuthServiceClient<Channel>,
}

impl AuthClient {
    pub fn new(url: &str) -> Result<Self, GatewayError> {
        Ok(Self { inner: AuthServiceClient::new(lazy_channel(url)?) })
    }

    pub async fn validate_token(
        &self,
        token: &str,
    ) -> Result<auth::ValidateTokenResponse, GatewayError> {
        let mut c = self.inner.clone();
        c.validate_token(auth::ValidateTokenRequest { token: token.to_string() })
            .await
            .map(|r| r.into_inner())
            .map_err(GatewayError::Grpc)
    }

    pub async fn register(
        &self,
        req: auth::RegisterRequest,
    ) -> Result<auth::RegisterResponse, GatewayError> {
        let mut c = self.inner.clone();
        c.register(req).await.map(|r| r.into_inner()).map_err(GatewayError::Grpc)
    }

    pub async fn login(
        &self,
        req: auth::LoginRequest,
    ) -> Result<auth::LoginResponse, GatewayError> {
        let mut c = self.inner.clone();
        c.login(req).await.map(|r| r.into_inner()).map_err(GatewayError::Grpc)
    }

    pub async fn verify_otp(
        &self,
        req: auth::VerifyOtpRequest,
    ) -> Result<auth::VerifyOtpResponse, GatewayError> {
        let mut c = self.inner.clone();
        c.verify_otp(req).await.map(|r| r.into_inner()).map_err(GatewayError::Grpc)
    }

    pub async fn refresh_token(
        &self,
        req: auth::RefreshTokenRequest,
    ) -> Result<auth::RefreshTokenResponse, GatewayError> {
        let mut c = self.inner.clone();
        c.refresh_token(req).await.map(|r| r.into_inner()).map_err(GatewayError::Grpc)
    }
}

#[derive(Clone)]
pub struct ScoringClient {
    inner: ThreatScorerClient<Channel>,
}

impl ScoringClient {
    pub fn new(url: &str) -> Result<Self, GatewayError> {
        Ok(Self { inner: ThreatScorerClient::new(lazy_channel(url)?) })
    }

    pub async fn get_score(
        &self,
        email_id: &str,
        tenant_id: &str,
    ) -> Result<scoring::ComputeScoreResponse, GatewayError> {
        let mut c = self.inner.clone();
        c.get_score(scoring::GetScoreRequest {
            email_id: email_id.to_string(),
            tenant_id: tenant_id.to_string(),
        })
        .await
        .map(|r| r.into_inner())
        .map_err(GatewayError::Grpc)
    }
}

#[derive(Clone)]
pub struct ReportClient {
    inner: ReportServiceClient<Channel>,
}

impl ReportClient {
    pub fn new(url: &str) -> Result<Self, GatewayError> {
        Ok(Self { inner: ReportServiceClient::new(lazy_channel(url)?) })
    }

    pub async fn get_report(
        &self,
        email_id: &str,
        tenant_id: &str,
        format: &str,
    ) -> Result<report::ReportResponse, GatewayError> {
        let mut c = self.inner.clone();
        c.get_report(report::GetReportRequest {
            email_id: email_id.to_string(),
            tenant_id: tenant_id.to_string(),
            format: format.to_string(),
        })
        .await
        .map(|r| r.into_inner())
        .map_err(GatewayError::Grpc)
    }
}

#[derive(Clone)]
pub struct IocClient {
    inner: IocExtractorClient<Channel>,
}

impl IocClient {
    pub fn new(url: &str) -> Result<Self, GatewayError> {
        Ok(Self { inner: IocExtractorClient::new(lazy_channel(url)?) })
    }

    pub async fn get_email_iocs(
        &self,
        email_id: &str,
        tenant_id: &str,
    ) -> Result<ioc::IocEmailResponse, GatewayError> {
        let mut c = self.inner.clone();
        c.get_email_iocs(ioc::IocEmailRequest {
            email_id: email_id.to_string(),
            tenant_id: tenant_id.to_string(),
        })
        .await
        .map(|r| r.into_inner())
        .map_err(GatewayError::Grpc)
    }
}

#[derive(Clone)]
pub struct GraphClient {
    inner: GraphServiceClient<Channel>,
}

impl GraphClient {
    pub fn new(url: &str) -> Result<Self, GatewayError> {
        Ok(Self { inner: GraphServiceClient::new(lazy_channel(url)?) })
    }

    pub async fn query_related(
        &self,
        ioc_value: &str,
        ioc_type: &str,
        depth: i32,
        tenant_id: &str,
    ) -> Result<graph::GraphQueryResponse, GatewayError> {
        let mut c = self.inner.clone();
        c.query_related(graph::GraphQueryRequest {
            ioc_value: ioc_value.to_string(),
            ioc_type: ioc_type.to_string(),
            depth,
            tenant_id: tenant_id.to_string(),
        })
        .await
        .map(|r| r.into_inner())
        .map_err(GatewayError::Grpc)
    }
}

#[derive(Clone)]
pub struct TenantClient {
    inner: TenantServiceClient<Channel>,
}

impl TenantClient {
    pub fn new(url: &str) -> Result<Self, GatewayError> {
        Ok(Self { inner: TenantServiceClient::new(lazy_channel(url)?) })
    }

    pub async fn get_tenant(
        &self,
        tenant_id: &str,
    ) -> Result<tenant::TenantResponse, GatewayError> {
        let mut c = self.inner.clone();
        c.get_tenant(tenant::GetTenantRequest { tenant_id: tenant_id.to_string() })
            .await
            .map(|r| r.into_inner())
            .map_err(GatewayError::Grpc)
    }

    pub async fn invite_member(
        &self,
        req: tenant::InviteMemberRequest,
    ) -> Result<tenant::InviteMemberResponse, GatewayError> {
        let mut c = self.inner.clone();
        c.invite_member(req).await.map(|r| r.into_inner()).map_err(GatewayError::Grpc)
    }
}

#[derive(Clone)]
pub struct BillingClient {
    inner: BillingServiceClient<Channel>,
}

impl BillingClient {
    pub fn new(url: &str) -> Result<Self, GatewayError> {
        Ok(Self { inner: BillingServiceClient::new(lazy_channel(url)?) })
    }

    pub async fn get_usage(
        &self,
        tenant_id: &str,
        billing_period: &str,
    ) -> Result<billing::GetUsageResponse, GatewayError> {
        let mut c = self.inner.clone();
        c.get_usage(billing::GetUsageRequest {
            tenant_id: tenant_id.to_string(),
            billing_period: billing_period.to_string(),
        })
        .await
        .map(|r| r.into_inner())
        .map_err(GatewayError::Grpc)
    }
}

#[derive(Clone)]
pub struct NotifyClient {
    inner: NotifyServiceClient<Channel>,
}

impl NotifyClient {
    pub fn new(url: &str) -> Result<Self, GatewayError> {
        Ok(Self { inner: NotifyServiceClient::new(lazy_channel(url)?) })
    }

    pub async fn get_config(
        &self,
        tenant_id: &str,
    ) -> Result<notify::NotifyConfigResponse, GatewayError> {
        let mut c = self.inner.clone();
        c.get_config(notify::GetConfigRequest { tenant_id: tenant_id.to_string() })
            .await
            .map(|r| r.into_inner())
            .map_err(GatewayError::Grpc)
    }

    pub async fn upsert_config(
        &self,
        req: notify::UpsertConfigRequest,
    ) -> Result<notify::NotifyConfigResponse, GatewayError> {
        let mut c = self.inner.clone();
        c.upsert_config(req).await.map(|r| r.into_inner()).map_err(GatewayError::Grpc)
    }
}
