-- deepmail-geo schema: IP geolocation, ASN, threat classification

CREATE TABLE email_geo_analyses (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email_id        UUID NOT NULL,
    tenant_id       UUID NOT NULL,
    hop_count       INTEGER NOT NULL DEFAULT 0,
    origin_ip       INET,
    origin_country  TEXT,
    origin_asn      INTEGER,
    overall_risk    TEXT NOT NULL CHECK (overall_risk IN ('NONE','LOW','MEDIUM','HIGH','CRITICAL')),
    risk_score      REAL NOT NULL CHECK (risk_score BETWEEN 0.0 AND 1.0),
    analyzed_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(email_id)
);

CREATE INDEX idx_geo_analyses_email    ON email_geo_analyses(email_id);
CREATE INDEX idx_geo_analyses_tenant   ON email_geo_analyses(tenant_id);
CREATE INDEX idx_geo_analyses_time     ON email_geo_analyses(analyzed_at DESC);
CREATE INDEX idx_geo_analyses_country  ON email_geo_analyses(origin_country);

CREATE TABLE hop_geo_details (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    analysis_id     UUID NOT NULL REFERENCES email_geo_analyses(id) ON DELETE CASCADE,
    hop_index       INTEGER NOT NULL,
    ip_address      INET NOT NULL,
    ip_type         TEXT NOT NULL CHECK (ip_type IN ('HOSTING','RESIDENTIAL','MOBILE','EDUCATION','GOVERNMENT','UNKNOWN')),
    country_code    TEXT,
    country_name    TEXT,
    city            TEXT,
    latitude        DOUBLE PRECISION,
    longitude       DOUBLE PRECISION,
    postal_code     TEXT,
    timezone        TEXT,
    asn             INTEGER,
    as_org          TEXT,
    network_cidr    TEXT,
    is_tor          BOOLEAN NOT NULL DEFAULT false,
    is_vpn          BOOLEAN NOT NULL DEFAULT false,
    is_botnet       BOOLEAN NOT NULL DEFAULT false,
    abuse_score     INTEGER,
    greynoise_class TEXT,
    ipinfo_raw      JSONB,
    abuseipdb_raw   JSONB,
    greynoise_raw   JSONB,
    enriched_at     TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_hop_details_analysis ON hop_geo_details(analysis_id);
CREATE INDEX idx_hop_details_ip       ON hop_geo_details(ip_address);
CREATE INDEX idx_hop_details_tor      ON hop_geo_details(is_tor);
CREATE INDEX idx_hop_details_vpn      ON hop_geo_details(is_vpn);
CREATE INDEX idx_hop_details_botnet   ON hop_geo_details(is_botnet);

CREATE TABLE geo_blocklist_feeds (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    feed_name       TEXT NOT NULL UNIQUE,
    feed_url        TEXT NOT NULL,
    last_fetched_at TIMESTAMPTZ,
    entry_count     INTEGER NOT NULL DEFAULT 0,
    is_active       BOOLEAN NOT NULL DEFAULT true,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE geo_blocklist_entries (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    feed_id         UUID NOT NULL REFERENCES geo_blocklist_feeds(id) ON DELETE CASCADE,
    ip_address      INET,
    cidr_range      CIDR,
    threat_type     TEXT NOT NULL,
    first_seen      TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen       TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT ip_or_cidr CHECK (ip_address IS NOT NULL OR cidr_range IS NOT NULL)
);

CREATE INDEX idx_blocklist_entries_ip   ON geo_blocklist_entries(ip_address);
CREATE INDEX idx_blocklist_entries_cidr ON geo_blocklist_entries USING GIST(cidr_range inet_ops);
CREATE INDEX idx_blocklist_entries_feed ON geo_blocklist_entries(feed_id);

INSERT INTO geo_blocklist_feeds (feed_name, feed_url, is_active) VALUES
    ('Feodo Tracker', 'https://feodotracker.abuse.ch/downloads/ipblocklist.txt', true),
    ('Spamhaus DROP', 'https://www.spamhaus.org/drop/drop.txt', true),
    ('CINS Army', 'https://cinsscore.com/list/ci-badguys.txt', true);
