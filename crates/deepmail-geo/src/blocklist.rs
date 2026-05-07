use std::net::IpAddr;
use std::str::FromStr;

use ipnetwork::IpNetwork;
use sqlx::PgPool;

use crate::error::GeoError;

#[derive(Debug)]
pub struct IpEntry {
    pub ip: Option<IpAddr>,
    pub cidr: Option<IpNetwork>,
    pub threat_type: String,
}

pub async fn refresh_feed(
    client: &reqwest::Client,
    feed_name: &str,
    feed_url: &str,
) -> Result<Vec<IpEntry>, GeoError> {
    let resp = client
        .get(feed_url)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| GeoError::Http(format!("blocklist fetch {feed_name}: {e}")))?;

    let text = resp
        .text()
        .await
        .map_err(|e| GeoError::Http(format!("blocklist body {feed_name}: {e}")))?;

    let threat_type = infer_threat_type(feed_name);
    let mut entries = Vec::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';') {
            continue;
        }

        let token = if trimmed.contains(';') {
            trimmed.split(';').next().unwrap_or("").trim()
        } else {
            trimmed
        };

        if token.is_empty() {
            continue;
        }

        if let Ok(net) = IpNetwork::from_str(token) {
            entries.push(IpEntry {
                ip: None,
                cidr: Some(net),
                threat_type: threat_type.clone(),
            });
        } else if let Ok(ip) = IpAddr::from_str(token) {
            entries.push(IpEntry {
                ip: Some(ip),
                cidr: None,
                threat_type: threat_type.clone(),
            });
        }
    }

    tracing::info!(feed = feed_name, entries = entries.len(), "parsed blocklist feed");
    Ok(entries)
}

fn infer_threat_type(feed_name: &str) -> String {
    let lower = feed_name.to_lowercase();
    if lower.contains("feodo") {
        "botnet".to_string()
    } else if lower.contains("spamhaus") {
        "spam".to_string()
    } else if lower.contains("cins") {
        "scanner".to_string()
    } else {
        "malware".to_string()
    }
}

pub async fn bulk_upsert_entries(
    pool: &PgPool,
    feed_id: uuid::Uuid,
    entries: &[IpEntry],
) -> Result<usize, GeoError> {
    let mut count = 0usize;
    for entry in entries {
        let ip_str = entry.ip.map(|ip| ip.to_string());
        let cidr_str = entry.cidr.map(|c| c.to_string());

        let result = sqlx::query(
            r#"INSERT INTO geo_blocklist_entries (feed_id, ip_address, cidr_range, threat_type)
               VALUES ($1, $2::inet, $3::cidr, $4)
               ON CONFLICT DO NOTHING"#,
        )
        .bind(feed_id)
        .bind(&ip_str)
        .bind(&cidr_str)
        .bind(&entry.threat_type)
        .execute(pool)
        .await;

        match result {
            Ok(_) => count += 1,
            Err(e) => {
                tracing::debug!(error = %e, "blocklist entry insert skip");
            }
        }
    }
    Ok(count)
}

pub async fn update_feed_status(
    pool: &PgPool,
    feed_id: uuid::Uuid,
    entry_count: i32,
) -> Result<(), GeoError> {
    sqlx::query(
        r#"UPDATE geo_blocklist_feeds
           SET last_fetched_at = now(), entry_count = $1
           WHERE id = $2"#,
    )
    .bind(entry_count)
    .bind(feed_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn check_ip_in_blocklist(pool: &PgPool, ip: IpAddr) -> Result<bool, GeoError> {
    let ip_str = ip.to_string();
    let row = sqlx::query_scalar::<_, bool>(
        r#"SELECT EXISTS(
             SELECT 1 FROM geo_blocklist_entries
             WHERE ip_address = $1::inet OR cidr_range >>= $1::inet
           )"#,
    )
    .bind(&ip_str)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

#[derive(Debug)]
struct FeedRow {
    id: uuid::Uuid,
    feed_name: String,
    feed_url: String,
}

pub async fn refresh_all_feeds(
    pool: &PgPool,
    client: &reqwest::Client,
) -> Result<(), GeoError> {
    let feeds = sqlx::query_as!(
        FeedRow,
        r#"SELECT id, feed_name, feed_url FROM geo_blocklist_feeds WHERE is_active = true"#,
    )
    .fetch_all(pool)
    .await?;

    for feed in &feeds {
        match refresh_feed(client, &feed.feed_name, &feed.feed_url).await {
            Ok(entries) => {
                let count = bulk_upsert_entries(pool, feed.id, &entries).await.unwrap_or(0);
                let _ = update_feed_status(pool, feed.id, count as i32).await;
                tracing::info!(feed = %feed.feed_name, inserted = count, "blocklist feed refreshed");
            }
            Err(e) => {
                tracing::error!(feed = %feed.feed_name, error = %e, "blocklist feed refresh failed");
            }
        }
    }
    Ok(())
}

pub async fn refresh_loop(pool: PgPool, client: reqwest::Client) {
    loop {
        if let Err(e) = refresh_all_feeds(&pool, &client).await {
            tracing::error!(error = %e, "blocklist refresh cycle failed");
        }
        tokio::time::sleep(std::time::Duration::from_secs(21600)).await;
    }
}
