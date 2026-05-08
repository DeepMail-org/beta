-- deepmail-body schema: body analysis + extracted URLs + QR findings.

CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- ============================================================================
-- Per-email body analysis summary
-- ============================================================================
CREATE TABLE body_analyses (
    id                    UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email_id              UUID NOT NULL,
    tenant_id             UUID NOT NULL,
    plain_text_length     INTEGER NOT NULL DEFAULT 0,
    html_length           INTEGER NOT NULL DEFAULT 0,
    url_count             INTEGER NOT NULL DEFAULT 0,
    external_url_count    INTEGER NOT NULL DEFAULT 0,
    shortener_url_count   INTEGER NOT NULL DEFAULT 0,
    qr_code_count         INTEGER NOT NULL DEFAULT 0,
    base64_url_count      INTEGER NOT NULL DEFAULT 0,
    keyword_score         REAL NOT NULL DEFAULT 0.0,
    ml_score              REAL,
    final_phishing_score  REAL NOT NULL DEFAULT 0.0
                          CHECK (final_phishing_score BETWEEN 0.0 AND 1.0),
    has_obfuscation       BOOLEAN NOT NULL DEFAULT false,
    has_tracking_pixels   BOOLEAN NOT NULL DEFAULT false,
    has_c2_beacons        BOOLEAN NOT NULL DEFAULT false,
    obfuscation_types     TEXT[] NOT NULL DEFAULT '{}',
    urgency_score         REAL NOT NULL DEFAULT 0.0,
    verdict               TEXT NOT NULL DEFAULT 'CLEAN'
                          CHECK (verdict IN ('CLEAN','SUSPICIOUS','PHISHING','MALICIOUS')),
    analyzed_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(email_id)
);

CREATE INDEX idx_body_analyses_tenant_at ON body_analyses(tenant_id, analyzed_at DESC);
CREATE INDEX idx_body_analyses_verdict ON body_analyses(verdict);
CREATE INDEX idx_body_analyses_phishing ON body_analyses(final_phishing_score DESC);

-- ============================================================================
-- Extracted URLs from email body
-- ============================================================================
CREATE TABLE extracted_urls (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    analysis_id       UUID NOT NULL REFERENCES body_analyses(id) ON DELETE CASCADE,
    raw_url           TEXT NOT NULL,
    normalized_url    TEXT NOT NULL,
    url_type          TEXT NOT NULL
                      CHECK (url_type IN ('href','src','action','meta_refresh',
                                          'css_url','data_uri','base64_decoded',
                                          'plain_text','qr_candidate')),
    is_shortened      BOOLEAN NOT NULL DEFAULT false,
    shortener_domain  TEXT,
    is_external       BOOLEAN NOT NULL DEFAULT false,
    destination_domain TEXT,
    is_suspicious     BOOLEAN NOT NULL DEFAULT false,
    sent_to_sandbox   BOOLEAN NOT NULL DEFAULT false,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_extracted_urls_analysis ON extracted_urls(analysis_id);
CREATE INDEX idx_extracted_urls_normalized ON extracted_urls(normalized_url);
CREATE INDEX idx_extracted_urls_shortened ON extracted_urls(is_shortened);
CREATE INDEX idx_extracted_urls_suspicious ON extracted_urls(is_suspicious);

-- ============================================================================
-- QR code findings
-- ============================================================================
CREATE TABLE qr_code_findings (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    analysis_id       UUID NOT NULL REFERENCES body_analyses(id) ON DELETE CASCADE,
    image_src         TEXT NOT NULL,
    image_type        TEXT NOT NULL,
    width             INTEGER,
    height            INTEGER,
    alt_text          TEXT,
    sent_to_sandbox   BOOLEAN NOT NULL DEFAULT false,
    sandbox_job_id    UUID,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_qr_code_findings_analysis ON qr_code_findings(analysis_id);
