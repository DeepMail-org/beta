/// BGP prefix lookup via RIPE Stat API + bogon detection.

use std::net::IpAddr;

use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};

use crate::error::IpError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BgpInfo {
    pub prefix: Option<String>,
    pub asn: Option<i32>,
    pub holder: Option<String>,
    pub announced: bool,
}

#[derive(Debug, Deserialize)]
struct RipeResponse {
    data: Option<RipeData>,
}

#[derive(Debug, Deserialize)]
struct RipeData {
    prefix: Option<String>,
    asns: Option<Vec<RipeAsn>>,
    announced: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct RipeAsn {
    asn: Option<i64>,
    holder: Option<String>,
}

fn redis_key(ip: IpAddr) -> String {
    format!("deepmail:ip:bgp:{ip}")
}

pub async fn fetch_bgp(
    client: &reqwest::Client,
    redis: &mut ConnectionManager,
    ip: IpAddr,
) -> Result<BgpInfo, IpError> {
    // Check Redis cache first
    let key = redis_key(ip);
    let cached: Option<String> = redis.get(&key).await?;
    if let Some(json) = cached {
        if let Ok(info) = serde_json::from_str::<BgpInfo>(&json) {
            return Ok(info);
        }
    }

    let url = format!(
        "https://stat.ripe.net/data/prefix-overview/data.json?resource={ip}"
    );

    let resp = client.get(&url).send().await?;

    if !resp.status().is_success() {
        tracing::warn!(ip = %ip, status = %resp.status(), "RIPE Stat API error");
        let info = BgpInfo {
            prefix: None,
            asn: None,
            holder: None,
            announced: false,
        };
        return Ok(info);
    }

    let body: RipeResponse = resp.json().await.map_err(|e| IpError::Http(e.to_string()))?;

    let info = match body.data {
        Some(data) => {
            let first_asn = data.asns.as_ref().and_then(|a| a.first());
            BgpInfo {
                prefix: data.prefix,
                asn: first_asn.and_then(|a| a.asn.map(|v| v as i32)),
                holder: first_asn.and_then(|a| a.holder.clone()),
                announced: data.announced.unwrap_or(false),
            }
        }
        None => BgpInfo {
            prefix: None,
            asn: None,
            holder: None,
            announced: false,
        },
    };

    // Cache in Redis with 6-hour TTL
    if let Ok(json) = serde_json::to_string(&info) {
        let _: Result<(), _> = redis.set_ex(&key, &json, 21600).await;
    }

    Ok(info)
}

/// Check whether an IP is in bogon (non-routable) space.
///
/// Covers RFC 1918 (private), RFC 5737 (TEST-NET), RFC 3927 (link-local),
/// RFC 6598 (shared/CGN), loopback, multicast, 0.0.0.0/8, 240.0.0.0/4.
pub fn is_bogon(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            let octets = v4.octets();

            // 0.0.0.0/8
            if octets[0] == 0 { return true; }
            // Loopback 127.0.0.0/8
            if octets[0] == 127 { return true; }
            // 10.0.0.0/8
            if octets[0] == 10 { return true; }
            // 172.16.0.0/12
            if octets[0] == 172 && (octets[1] >= 16 && octets[1] <= 31) { return true; }
            // 192.168.0.0/16
            if octets[0] == 192 && octets[1] == 168 { return true; }
            // Link-local 169.254.0.0/16 (RFC 3927)
            if octets[0] == 169 && octets[1] == 254 { return true; }
            // Shared address space 100.64.0.0/10 (RFC 6598)
            if octets[0] == 100 && (octets[1] >= 64 && octets[1] <= 127) { return true; }
            // TEST-NET-1 192.0.2.0/24 (RFC 5737)
            if octets[0] == 192 && octets[1] == 0 && octets[2] == 2 { return true; }
            // TEST-NET-2 198.51.100.0/24 (RFC 5737)
            if octets[0] == 198 && octets[1] == 51 && octets[2] == 100 { return true; }
            // TEST-NET-3 203.0.113.0/24 (RFC 5737)
            if octets[0] == 203 && octets[1] == 0 && octets[2] == 113 { return true; }
            // Benchmarking 198.18.0.0/15
            if octets[0] == 198 && (octets[1] == 18 || octets[1] == 19) { return true; }
            // Multicast 224.0.0.0/4
            if octets[0] >= 224 && octets[0] <= 239 { return true; }
            // Reserved / future 240.0.0.0/4
            if octets[0] >= 240 { return true; }
            // IETF Protocol Assignments 192.0.0.0/24 (RFC 6890)
            if octets[0] == 192 && octets[1] == 0 && octets[2] == 0 { return true; }

            false
        }
        IpAddr::V6(v6) => {
            v6.is_loopback() || v6.is_multicast() || {
                // Link-local, site-local, unique-local
                let segments = v6.segments();
                let first = segments[0];
                // ::1 (loopback) already handled
                // :: (unspecified)
                v6.is_unspecified()
                    // fe80::/10 link-local
                    || (first & 0xffc0) == 0xfe80
                    // fc00::/7 unique-local
                    || (first & 0xfe00) == 0xfc00
            }
        }
    }
}

/// Check whether an IP is in private (non-routable) address space.
/// Convenience wrapper used to filter hop IPs before enrichment.
pub fn is_private(ip: IpAddr) -> bool {
    is_bogon(ip)
}
