-- deepmail-sandbox-url schema: URL sandbox jobs + reports.

CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- ============================================================================
-- URL sandbox jobs
-- ============================================================================
CREATE TABLE url_sandbox_jobs (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email_id        UUID NOT NULL,
    tenant_id       UUID NOT NULL,
    url             TEXT NOT NULL,
    url_type        TEXT NOT NULL DEFAULT 'href'
                    CHECK (url_type IN ('href','src','action','meta_refresh',
                                        'css_url','data_uri','base64_decoded',
                                        'plain_text','qr_candidate','qr_decoded')),
    status          TEXT NOT NULL DEFAULT 'pending'
                    CHECK (status IN ('pending','running','completed',
                                      'failed','timeout')),
    attempt_count   INTEGER NOT NULL DEFAULT 0,
    container_id    TEXT,
    started_at      TIMESTAMPTZ,
    completed_at    TIMESTAMPTZ,
    error_message   TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_sandbox_jobs_email    ON url_sandbox_jobs(email_id);
CREATE INDEX idx_sandbox_jobs_tenant   ON url_sandbox_jobs(tenant_id);
CREATE INDEX idx_sandbox_jobs_status   ON url_sandbox_jobs(status);
CREATE INDEX idx_sandbox_jobs_created  ON url_sandbox_jobs(created_at DESC);
CREATE INDEX idx_sandbox_jobs_url      ON url_sandbox_jobs(url);

-- ============================================================================
-- URL sandbox reports
-- ============================================================================
CREATE TABLE url_sandbox_reports (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_id              UUID NOT NULL REFERENCES url_sandbox_jobs(id) ON DELETE CASCADE,
    email_id            UUID NOT NULL,
    tenant_id           UUID NOT NULL,
    original_url        TEXT NOT NULL,
    final_url           TEXT,
    redirect_count      INTEGER NOT NULL DEFAULT 0,
    redirect_chain      JSONB NOT NULL DEFAULT '[]',
    page_title          TEXT,
    network_requests    JSONB NOT NULL DEFAULT '[]',
    cookies             JSONB NOT NULL DEFAULT '[]',
    external_scripts    JSONB NOT NULL DEFAULT '[]',
    iframes             JSONB NOT NULL DEFAULT '[]',
    has_password_field  BOOLEAN NOT NULL DEFAULT false,
    has_email_field     BOOLEAN NOT NULL DEFAULT false,
    has_login_form      BOOLEAN NOT NULL DEFAULT false,
    has_download_trigger BOOLEAN NOT NULL DEFAULT false,
    js_dialogs          JSONB NOT NULL DEFAULT '[]',
    qr_decoded_urls     TEXT[] NOT NULL DEFAULT '{}',
    screenshot_s3_key   TEXT,
    threat_class        TEXT NOT NULL DEFAULT 'benign'
                        CHECK (threat_class IN ('benign','credential_harvesting',
                                                'phishing','malware_distribution',
                                                'c2_panel','suspicious')),
    threat_score        REAL NOT NULL DEFAULT 0.0
                        CHECK (threat_score BETWEEN 0.0 AND 1.0),
    analysis_notes      TEXT[] NOT NULL DEFAULT '{}',
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(job_id)
);

CREATE INDEX idx_sandbox_reports_email   ON url_sandbox_reports(email_id);
CREATE INDEX idx_sandbox_reports_tenant  ON url_sandbox_reports(tenant_id);
CREATE INDEX idx_sandbox_reports_class   ON url_sandbox_reports(threat_class);
CREATE INDEX idx_sandbox_reports_score   ON url_sandbox_reports(threat_score DESC);
