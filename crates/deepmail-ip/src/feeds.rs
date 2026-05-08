/// Blocklist feed fetcher: HTTP GET, parse by format, bulk DB upsert,
/// Redis SET population, per-feed error counter.

use std::net::IpAddr;
use std::str::FromStr;

use ipnetwork::IpNetwork;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::IpError;

#[derive(Debug, Clone)]
pub struct ParsedEntry {
    pub raw: String,
    pub ip: Option<IpAddr>,
    pub cidr: Option<IpNetwork>,
}

/// Parse a feed body according to its feed_name.
pub fn parse_feed(feed_name: &str, body: &str) -> Vec<ParsedEntry> {
    match feed_name {
        "spamhaus_drop" | "spamhaus_edrop" => parse_spamhaus(body),
        "dshield_top20" => parse_dshield(body),
        "alienvault_otx" => parse_alienvault(body),
        "tor_exits" => parse_ip_per_line_skip_hash(body),
        _ => parse_ip_per_line_skip_hash(body), // feodo, cins, emerging, blocklist_de, brute_force
    }
}

fn parse_ip_per_line_skip_hash(body: &str) -> Vec<ParsedEntry> {
    let mut entries = Vec::new();
    for line in body.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Ok(ip) = IpAddr::from_str(line) {
            entries.push(ParsedEntry {
                raw: line.to_string(),
                ip: Some(ip),
                cidr: None,
            });
        }
    }
    entries
}

fn parse_spamhaus(body: &str) -> Vec<ParsedEntry> {
    let mut entries = Vec::new();
    for line in body.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with(';') {
            continue;
        }
        // Format: "CIDR ; SBL-ref"
        let cidr_str = line.split(';').next().unwrap_or("").trim();
        if let Ok(net) = IpNetwork::from_str(cidr_str) {
            entries.push(ParsedEntry {
                raw: cidr_str.to_string(),
                ip: None,
                cidr: Some(net),
            });
        }
    }
    entries
}

fn parse_dshield(body: &str) -> Vec<ParsedEntry> {
    let mut entries = Vec::new();
    let mut header_lines = 0;
    for line in body.lines() {
        let line = line.trim();
        if line.starts_with('#') {
            header_lines += 1;
            continue;
        }
        if header_lines < 4 {
            header_lines += 1;
            continue;
        }
        if line.is_empty() {
            continue;
        }
        if let Ok(ip) = IpAddr::from_str(line) {
            entries.push(ParsedEntry {
                raw: line.to_string(),
                ip: Some(ip),
                cidr: None,
            });
        }
    }
    entries
}

fn parse_alienvault(body: &str) -> Vec<ParsedEntry> {
    let mut entries = Vec::new();
    for line in body.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        // Format: "IP # category score"
        let ip_str = line.split('#').next().unwrap_or("").trim();
        if let Ok(ip) = IpAddr::from_str(ip_str) {
            entries.push(ParsedEntry {
                raw: ip_str.to_string(),
                ip: Some(ip),
                cidr: None,
            });
        }
    }
    entries
}

/// Refresh a single feed: fetch, parse, upsert to DB, populate Redis SET.
pub async fn refresh_feed(
    pool: &PgPool,
    client: &reqwest::Client,
    redis: &mut ConnectionManager,
    feed_id: Uuid,
    feed_name: &str,
    feed_url: &str,
) -> Result<usize, IpError> {
    tracing::info!(feed = feed_name, "refreshing blocklist feed");

    let resp = client.get(feed_url).send().await?;
    if !resp.status().is_success() {
        return Err(IpError::Http(format!(
            "feed {} returned status {}",
            feed_name,
            resp.status()
        )));
    }

    let body = resp.text().await?;
    let entries = parse_feed(feed_name, &body);
    let count = entries.len();

    if count == 0 {
        tracing::warn!(feed = feed_name, "feed returned 0 entries");
        return Ok(0);
    }

    // Bulk upsert to database in batches of 500
    bulk_upsert_entries(pool, feed_id, &entries).await?;

    // Populate Redis SET
    let redis_key = format!("deepmail:ip:feed:{feed_name}");
    // Delete and re-create the set
    let _: Result<(), _> = redis.del(&redis_key).await;

    // SADD in chunks to avoid huge single command
    let chunk_size = 500;
    let members: Vec<String> = entries
        .iter()
        .map(|e| {
            if let Some(ip) = e.ip {
                ip.to_string()
            } else if let Some(cidr) = e.cidr {
                cidr.to_string()
            } else {
                e.raw.clone()
            }
        })
        .collect();

    for chunk in members.chunks(chunk_size) {
        let _: Result<(), _> = redis.sadd(&redis_key, chunk).await;
    }

    // Set TTL of 4 hours
    let _: Result<(), _> = redis.expire(&redis_key, 14400).await;

    // Update feed metadata
    crate::db::feeds::mark_fetched(pool, feed_id, count as i32).await?;

    tracing::info!(feed = feed_name, entries = count, "feed refresh complete");
    Ok(count)
}

async fn bulk_upsert_entries(
    pool: &PgPool,
    feed_id: Uuid,
    entries: &[ParsedEntry],
) -> Result<(), IpError> {
    let chunk_size = 500;
    for chunk in entries.chunks(chunk_size) {
        let mut tx = pool.begin().await?;

        for entry in chunk {
            if let Some(ip) = entry.ip {
                sqlx::query(
                    r#"INSERT INTO ip_blocklist_entries (feed_id, ip_address, raw_value)
                       VALUES ($1, $2::inet, $3)
                       ON CONFLICT (feed_id, ip_address)
                       DO UPDATE SET last_seen = now()"#,
                )
                .bind(feed_id)
                .bind(ip.to_string())
                .bind(&entry.raw)
                .execute(&mut *tx)
                .await?;
            } else if let Some(cidr) = entry.cidr {
                sqlx::query(
                    r#"INSERT INTO ip_blocklist_entries (feed_id, cidr_range, raw_value)
                       VALUES ($1, $2::cidr, $3)
                       ON CONFLICT DO NOTHING"#,
                )
                .bind(feed_id)
                .bind(cidr.to_string())
                .bind(&entry.raw)
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await?;
    }
    Ok(())
}

/// Check if an IP is a member of a specific feed's Redis SET.
pub async fn check_ip_in_feed(
    redis: &mut ConnectionManager,
    feed_name: &str,
    ip: IpAddr,
) -> Result<bool, IpError> {
    let key = format!("deepmail:ip:feed:{feed_name}");
    let is_member: bool = redis.sismember(&key, ip.to_string()).await?;
    Ok(is_member)
}

/// Refresh all active feeds. Errors on individual feeds are logged but do
/// NOT abort the entire refresh cycle.
pub async fn refresh_all_feeds(
    pool: &PgPool,
    client: &reqwest::Client,
    redis: &mut ConnectionManager,
) -> Result<(), IpError> {
    let feeds = crate::db::feeds::list_active_feeds(pool).await?;

    for feed in &feeds {
        match refresh_feed(pool, client, redis, feed.id, &feed.feed_name, &feed.feed_url).await {
            Ok(count) => {
                tracing::info!(feed = %feed.feed_name, entries = count, "feed refreshed");
            }
            Err(e) => {
                tracing::warn!(feed = %feed.feed_name, error = %e, "feed refresh failed");
                if let Err(db_err) = crate::db::feeds::increment_error(pool, feed.id).await {
                    tracing::error!(feed = %feed.feed_name, error = %db_err, "failed to increment error counter");
                }
            }
        }
    }

    Ok(())
}

/// Background task: refresh all feeds on a recurring interval.
pub async fn refresh_loop(
    pool: PgPool,
    client: reqwest::Client,
    mut redis: ConnectionManager,
    interval_hours: u64,
) {
    let interval = std::time::Duration::from_secs(interval_hours * 3600);
    loop {
        if let Err(e) = refresh_all_feeds(&pool, &client, &mut redis).await {
            tracing::error!(error = %e, "feed refresh loop error");
        }
        tokio::time::sleep(interval).await;
    }
}
