/// Extract domains from email using cross-service DB queries.

use std::collections::HashSet;

use once_cell::sync::Lazy;
use regex::Regex;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::HomographError;

/// Regex to extract domain after @ in email addresses / headers.
static RE_DOMAIN_AT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"@([a-zA-Z0-9.\-]+)").unwrap()
});

/// Extract a domain from a header value (email address format).
fn extract_domain_from_header(value: &str) -> Option<String> {
    // Try to find domain after @
    if let Some(cap) = RE_DOMAIN_AT.captures(value) {
        if let Some(m) = cap.get(1) {
            let domain = m.as_str().to_lowercase();
            if domain.contains('.') {
                return Some(domain);
            }
        }
    }
    None
}

/// Extract all domains for an email from IOC DB + parser DB.
///
/// Returns deduplicated domain list. Never returns an error for parser
/// failures — IOC domains alone are sufficient.
pub async fn extract_domains_from_email(
    parser_pool: &PgPool,
    ioc_pool: &PgPool,
    email_id: Uuid,
) -> Result<Vec<String>, HomographError> {
    let mut domains: HashSet<String> = HashSet::new();

    // Query 1: IOC DB — domain nodes for this email
    let ioc_rows: Vec<(String,)> = sqlx::query_as(
        r#"SELECT n.ioc_value FROM ioc_nodes n
           JOIN email_ioc_occurrences o ON o.ioc_node_id = n.id
           WHERE o.email_id = $1 AND n.ioc_type = 'domain'"#,
    )
    .bind(email_id)
    .fetch_all(ioc_pool)
    .await
    .unwrap_or_default();

    for (domain,) in ioc_rows {
        let d = domain.to_lowercase().trim_end_matches('.').to_string();
        if d.contains('.') && d.len() >= 4 {
            domains.insert(d);
        }
    }

    // Query 2: Parser DB — header values for from/reply-to/return-path
    let parser_domains = extract_from_parser(parser_pool, email_id).await;
    match parser_domains {
        Ok(pdoms) => {
            for d in pdoms {
                domains.insert(d);
            }
        }
        Err(e) => {
            tracing::warn!(
                %email_id,
                error = %e,
                "parser DB query failed, using IOC domains only"
            );
        }
    }

    tracing::info!(
        %email_id,
        domain_count = domains.len(),
        "extracted domains for homograph analysis"
    );

    Ok(domains.into_iter().collect())
}

/// Extract domains from parser DB headers (best-effort).
async fn extract_from_parser(
    parser_pool: &PgPool,
    email_id: Uuid,
) -> Result<Vec<String>, HomographError> {
    // Get parsed_email_id
    let parsed_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM parsed_emails WHERE email_id = $1 LIMIT 1",
    )
    .bind(email_id)
    .fetch_optional(parser_pool)
    .await?;

    let parsed_id = match parsed_id {
        Some(id) => id,
        None => return Ok(vec![]),
    };

    // Get header values
    let rows: Vec<(String, String)> = sqlx::query_as(
        r#"SELECT name, value FROM email_headers
           WHERE parsed_email_id = $1
             AND name IN ('from','reply-to','return-path','x-mailer')"#,
    )
    .bind(parsed_id)
    .fetch_all(parser_pool)
    .await?;

    let mut domains = Vec::new();
    for (_name, value) in rows {
        if let Some(domain) = extract_domain_from_header(&value) {
            if domain.len() >= 4 {
                domains.push(domain);
            }
        }
    }

    Ok(domains)
}
