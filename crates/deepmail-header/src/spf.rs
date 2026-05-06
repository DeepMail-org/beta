//! SPF record parsing and evaluation.
//!
//! Implements core RFC 7208 SPF evaluation.
//! Fully self-contained — no external SPF library dependency.
//!
//! Supported mechanisms: +all -all ~all ?all ip4 ip6 a mx include
//! Recursion depth for include: limited to 5 to prevent DoS.

use std::net::IpAddr;

use crate::dns::Resolver;

/// SPF evaluation result.
#[derive(Debug, Clone, PartialEq)]
pub enum SpfResult {
    Pass,
    Fail,
    SoftFail,
    Neutral,
    None,
    PermError,
    TempError,
}

impl SpfResult {
    pub fn as_str(&self) -> &'static str {
        match self {
            SpfResult::Pass      => "pass",
            SpfResult::Fail      => "fail",
            SpfResult::SoftFail  => "softfail",
            SpfResult::Neutral   => "neutral",
            SpfResult::None      => "none",
            SpfResult::PermError => "permerror",
            SpfResult::TempError => "temperror",
        }
    }
}

/// Evaluate SPF for a given sender IP and domain.
/// Returns the SPF result and the raw TXT record found (if any).
pub async fn evaluate(
    resolver: &Resolver,
    sender_ip: IpAddr,
    sender_domain: &str,
) -> (SpfResult, Option<String>) {
    evaluate_recursive(resolver, sender_ip, sender_domain, 0).await
}

/// Recursive SPF evaluation with depth limit.
async fn evaluate_recursive(
    resolver: &Resolver,
    sender_ip: IpAddr,
    domain: &str,
    depth: u8,
) -> (SpfResult, Option<String>) {
    if depth > 5 {
        return (SpfResult::PermError, None);
    }

    let result = resolver.lookup_txt(domain).await;

    if result.nxdomain || result.answers.is_empty() {
        return (SpfResult::None, None);
    }

    // Find the SPF record (starts with "v=spf1")
    let spf_record = match result
        .answers
        .iter()
        .find(|a| a.to_lowercase().starts_with("v=spf1"))
    {
        Some(r) => r.clone(),
        None => return (SpfResult::None, None),
    };

    let raw = spf_record.clone();

    // Parse mechanisms
    let parts: Vec<&str> = spf_record.split_whitespace().collect();
    // Skip "v=spf1"
    for mechanism in parts.iter().skip(1) {
        let (qualifier, rest) = parse_qualifier(mechanism);

        let matches = match_mechanism(resolver, sender_ip, domain, rest, depth).await;

        match matches {
            MechanismMatch::Match => {
                return (qualifier_to_result(qualifier), Some(raw));
            }
            MechanismMatch::NoMatch => continue,
            MechanismMatch::Error => {
                return (SpfResult::TempError, Some(raw));
            }
        }
    }

    // No mechanism matched — default result is neutral
    (SpfResult::Neutral, Some(raw))
}

#[derive(Debug)]
enum MechanismMatch { Match, NoMatch, Error }

/// Parse leading qualifier: + (pass), - (fail), ~ (softfail), ? (neutral).
/// Default qualifier when absent is + (pass).
fn parse_qualifier(mechanism: &str) -> (char, &str) {
    match mechanism.chars().next() {
        Some(q @ ('+' | '-' | '~' | '?')) => (q, &mechanism[1..]),
        _ => ('+', mechanism),
    }
}

fn qualifier_to_result(q: char) -> SpfResult {
    match q {
        '+' => SpfResult::Pass,
        '-' => SpfResult::Fail,
        '~' => SpfResult::SoftFail,
        '?' => SpfResult::Neutral,
        _   => SpfResult::Neutral,
    }
}

/// Check if a sender_ip matches a single SPF mechanism.
async fn match_mechanism(
    resolver: &Resolver,
    sender_ip: IpAddr,
    current_domain: &str,
    mechanism: &str,
    depth: u8,
) -> MechanismMatch {
    let lower = mechanism.to_lowercase();

    if lower == "all" {
        return MechanismMatch::Match;
    }

    // ip4:x.x.x.x or ip4:x.x.x.x/nn
    if let Some(rest) = lower.strip_prefix("ip4:") {
        if let IpAddr::V4(v4) = sender_ip {
            return if cidr_match_v4(v4, rest) {
                MechanismMatch::Match
            } else {
                MechanismMatch::NoMatch
            };
        }
        return MechanismMatch::NoMatch;
    }

    // ip6:addr or ip6:addr/prefix
    if let Some(rest) = lower.strip_prefix("ip6:") {
        if let IpAddr::V6(v6) = sender_ip {
            return if cidr_match_v6(v6, rest) {
                MechanismMatch::Match
            } else {
                MechanismMatch::NoMatch
            };
        }
        return MechanismMatch::NoMatch;
    }

    // a or a:domain or a/prefix or a:domain/prefix
    if lower == "a" || lower.starts_with("a:") || lower.starts_with("a/") {
        let (target_domain, _prefix) = parse_domain_prefix(&lower, "a", current_domain);
        return match_a_record(resolver, sender_ip, &target_domain).await;
    }

    // mx or mx:domain
    if lower == "mx" || lower.starts_with("mx:") || lower.starts_with("mx/") {
        let (target_domain, _prefix) = parse_domain_prefix(&lower, "mx", current_domain);
        return match_mx_record(resolver, sender_ip, &target_domain).await;
    }

    // include:domain
    if let Some(include_domain) = lower.strip_prefix("include:") {
        let (result, _) = Box::pin(evaluate_recursive(
            resolver,
            sender_ip,
            include_domain,
            depth + 1,
        ))
        .await;
        return match result {
            SpfResult::Pass     => MechanismMatch::Match,
            SpfResult::None
            | SpfResult::PermError => MechanismMatch::Error,
            _                   => MechanismMatch::NoMatch,
        };
    }

    // ptr — deprecated, always NoMatch for safety
    if lower == "ptr" || lower.starts_with("ptr:") {
        return MechanismMatch::NoMatch;
    }

    // exists:domain — check if domain has any A record
    if let Some(exists_domain) = lower.strip_prefix("exists:") {
        return match resolver.inner.lookup_ip(exists_domain).await {
            Ok(lookup) if lookup.iter().next().is_some() => MechanismMatch::Match,
            _ => MechanismMatch::NoMatch,
        };
    }

    // Unknown mechanism — PermError per RFC 7208
    MechanismMatch::Error
}

/// Parse "a:domain/prefix" or "mx:domain" into (domain, Option<u8>).
fn parse_domain_prefix<'a>(
    mechanism: &'a str,
    mtype: &str,
    default_domain: &'a str,
) -> (String, Option<u8>) {
    let without_type = mechanism
        .strip_prefix(mtype)
        .unwrap_or(mechanism);

    let (domain_part, prefix) = if let Some(slash) = without_type.rfind('/') {
        let p: Option<u8> = without_type[slash + 1..].parse().ok();
        (&without_type[..slash], p)
    } else {
        (without_type, None)
    };

    let domain = if domain_part.is_empty() || domain_part == ":" {
        default_domain.to_string()
    } else {
        domain_part.trim_start_matches(':').to_string()
    };

    (domain, prefix)
}

/// Check if sender_ip matches any A/AAAA record for domain.
async fn match_a_record(
    resolver: &Resolver,
    sender_ip: IpAddr,
    domain: &str,
) -> MechanismMatch {
    match resolver.inner.lookup_ip(domain).await {
        Ok(lookup) => {
            for addr in lookup.iter() {
                if addr == sender_ip {
                    return MechanismMatch::Match;
                }
            }
            MechanismMatch::NoMatch
        }
        Err(_) => MechanismMatch::NoMatch,
    }
}

/// Check if sender_ip matches any host pointed to by MX records.
async fn match_mx_record(
    resolver: &Resolver,
    sender_ip: IpAddr,
    domain: &str,
) -> MechanismMatch {
    let mx_result = resolver.lookup_mx(domain).await;
    for mx_host in &mx_result.answers {
        match resolver.inner.lookup_ip(mx_host.trim_end_matches('.')).await {
            Ok(lookup) => {
                for addr in lookup.iter() {
                    if addr == sender_ip {
                        return MechanismMatch::Match;
                    }
                }
            }
            Err(_) => continue,
        }
    }
    MechanismMatch::NoMatch
}

/// Check if an IPv4 address is within an ip4 CIDR string.
fn cidr_match_v4(ip: std::net::Ipv4Addr, cidr: &str) -> bool {
    if let Some(slash) = cidr.find('/') {
        let addr_str = &cidr[..slash];
        let prefix: u8 = cidr[slash + 1..].parse().unwrap_or(32);
        let network: std::net::Ipv4Addr = match addr_str.parse() {
            Ok(a) => a,
            Err(_) => return false,
        };
        let mask = if prefix == 0 {
            0u32
        } else if prefix >= 32 {
            u32::MAX
        } else {
            !((1u32 << (32 - prefix)) - 1)
        };
        (u32::from(ip) & mask) == (u32::from(network) & mask)
    } else {
        ip.to_string() == cidr
    }
}

/// Check if an IPv6 address is within an ip6 CIDR string.
fn cidr_match_v6(ip: std::net::Ipv6Addr, cidr: &str) -> bool {
    if let Some(slash) = cidr.find('/') {
        let addr_str = &cidr[..slash];
        let prefix: u8 = cidr[slash + 1..].parse().unwrap_or(128);
        let network: std::net::Ipv6Addr = match addr_str.parse() {
            Ok(a) => a,
            Err(_) => return false,
        };
        let ip_bits = u128::from(ip);
        let net_bits = u128::from(network);
        let mask = if prefix == 0 {
            0u128
        } else if prefix >= 128 {
            u128::MAX
        } else {
            !((1u128 << (128 - prefix)) - 1)
        };
        (ip_bits & mask) == (net_bits & mask)
    } else {
        ip.to_string() == cidr
    }
}
