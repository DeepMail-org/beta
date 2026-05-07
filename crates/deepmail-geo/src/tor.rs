use std::net::IpAddr;
use std::str::FromStr;

use redis::AsyncCommands;

use crate::error::GeoError;

const TOR_EXIT_LIST_URL: &str = "https://check.torproject.org/torbulkexitlist";
const REDIS_KEY: &str = "deepmail:tor:exits";
const TTL_SECONDS: i64 = 21600;

pub async fn fetch_tor_exits(client: &reqwest::Client) -> Result<Vec<IpAddr>, GeoError> {
    let resp = client
        .get(TOR_EXIT_LIST_URL)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| GeoError::Http(format!("tor exit list fetch: {e}")))?;

    let text = resp
        .text()
        .await
        .map_err(|e| GeoError::Http(format!("tor exit list body: {e}")))?;

    let mut ips = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Ok(ip) = IpAddr::from_str(trimmed) {
            ips.push(ip);
        }
    }

    tracing::info!(count = ips.len(), "fetched TOR exit list");
    Ok(ips)
}

pub async fn cache_tor_exits(
    redis: &mut redis::aio::ConnectionManager,
    ips: &[IpAddr],
) -> Result<(), GeoError> {
    if ips.is_empty() {
        return Ok(());
    }

    let _: () = redis::cmd("DEL")
        .arg(REDIS_KEY)
        .query_async(redis)
        .await?;

    let ip_strings: Vec<String> = ips.iter().map(|ip| ip.to_string()).collect();

    let _: () = redis::cmd("SADD")
        .arg(REDIS_KEY)
        .arg(&ip_strings)
        .query_async(redis)
        .await?;

    let _: () = redis.expire(REDIS_KEY, TTL_SECONDS).await?;

    tracing::info!(count = ips.len(), ttl = TTL_SECONDS, "cached TOR exits in Redis");
    Ok(())
}

pub async fn is_tor_exit(
    redis: &mut redis::aio::ConnectionManager,
    ip: IpAddr,
) -> Result<bool, GeoError> {
    let result: bool = redis.sismember(REDIS_KEY, ip.to_string()).await?;
    Ok(result)
}

pub async fn refresh_loop(
    http_client: reqwest::Client,
    mut redis: redis::aio::ConnectionManager,
) {
    loop {
        match fetch_tor_exits(&http_client).await {
            Ok(ips) => {
                if let Err(e) = cache_tor_exits(&mut redis, &ips).await {
                    tracing::error!(error = %e, "failed to cache TOR exits");
                }
            }
            Err(e) => {
                tracing::error!(error = %e, "failed to fetch TOR exit list, retrying in 60s");
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                continue;
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(21600)).await;
    }
}
