-- DeepMail Intel Service — initial schema

CREATE TABLE enrichment_cache (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ioc_type        TEXT NOT NULL
                    CHECK (ioc_type IN ('ip','domain','url','hash')),
    ioc_value       TEXT NOT NULL,
    provider        TEXT NOT NULL
                    CHECK (provider IN ('virustotal','abuseipdb','greynoise',
                                        'ipinfo','shodan','otx')),
    result_json     JSONB NOT NULL,
    vt_score        REAL,
    abuse_score     INTEGER,
    pulse_count     INTEGER,
    fetched_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at      TIMESTAMPTZ NOT NULL,
    UNIQUE(ioc_type, ioc_value, provider)
);

CREATE INDEX idx_enrichment_cache_expires  ON enrichment_cache(expires_at);
CREATE INDEX idx_enrichment_cache_ioc      ON enrichment_cache(ioc_value);

CREATE TABLE email_intel_results (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email_id        UUID NOT NULL UNIQUE,
    tenant_id       UUID NOT NULL,
    iocs_analyzed   INTEGER NOT NULL DEFAULT 0,
    max_vt_score    REAL NOT NULL DEFAULT 0.0,
    malicious_iocs  INTEGER NOT NULL DEFAULT 0,
    provider_hits   TEXT[] NOT NULL DEFAULT '{}',
    summary_json    JSONB NOT NULL DEFAULT '{}',
    analyzed_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_email_intel_tenant  ON email_intel_results(tenant_id);
CREATE INDEX idx_email_intel_date    ON email_intel_results(analyzed_at DESC);

CREATE TABLE provider_telemetry (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider        TEXT NOT NULL,
    window_start    TIMESTAMPTZ NOT NULL,
    window_end      TIMESTAMPTZ NOT NULL,
    request_count   INTEGER NOT NULL DEFAULT 0,
    success_count   INTEGER NOT NULL DEFAULT 0,
    failure_count   INTEGER NOT NULL DEFAULT 0,
    cache_hit_count INTEGER NOT NULL DEFAULT 0,
    total_latency_ms BIGINT NOT NULL DEFAULT 0,
    quota_remaining INTEGER,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_telemetry_provider_window ON provider_telemetry(provider, window_start);
