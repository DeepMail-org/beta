use std::net::IpAddr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpType {
    Hosting,
    Residential,
    Mobile,
    Education,
    Government,
    Unknown,
}

impl IpType {
    pub fn as_str(&self) -> &'static str {
        match self {
            IpType::Hosting => "HOSTING",
            IpType::Residential => "RESIDENTIAL",
            IpType::Mobile => "MOBILE",
            IpType::Education => "EDUCATION",
            IpType::Government => "GOVERNMENT",
            IpType::Unknown => "UNKNOWN",
        }
    }
}

impl std::fmt::Display for IpType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

pub fn is_private(ip: IpAddr) -> bool {
    use ipnetwork::IpNetwork;
    use std::str::FromStr;

    let private_ranges = [
        "10.0.0.0/8",
        "172.16.0.0/12",
        "192.168.0.0/16",
        "127.0.0.0/8",
        "169.254.0.0/16",
        "::1/128",
        "fc00::/7",
        "fe80::/10",
    ];

    for range in &private_ranges {
        if let Ok(net) = IpNetwork::from_str(range) {
            if net.contains(ip) {
                return true;
            }
        }
    }
    false
}

pub fn classify(as_org: &str) -> IpType {
    let lower = as_org.to_lowercase();

    let hosting_keywords = [
        "hosting", "cloud", "datacenter", "data center", "server", "vps",
        "digital ocean", "amazon", "google", "microsoft", "ovh", "hetzner",
        "linode", "vultr", "contabo",
    ];
    if hosting_keywords.iter().any(|k| lower.contains(k)) {
        return IpType::Hosting;
    }

    let mobile_keywords = ["mobile", "cellular", "wireless", "lte"];
    if mobile_keywords.iter().any(|k| lower.contains(k)) {
        return IpType::Mobile;
    }

    let edu_keywords = ["university", "college", "edu", "academic", "institute of technology"];
    if edu_keywords.iter().any(|k| lower.contains(k)) {
        return IpType::Education;
    }

    let gov_keywords = ["government", "ministry", "federal", "state govt"];
    if gov_keywords.iter().any(|k| lower.contains(k)) {
        return IpType::Government;
    }

    IpType::Residential
}
