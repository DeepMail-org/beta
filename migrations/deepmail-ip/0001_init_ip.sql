-- DeepMail IP Reputation Service — initial schema
-- =================================================

-- ip_reputation: per-IP composite reputation record
CREATE TABLE ip_reputation (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ip_address      INET NOT NULL UNIQUE,
    threat_score    REAL NOT NULL DEFAULT 0.0
                    CHECK (threat_score BETWEEN 0.0 AND 1.0),
    threat_verdict  TEXT NOT NULL DEFAULT 'CLEAN'
                    CHECK (threat_verdict IN ('CLEAN','LOW','MEDIUM','HIGH','CRITICAL')),
    feed_hits       TEXT[] NOT NULL DEFAULT '{}',
    abuse_score     INTEGER CHECK (abuse_score BETWEEN 0 AND 100),
    shodan_ports    INTEGER[] NOT NULL DEFAULT '{}',
    shodan_tags     TEXT[] NOT NULL DEFAULT '{}',
    shodan_vulns    TEXT[] NOT NULL DEFAULT '{}',
    shodan_org      TEXT,
    pdns_hostnames  TEXT[] NOT NULL DEFAULT '{}',
    pdns_first_seen TIMESTAMPTZ,
    pdns_last_seen  TIMESTAMPTZ,
    bgp_prefix      CIDR,
    bgp_asn         INTEGER,
    bgp_holder      TEXT,
    bgp_announced   BOOLEAN NOT NULL DEFAULT true,
    is_bogon        BOOLEAN NOT NULL DEFAULT false,
    sighting_count  INTEGER NOT NULL DEFAULT 1,
    first_seen      TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen       TIMESTAMPTZ NOT NULL DEFAULT now(),
    enriched_at     TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_ip_reputation_verdict   ON ip_reputation(threat_verdict);
CREATE INDEX idx_ip_reputation_last_seen ON ip_reputation(last_seen DESC);
CREATE INDEX idx_ip_reputation_score     ON ip_reputation(threat_score DESC);

-- ip_blocklist_feeds: metadata for each blocklist feed
CREATE TABLE ip_blocklist_feeds (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    feed_name       TEXT NOT NULL UNIQUE,
    feed_url        TEXT NOT NULL,
    threat_category TEXT NOT NULL,
    last_fetched_at TIMESTAMPTZ,
    entry_count     INTEGER NOT NULL DEFAULT 0,
    fetch_errors    INTEGER NOT NULL DEFAULT 0,
    is_active       BOOLEAN NOT NULL DEFAULT true,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ip_blocklist_entries: individual IPs/CIDRs from feeds
CREATE TABLE ip_blocklist_entries (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    feed_id         UUID NOT NULL REFERENCES ip_blocklist_feeds(id) ON DELETE CASCADE,
    ip_address      INET,
    cidr_range      CIDR,
    raw_value       TEXT NOT NULL,
    threat_score    REAL NOT NULL DEFAULT 0.5,
    first_seen      TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen       TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT ip_or_cidr_required CHECK (ip_address IS NOT NULL OR cidr_range IS NOT NULL)
);

CREATE INDEX idx_blocklist_entries_ip   ON ip_blocklist_entries(ip_address);
CREATE INDEX idx_blocklist_entries_cidr ON ip_blocklist_entries USING GIST(cidr_range inet_ops);
CREATE INDEX idx_blocklist_entries_feed ON ip_blocklist_entries(feed_id);
-- Unique constraints for upsert ON CONFLICT
CREATE UNIQUE INDEX uq_blocklist_entries_feed_ip
    ON ip_blocklist_entries(feed_id, ip_address) WHERE ip_address IS NOT NULL;
CREATE UNIQUE INDEX uq_blocklist_entries_feed_cidr
    ON ip_blocklist_entries(feed_id, cidr_range) WHERE cidr_range IS NOT NULL;

-- ip_shodan_cache: cached Shodan enrichment per IP
CREATE TABLE ip_shodan_cache (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ip_address      INET NOT NULL UNIQUE,
    ports           INTEGER[] NOT NULL DEFAULT '{}',
    tags            TEXT[] NOT NULL DEFAULT '{}',
    vulns           TEXT[] NOT NULL DEFAULT '{}',
    org             TEXT,
    isp             TEXT,
    os              TEXT,
    hostnames       TEXT[] NOT NULL DEFAULT '{}',
    country_code    TEXT,
    asn             TEXT,
    last_update     TIMESTAMPTZ,
    fetched_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at      TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_shodan_cache_expires ON ip_shodan_cache(expires_at);

-- ip_pdns_records: passive DNS correlation per IP
CREATE TABLE ip_pdns_records (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ip_address      INET NOT NULL,
    hostname        TEXT NOT NULL,
    record_type     TEXT NOT NULL DEFAULT 'A',
    first_seen      TIMESTAMPTZ,
    last_seen       TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(ip_address, hostname)
);

CREATE INDEX idx_pdns_records_ip ON ip_pdns_records(ip_address);

-- email_ip_analyses: per-email IP analysis results
CREATE TABLE email_ip_analyses (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email_id        UUID NOT NULL UNIQUE,
    tenant_id       UUID NOT NULL,
    analyzed_ips    INET[] NOT NULL DEFAULT '{}',
    max_threat_score REAL NOT NULL DEFAULT 0.0,
    max_verdict     TEXT NOT NULL DEFAULT 'CLEAN',
    summary_json    JSONB NOT NULL DEFAULT '{}',
    analyzed_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_email_ip_tenant  ON email_ip_analyses(tenant_id);
CREATE INDEX idx_email_ip_date    ON email_ip_analyses(analyzed_at DESC);

-- Seed all 10 blocklist feeds
INSERT INTO ip_blocklist_feeds (feed_name, feed_url, threat_category) VALUES
    ('feodo_tracker',       'https://feodotracker.abuse.ch/downloads/ipblocklist.txt',          'c2'),
    ('spamhaus_drop',       'https://www.spamhaus.org/drop/drop.txt',                           'spam'),
    ('spamhaus_edrop',      'https://www.spamhaus.org/drop/edrop.txt',                          'spam'),
    ('cins_army',           'https://cinsscore.com/list/ci-badguys.txt',                        'scanner'),
    ('emerging_threats',    'https://rules.emergingthreats.net/blockrules/compromised-ips.txt', 'botnet'),
    ('blocklist_de',        'https://lists.blocklist.de/lists/all.txt',                         'scanner'),
    ('dshield_top20',       'https://www.dshield.org/feeds/suspiciousdomains_Low.txt',          'scanner'),
    ('tor_exits',           'https://check.torproject.org/torbulkexitlist',                     'proxy'),
    ('brute_force_logins',  'https://lists.blocklist.de/lists/ssh.txt',                         'brute_force'),
    ('alienvault_otx',      'https://reputation.alienvault.com/reputation.data',                'botnet');
