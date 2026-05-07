use std::net::IpAddr;
use std::str::FromStr;
use std::time::Duration;

use ipnetwork::IpNetwork;
use redis::AsyncCommands;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::error::GeoError;

const REDIS_KEY: &str = "deepmail:vpn:cidrs";
const TTL_SECONDS: i64 = 43200;

const VPN_ASNS: &[u32] = &[
    9009,   // M247
    20473,  // Choopa/Vultr
    23959,  // ExpressVPN
    35366,  // NordVPN
    62904,  // Surfshark
    209854, // Mullvad
    396507, // ProtonVPN
    14618,  // AWS
];

pub async fn fetch_asn_prefixes(asn: u32, timeout_secs: u64) -> Result<Vec<IpNetwork>, GeoError> {
    let addr = "whois.radb.net:43";
    let stream = tokio::time::timeout(
        Duration::from_secs(timeout_secs),
        TcpStream::connect(addr),
    )
    .await
    .map_err(|_| GeoError::Dns(format!("WHOIS timeout connecting for AS{asn}")))?
    .map_err(|e| GeoError::Dns(format!("WHOIS connect AS{asn}: {e}")))?;

    let (mut reader, mut writer) = tokio::io::split(stream);

    let query = format!("!gAS{asn}\r\n");
    tokio::time::timeout(Duration::from_secs(timeout_secs), writer.write_all(query.as_bytes()))
        .await
        .map_err(|_| GeoError::Dns(format!("WHOIS write timeout AS{asn}")))?
        .map_err(|e| GeoError::Dns(format!("WHOIS write AS{asn}: {e}")))?;

    let mut buf = vec![0u8; 65536];
    let n = tokio::time::timeout(Duration::from_secs(timeout_secs), reader.read(&mut buf))
        .await
        .map_err(|_| GeoError::Dns(format!("WHOIS read timeout AS{asn}")))?
        .map_err(|e| GeoError::Dns(format!("WHOIS read AS{asn}: {e}")))?;

    let response = String::from_utf8_lossy(&buf[..n]);
    let mut networks = Vec::new();

    for line in response.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('A') || trimmed.starts_with('C') || trimmed.starts_with('D') {
            continue;
        }
        for token in trimmed.split_whitespace() {
            if let Ok(net) = IpNetwork::from_str(token) {
                networks.push(net);
            }
        }
    }

    tracing::debug!(asn = asn, prefixes = networks.len(), "fetched ASN prefixes");
    Ok(networks)
}

pub async fn cache_vpn_cidrs(
    redis: &mut redis::aio::ConnectionManager,
    networks: &[IpNetwork],
) -> Result<(), GeoError> {
    if networks.is_empty() {
        return Ok(());
    }

    let _: () = redis::cmd("DEL")
        .arg(REDIS_KEY)
        .query_async(redis)
        .await?;

    let cidr_strings: Vec<String> = networks.iter().map(|n| n.to_string()).collect();

    let _: () = redis::cmd("SADD")
        .arg(REDIS_KEY)
        .arg(&cidr_strings)
        .query_async(redis)
        .await?;

    let _: () = redis.expire(REDIS_KEY, TTL_SECONDS).await?;

    tracing::info!(count = networks.len(), "cached VPN CIDRs in Redis");
    Ok(())
}

pub async fn is_vpn_ip(
    redis: &mut redis::aio::ConnectionManager,
    ip: IpAddr,
    as_org: &str,
) -> Result<bool, GeoError> {
    let lower = as_org.to_lowercase();
    if lower.contains("vpn") || lower.contains("proxy") || lower.contains("anonymi") {
        return Ok(true);
    }

    let cidrs: Vec<String> = redis::cmd("SMEMBERS")
        .arg(REDIS_KEY)
        .query_async(redis)
        .await
        .unwrap_or_default();

    for cidr_str in &cidrs {
        if let Ok(net) = IpNetwork::from_str(cidr_str) {
            if net.contains(ip) {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

pub async fn refresh_loop(
    mut redis: redis::aio::ConnectionManager,
    whois_timeout_secs: u64,
) {
    loop {
        let mut all_networks = Vec::new();
        for &asn in VPN_ASNS {
            match fetch_asn_prefixes(asn, whois_timeout_secs).await {
                Ok(nets) => all_networks.extend(nets),
                Err(e) => {
                    tracing::warn!(asn = asn, error = %e, "failed to fetch ASN prefixes");
                }
            }
        }

        if let Err(e) = cache_vpn_cidrs(&mut redis, &all_networks).await {
            tracing::error!(error = %e, "failed to cache VPN CIDRs");
        } else {
            tracing::info!(total_cidrs = all_networks.len(), "VPN CIDR refresh complete");
        }

        tokio::time::sleep(std::time::Duration::from_secs(43200)).await;
    }
}
