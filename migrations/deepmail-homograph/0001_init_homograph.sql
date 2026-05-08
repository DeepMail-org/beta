-- deepmail-homograph schema: homograph analyses + domain scores.

CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- ============================================================================
-- Per-email homograph analysis summary
-- ============================================================================
CREATE TABLE homograph_analyses (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email_id          UUID NOT NULL,
    tenant_id         UUID NOT NULL,
    domains_checked   INTEGER NOT NULL DEFAULT 0,
    high_risk_count   INTEGER NOT NULL DEFAULT 0,
    overall_risk      TEXT NOT NULL DEFAULT 'NONE'
                      CHECK (overall_risk IN ('NONE','LOW','MEDIUM','HIGH','CRITICAL')),
    analyzed_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(email_id)
);

CREATE INDEX idx_homograph_analyses_tenant_at ON homograph_analyses(tenant_id, analyzed_at DESC);
CREATE INDEX idx_homograph_analyses_risk ON homograph_analyses(overall_risk);

-- ============================================================================
-- Per-domain scoring results
-- ============================================================================
CREATE TABLE domain_scores (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    analysis_id       UUID NOT NULL REFERENCES homograph_analyses(id) ON DELETE CASCADE,
    domain            TEXT NOT NULL,
    decoded_domain    TEXT NOT NULL,
    skeleton          TEXT NOT NULL,
    best_brand_match  TEXT NOT NULL,
    raw_similarity    REAL NOT NULL,
    final_score       REAL NOT NULL CHECK (final_score BETWEEN 0.0 AND 1.0),
    edit_distance     INTEGER NOT NULL,
    mixed_script      BOOLEAN NOT NULL DEFAULT false,
    punycode_abuse    BOOLEAN NOT NULL DEFAULT false,
    risk_level        TEXT NOT NULL DEFAULT 'NONE'
                      CHECK (risk_level IN ('NONE','LOW','MEDIUM','HIGH','CRITICAL')),
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_domain_scores_analysis ON domain_scores(analysis_id);
CREATE INDEX idx_domain_scores_domain ON domain_scores(domain);
CREATE INDEX idx_domain_scores_risk ON domain_scores(risk_level);
CREATE INDEX idx_domain_scores_brand ON domain_scores(best_brand_match);
