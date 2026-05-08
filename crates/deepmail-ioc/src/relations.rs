/// IOC relationship inference from email context.

use std::collections::HashSet;
use std::net::IpAddr;

use uuid::Uuid;

/// A normalized IOC with its DB node id.
#[derive(Debug, Clone)]
pub struct NormalizedIoc {
    pub node_id: Uuid,
    pub ioc_type: String,
    pub ioc_value: String,
    pub source: String,
}

/// An inferred relationship between two IOCs.
#[derive(Debug, Clone)]
pub struct IocRelation {
    pub source_ioc_id: Uuid,
    pub target_ioc_id: Uuid,
    pub relation_type: String,
    pub confidence: f32,
}

/// Infer relationships between IOCs from the same email.
pub fn infer_relations(iocs: &[NormalizedIoc], _email_id: Uuid) -> Vec<IocRelation> {
    let mut relations: Vec<IocRelation> = Vec::new();
    let mut seen: HashSet<(Uuid, Uuid, String)> = HashSet::new();

    let ips: Vec<&NormalizedIoc> = iocs.iter().filter(|i| i.ioc_type == "ip").collect();
    let domains: Vec<&NormalizedIoc> = iocs.iter().filter(|i| i.ioc_type == "domain").collect();
    let urls: Vec<&NormalizedIoc> = iocs.iter().filter(|i| i.ioc_type == "url").collect();
    let emails: Vec<&NormalizedIoc> = iocs.iter().filter(|i| i.ioc_type == "email").collect();
    let hashes: Vec<&NormalizedIoc> = iocs.iter().filter(|i| i.ioc_type == "hash").collect();

    // Rule 1: IP + Domain in same email → Domain RESOLVES_TO IP (confidence 0.6)
    for domain in &domains {
        for ip in &ips {
            let key = (domain.node_id, ip.node_id, "RESOLVES_TO".to_string());
            if seen.insert(key) {
                relations.push(IocRelation {
                    source_ioc_id: domain.node_id,
                    target_ioc_id: ip.node_id,
                    relation_type: "RESOLVES_TO".into(),
                    confidence: 0.6,
                });
            }
        }
    }

    // Rule 2: URL contains IP as host → URL HOSTED_ON IP (confidence 0.95)
    for url_ioc in &urls {
        if let Ok(parsed) = url::Url::parse(&url_ioc.ioc_value) {
            if let Some(host) = parsed.host_str() {
                // Check if host is an IP address
                if host.parse::<IpAddr>().is_ok() {
                    for ip in &ips {
                        if ip.ioc_value == host {
                            let key = (url_ioc.node_id, ip.node_id, "HOSTED_ON".to_string());
                            if seen.insert(key) {
                                relations.push(IocRelation {
                                    source_ioc_id: url_ioc.node_id,
                                    target_ioc_id: ip.node_id,
                                    relation_type: "HOSTED_ON".into(),
                                    confidence: 0.95,
                                });
                            }
                        }
                    }
                } else {
                    // Rule 3: URL hostname matches a domain IOC → URL HOSTED_ON Domain
                    let host_lower = host.to_lowercase();
                    for domain in &domains {
                        if domain.ioc_value == host_lower
                            || host_lower.ends_with(&format!(".{}", domain.ioc_value))
                        {
                            let key = (url_ioc.node_id, domain.node_id, "HOSTED_ON".to_string());
                            if seen.insert(key) {
                                relations.push(IocRelation {
                                    source_ioc_id: url_ioc.node_id,
                                    target_ioc_id: domain.node_id,
                                    relation_type: "HOSTED_ON".into(),
                                    confidence: 0.95,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // Rule 4: Domain is subdomain of another domain IOC
    for da in &domains {
        for db in &domains {
            if da.node_id == db.node_id {
                continue;
            }
            // da is subdomain of db: da ends with ".{db}" and db has at least 2 parts
            if da.ioc_value.ends_with(&format!(".{}", db.ioc_value))
                && db.ioc_value.matches('.').count() >= 1
            {
                let key = (da.node_id, db.node_id, "SUBDOMAIN_OF".to_string());
                if seen.insert(key) {
                    relations.push(IocRelation {
                        source_ioc_id: da.node_id,
                        target_ioc_id: db.node_id,
                        relation_type: "SUBDOMAIN_OF".into(),
                        confidence: 1.0,
                    });
                }
            }
        }
    }

    // Rule 5: Email IOC + IP from same source (header) → Email SENT_FROM IP
    for email_ioc in &emails {
        if email_ioc.source == "header" {
            // Pair with first header IP
            if let Some(ip) = ips.iter().find(|i| i.source == "header") {
                let key = (email_ioc.node_id, ip.node_id, "SENT_FROM".to_string());
                if seen.insert(key) {
                    relations.push(IocRelation {
                        source_ioc_id: email_ioc.node_id,
                        target_ioc_id: ip.node_id,
                        relation_type: "SENT_FROM".into(),
                        confidence: 0.8,
                    });
                }
            }
        }
    }

    // Rule 6: Attachment hash + URL with filename → URL DROPS Hash
    // (simplified: any hash from attachment paired with any URL from body)
    if !hashes.is_empty() && !urls.is_empty() {
        for hash in &hashes {
            if hash.source == "attachment" {
                for url_ioc in &urls {
                    let key = (url_ioc.node_id, hash.node_id, "DROPS".to_string());
                    if seen.insert(key) {
                        relations.push(IocRelation {
                            source_ioc_id: url_ioc.node_id,
                            target_ioc_id: hash.node_id,
                            relation_type: "DROPS".into(),
                            confidence: 0.7,
                        });
                        break; // One URL per hash is enough
                    }
                }
            }
        }
    }

    // Limit to max 200 relations per email
    if relations.len() > 200 {
        tracing::warn!(
            count = relations.len(),
            "truncating IOC relations to 200"
        );
        relations.truncate(200);
    }

    relations
}
