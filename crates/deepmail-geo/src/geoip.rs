use std::net::IpAddr;
use std::sync::Arc;

use maxminddb::Reader;

pub struct GeoIpReader {
    city_reader: Option<Arc<Reader<Vec<u8>>>>,
    asn_reader: Option<Arc<Reader<Vec<u8>>>>,
}

#[derive(Debug, Clone, Default)]
pub struct GeoData {
    pub country_code: Option<String>,
    pub country_name: Option<String>,
    pub city: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub postal_code: Option<String>,
    pub timezone: Option<String>,
    pub accuracy_radius: Option<u16>,
}

#[derive(Debug, Clone, Default)]
pub struct AsnData {
    pub asn: Option<u32>,
    pub as_org: Option<String>,
    pub network: Option<String>,
}

impl GeoIpReader {
    pub fn open(city_path: &str, asn_path: &str) -> Self {
        let city_reader = match Reader::open_readfile(city_path) {
            Ok(r) => {
                tracing::info!(path = city_path, "GeoLite2-City mmdb loaded");
                Some(Arc::new(r))
            }
            Err(e) => {
                tracing::warn!(path = city_path, error = %e, "failed to open city mmdb — geo lookups will return empty");
                None
            }
        };

        let asn_reader = match Reader::open_readfile(asn_path) {
            Ok(r) => {
                tracing::info!(path = asn_path, "GeoLite2-ASN mmdb loaded");
                Some(Arc::new(r))
            }
            Err(e) => {
                tracing::warn!(path = asn_path, error = %e, "failed to open ASN mmdb — ASN lookups will return empty");
                None
            }
        };

        Self { city_reader, asn_reader }
    }

    pub fn lookup_city(&self, ip: IpAddr) -> GeoData {
        let reader = match &self.city_reader {
            Some(r) => r,
            None => return GeoData::default(),
        };

        let lookup = match reader.lookup(ip) {
            Ok(lr) => lr,
            Err(e) => {
                tracing::debug!(ip = %ip, error = %e, "city lookup miss");
                return GeoData::default();
            }
        };

        let city: maxminddb::geoip2::City = match lookup.decode() {
            Ok(Some(c)) => c,
            Ok(None) => return GeoData::default(),
            Err(e) => {
                tracing::debug!(ip = %ip, error = %e, "city decode error");
                return GeoData::default();
            }
        };

        let country_code = city.country.iso_code.map(|s| s.to_string());
        let country_name = city.country.names.english.map(|s| s.to_string());
        let city_name = city.city.names.english.map(|s| s.to_string());
        let latitude = city.location.latitude;
        let longitude = city.location.longitude;
        let accuracy_radius = city.location.accuracy_radius;
        let postal_code = city.postal.code.map(|s| s.to_string());
        let timezone = city.location.time_zone.map(|s| s.to_string());

        GeoData {
            country_code,
            country_name,
            city: city_name,
            latitude,
            longitude,
            postal_code,
            timezone,
            accuracy_radius,
        }
    }

    pub fn lookup_asn(&self, ip: IpAddr) -> AsnData {
        let reader = match &self.asn_reader {
            Some(r) => r,
            None => return AsnData::default(),
        };

        let lookup = match reader.lookup(ip) {
            Ok(lr) => lr,
            Err(e) => {
                tracing::debug!(ip = %ip, error = %e, "ASN lookup miss");
                return AsnData::default();
            }
        };

        let asn: maxminddb::geoip2::Asn = match lookup.decode() {
            Ok(Some(a)) => a,
            Ok(None) => return AsnData::default(),
            Err(e) => {
                tracing::debug!(ip = %ip, error = %e, "ASN decode error");
                return AsnData::default();
            }
        };

        AsnData {
            asn: asn.autonomous_system_number,
            as_org: asn.autonomous_system_organization.map(|s| s.to_string()),
            network: None,
        }
    }
}
