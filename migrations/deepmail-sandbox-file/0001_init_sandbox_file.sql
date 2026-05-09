-- deepmail-sandbox-file: static file analysis sandbox
-- Tables: file_static_reports, yara_rules

CREATE TABLE IF NOT EXISTS file_static_reports (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email_id            UUID NOT NULL,
    tenant_id           UUID NOT NULL,
    attachment_id       UUID NOT NULL,
    s3_key              TEXT NOT NULL,
    filename            TEXT NOT NULL,
    file_size           BIGINT NOT NULL DEFAULT 0,
    mime_type           TEXT,
    file_magic          TEXT,
    sha256_hash         TEXT,
    md5_hash            TEXT,
    entropy             REAL NOT NULL DEFAULT 0.0,
    has_macros          BOOLEAN NOT NULL DEFAULT false,
    macro_count         INTEGER NOT NULL DEFAULT 0,
    has_vba             BOOLEAN NOT NULL DEFAULT false,
    vba_suspicious      BOOLEAN NOT NULL DEFAULT false,
    is_pe               BOOLEAN NOT NULL DEFAULT false,
    pe_is_signed        BOOLEAN NOT NULL DEFAULT false,
    pe_is_packed        BOOLEAN NOT NULL DEFAULT false,
    pe_suspicious_imports TEXT[] NOT NULL DEFAULT '{}',
    is_pdf              BOOLEAN NOT NULL DEFAULT false,
    pdf_has_js          BOOLEAN NOT NULL DEFAULT false,
    pdf_has_launch      BOOLEAN NOT NULL DEFAULT false,
    pdf_has_embedded    BOOLEAN NOT NULL DEFAULT false,
    pdf_is_encrypted    BOOLEAN NOT NULL DEFAULT false,
    embedded_files      JSONB NOT NULL DEFAULT '[]',
    yara_matches        TEXT[] NOT NULL DEFAULT '{}',
    suspicious_strings  TEXT[] NOT NULL DEFAULT '{}',
    exif_author         TEXT,
    exif_created        TIMESTAMPTZ,
    exif_modified       TIMESTAMPTZ,
    exif_software       TEXT,
    exif_raw            JSONB NOT NULL DEFAULT '{}',
    strings_count       INTEGER NOT NULL DEFAULT 0,
    tool_outputs        JSONB NOT NULL DEFAULT '{}',
    threat_score        REAL NOT NULL DEFAULT 0.0
                        CHECK (threat_score BETWEEN 0.0 AND 1.0),
    threat_verdict      TEXT NOT NULL DEFAULT 'CLEAN'
                        CHECK (threat_verdict IN ('CLEAN','SUSPICIOUS',
                                                  'MALICIOUS','PACKED','UNKNOWN')),
    analysis_notes      TEXT[] NOT NULL DEFAULT '{}',
    analyzed_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(attachment_id)
);

CREATE TABLE IF NOT EXISTS yara_rules (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_name           TEXT NOT NULL UNIQUE,
    rule_source         TEXT NOT NULL,
    is_active           BOOLEAN NOT NULL DEFAULT true,
    match_count         INTEGER NOT NULL DEFAULT 0,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Indexes on file_static_reports
CREATE INDEX IF NOT EXISTS idx_fsr_email_id      ON file_static_reports(email_id);
CREATE INDEX IF NOT EXISTS idx_fsr_tenant_time   ON file_static_reports(tenant_id, analyzed_at DESC);
CREATE INDEX IF NOT EXISTS idx_fsr_sha256        ON file_static_reports(sha256_hash);
CREATE INDEX IF NOT EXISTS idx_fsr_verdict        ON file_static_reports(threat_verdict);
CREATE INDEX IF NOT EXISTS idx_fsr_score          ON file_static_reports(threat_score DESC);
CREATE INDEX IF NOT EXISTS idx_fsr_macros         ON file_static_reports(has_macros);
CREATE INDEX IF NOT EXISTS idx_fsr_yara           ON file_static_reports USING gin(yara_matches);

-- Indexes on yara_rules
CREATE INDEX IF NOT EXISTS idx_yr_active          ON yara_rules(is_active);
