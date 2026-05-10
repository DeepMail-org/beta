-- deepmail-sandbox-dynamic: dynamic file analysis via CAPEv2

CREATE TABLE IF NOT EXISTS dynamic_analysis_jobs (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email_id          UUID NOT NULL,
    tenant_id         UUID NOT NULL,
    attachment_id     UUID NOT NULL,
    s3_key            TEXT NOT NULL,
    filename          TEXT NOT NULL,
    sha256_hash       TEXT,
    cape_task_id      INTEGER,
    status            TEXT NOT NULL DEFAULT 'pending'
                      CHECK (status IN ('pending','submitted','running',
                                        'completed','failed','timeout','skipped')),
    attempt_count     INTEGER NOT NULL DEFAULT 0,
    cape_unavailable  BOOLEAN NOT NULL DEFAULT false,
    submitted_at      TIMESTAMPTZ,
    completed_at      TIMESTAMPTZ,
    error_message     TEXT,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(attachment_id)
);

CREATE TABLE IF NOT EXISTS dynamic_analysis_reports (
    id                     UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_id                 UUID NOT NULL REFERENCES dynamic_analysis_jobs(id) ON DELETE CASCADE,
    email_id               UUID NOT NULL,
    tenant_id              UUID NOT NULL,
    attachment_id          UUID NOT NULL,
    filename               TEXT NOT NULL,
    sha256_hash            TEXT,
    malscore               REAL NOT NULL DEFAULT 0.0,
    cape_unavailable       BOOLEAN NOT NULL DEFAULT false,
    network_hosts          TEXT[] NOT NULL DEFAULT '{}',
    dns_requests           TEXT[] NOT NULL DEFAULT '{}',
    http_requests          JSONB NOT NULL DEFAULT '[]',
    smtp_activity          BOOLEAN NOT NULL DEFAULT false,
    processes_spawned      TEXT[] NOT NULL DEFAULT '{}',
    files_dropped          JSONB NOT NULL DEFAULT '[]',
    registry_mods          TEXT[] NOT NULL DEFAULT '{}',
    persistence_indicators TEXT[] NOT NULL DEFAULT '{}',
    c2_indicators          TEXT[] NOT NULL DEFAULT '{}',
    cape_signatures        JSONB NOT NULL DEFAULT '[]',
    dynamic_score          REAL NOT NULL DEFAULT 0.0
                           CHECK (dynamic_score BETWEEN 0.0 AND 1.0),
    dynamic_verdict        TEXT NOT NULL DEFAULT 'UNKNOWN'
                           CHECK (dynamic_verdict IN ('BENIGN','LOW_RISK','SUSPICIOUS',
                                                       'MALWARE','UNKNOWN')),
    analysis_notes         TEXT[] NOT NULL DEFAULT '{}',
    cape_report_s3_key     TEXT,
    analyzed_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at             TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(job_id)
);

-- Indexes on dynamic_analysis_jobs
CREATE INDEX IF NOT EXISTS idx_daj_email       ON dynamic_analysis_jobs(email_id);
CREATE INDEX IF NOT EXISTS idx_daj_tenant      ON dynamic_analysis_jobs(tenant_id);
CREATE INDEX IF NOT EXISTS idx_daj_status      ON dynamic_analysis_jobs(status);
CREATE INDEX IF NOT EXISTS idx_daj_cape_task   ON dynamic_analysis_jobs(cape_task_id);

-- Indexes on dynamic_analysis_reports
CREATE INDEX IF NOT EXISTS idx_dar_email       ON dynamic_analysis_reports(email_id);
CREATE INDEX IF NOT EXISTS idx_dar_tenant_time ON dynamic_analysis_reports(tenant_id, analyzed_at DESC);
CREATE INDEX IF NOT EXISTS idx_dar_verdict     ON dynamic_analysis_reports(dynamic_verdict);
CREATE INDEX IF NOT EXISTS idx_dar_score       ON dynamic_analysis_reports(dynamic_score DESC);
CREATE INDEX IF NOT EXISTS idx_dar_sha256      ON dynamic_analysis_reports(sha256_hash);
