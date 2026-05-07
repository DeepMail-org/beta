use std::net::IpAddr;
use std::str::FromStr;
use std::sync::Arc;

use redis::aio::ConnectionManager;
use sqlx::PgPool;
use tonic::{Request, Response, Status};

use deepmail_common::proto::geo::{
    geo_intel_server::GeoIntel,
    EmailGeoRequest, EmailGeoResult,
    GeoEnrichment, GeoPoint,
    HopChainRequest, HopChainResponse,
    HopGeoDetail, IpRequest,
};

use crate::classifier::{self, IpType};
use crate::db::{analyses, hops};
use crate::enrichment::EnrichmentConfig;
use crate::geoip::GeoIpReader;
use crate::pipeline;

pub struct GeoService {
    pub geo_pool: PgPool,
    pub ingest_pool: PgPool,
    pub parser_pool: PgPool,
    pub redis: ConnectionManager,
    pub geoip: Arc<GeoIpReader>,
    pub http_client: reqwest::Client,
    pub enrichment_config: Arc<EnrichmentConfig>,
}

#[tonic::async_trait]
impl GeoIntel for GeoService {
    async fn resolve_ip(
        &self,
        request: Request<IpRequest>,
    ) -> Result<Response<GeoEnrichment>, Status> {
        let req = request.into_inner();
        let ip: IpAddr = IpAddr::from_str(&req.ip_address)
            .map_err(|_| Status::invalid_argument("invalid IP address"))?;

        let geo = self.geoip.lookup_city(ip);
        let asn = self.geoip.lookup_asn(ip);
        let ip_type = classifier::classify(&asn.as_org.clone().unwrap_or_default());

        Ok(Response::new(GeoEnrichment {
            ip_address: ip.to_string(),
            country_code: geo.country_code.unwrap_or_default(),
            country_name: geo.country_name.unwrap_or_default(),
            region_name: String::new(),
            city: geo.city.unwrap_or_default(),
            postal_code: geo.postal_code.unwrap_or_default(),
            latitude: geo.latitude.unwrap_or(0.0),
            longitude: geo.longitude.unwrap_or(0.0),
            accuracy_radius_km: geo.accuracy_radius.unwrap_or(0) as i32,
            timezone: geo.timezone.unwrap_or_default(),
            asn_number: asn.asn.unwrap_or(0) as i32,
            asn_org: asn.as_org.unwrap_or_default(),
            isp: String::new(),
            connection_type: String::new(),
            is_hosting: ip_type == IpType::Hosting,
            is_residential: ip_type == IpType::Residential,
            is_datacenter: ip_type == IpType::Hosting,
            reverse_dns: String::new(),
            ptr_match: false,
            confidence_score: 0.0,
        }))
    }

    async fn resolve_hop_chain(
        &self,
        request: Request<HopChainRequest>,
    ) -> Result<Response<HopChainResponse>, Status> {
        let req = request.into_inner();

        let email_id: uuid::Uuid = req.email_id
            .parse()
            .map_err(|_| Status::invalid_argument("invalid email_id UUID"))?;
        let tenant_id: uuid::Uuid = req.tenant_id
            .parse()
            .map_err(|_| Status::invalid_argument("invalid tenant_id UUID"))?;

        if let Ok(Some(existing)) = analyses::get_by_email_id(&self.geo_pool, email_id).await {
            let hop_rows = hops::list_by_analysis(&self.geo_pool, existing.id)
                .await
                .unwrap_or_default();
            let proto_hops = hop_rows_to_proto(&hop_rows);
            return Ok(Response::new(HopChainResponse {
                hops: proto_hops,
                anomaly_score: existing.risk_score,
                routing_anomaly: existing.risk_score >= 0.35,
                anomaly_reasons: Vec::new(),
            }));
        }

        let state = pipeline::PipelineState {
            geo_pool: self.geo_pool.clone(),
            ingest_pool: self.ingest_pool.clone(),
            parser_pool: self.parser_pool.clone(),
            redis: self.redis.clone(),
            geoip: self.geoip.clone(),
            http_client: self.http_client.clone(),
            enrichment_config: self.enrichment_config.clone(),
        };

        let output = pipeline::analyze(&state, email_id, tenant_id)
            .await
            .map_err(|e| -> Status { e.into() })?;

        let existing = analyses::get_by_email_id(&self.geo_pool, email_id)
            .await
            .map_err(|e| -> Status { e.into() })?
            .ok_or_else(|| Status::internal("analysis not persisted"))?;

        let hop_rows = hops::list_by_analysis(&self.geo_pool, existing.id)
            .await
            .map_err(|e| -> Status { e.into() })?;
        let proto_hops = hop_rows_to_proto(&hop_rows);

        Ok(Response::new(HopChainResponse {
            hops: proto_hops,
            anomaly_score: output.risk_score,
            routing_anomaly: output.risk_score >= 0.35,
            anomaly_reasons: Vec::new(),
        }))
    }

    async fn get_email_geo(
        &self,
        request: Request<EmailGeoRequest>,
    ) -> Result<Response<EmailGeoResult>, Status> {
        let req = request.into_inner();
        let email_id: uuid::Uuid = req.email_id
            .parse()
            .map_err(|_| Status::invalid_argument("invalid email_id UUID"))?;

        let analysis = analyses::get_by_email_id(&self.geo_pool, email_id)
            .await
            .map_err(|e| -> Status { e.into() })?
            .ok_or_else(|| Status::not_found("no geo analysis for this email"))?;

        let hop_rows = hops::list_by_analysis(&self.geo_pool, analysis.id)
            .await
            .map_err(|e| -> Status { e.into() })?;

        let proto_hops = hop_rows_to_proto(&hop_rows);

        let map_points: Vec<GeoPoint> = hop_rows
            .iter()
            .filter(|h| h.latitude.is_some() && h.longitude.is_some())
            .map(|h| GeoPoint {
                ip: h.ip_address.clone(),
                lat: h.latitude.unwrap_or(0.0),
                lon: h.longitude.unwrap_or(0.0),
                label: h.city.clone().unwrap_or_default(),
                country: h.country_code.clone().unwrap_or_default(),
                asn_org: h.as_org.clone().unwrap_or_default(),
            })
            .collect();

        Ok(Response::new(EmailGeoResult {
            email_id: email_id.to_string(),
            hops: proto_hops,
            true_sender_country: analysis.origin_country.unwrap_or_default(),
            geo_anomaly_score: analysis.risk_score,
            map_points,
        }))
    }
}

fn hop_rows_to_proto(rows: &[hops::HopRow]) -> Vec<HopGeoDetail> {
    rows.iter().map(|h| {
        HopGeoDetail {
            hop_index: h.hop_index,
            ip_address: h.ip_address.clone(),
            geo: Some(GeoEnrichment {
                ip_address: h.ip_address.clone(),
                country_code: h.country_code.clone().unwrap_or_default(),
                country_name: h.country_name.clone().unwrap_or_default(),
                region_name: String::new(),
                city: h.city.clone().unwrap_or_default(),
                postal_code: h.postal_code.clone().unwrap_or_default(),
                latitude: h.latitude.unwrap_or(0.0),
                longitude: h.longitude.unwrap_or(0.0),
                accuracy_radius_km: 0,
                timezone: h.timezone.clone().unwrap_or_default(),
                asn_number: h.asn.unwrap_or(0),
                asn_org: h.as_org.clone().unwrap_or_default(),
                isp: String::new(),
                connection_type: String::new(),
                is_hosting: h.ip_type == "HOSTING",
                is_residential: h.ip_type == "RESIDENTIAL",
                is_datacenter: h.ip_type == "HOSTING",
                reverse_dns: String::new(),
                ptr_match: false,
                confidence_score: 0.0,
            }),
            anomaly_flag: h.is_tor || h.is_vpn || h.is_botnet,
            anomaly_reason: build_anomaly_reason(h),
        }
    }).collect()
}

fn build_anomaly_reason(h: &hops::HopRow) -> String {
    let mut reasons = Vec::new();
    if h.is_tor { reasons.push("TOR exit node"); }
    if h.is_vpn { reasons.push("VPN/proxy"); }
    if h.is_botnet { reasons.push("known botnet IP"); }
    reasons.join(", ")
}
