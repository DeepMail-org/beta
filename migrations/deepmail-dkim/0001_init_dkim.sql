CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- ── DKIM ANALYSES ────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS dkim_analyses (
  id                UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  email_id          UUID        NOT NULL,
  tenant_id         UUID        NOT NULL,
  raw_dkim_headers  TEXT[]      NOT NULL,
  overall_verdict   TEXT        NOT NULL,
  replay_confidence REAL        NOT NULL,
  signals_json      JSONB       NOT NULL DEFAULT '{}',
  analyzed_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
  created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
  CONSTRAINT dkim_analyses_email_id_key UNIQUE (email_id),
  CONSTRAINT dkim_verdict_check CHECK (
    overall_verdict IN ('NONE','LOW','MEDIUM','HIGH','CRITICAL')
  ),
  CONSTRAINT dkim_confidence_check CHECK (
    replay_confidence BETWEEN 0.0 AND 1.0
  )
);

-- ── DKIM SIGNATURES ──────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS dkim_signatures (
  id                  UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  analysis_id         UUID        NOT NULL
                        REFERENCES dkim_analyses(id) ON DELETE CASCADE,
  selector            TEXT        NOT NULL,
  domain              TEXT        NOT NULL,
  algorithm           TEXT        NOT NULL,
  canonicalization    TEXT        NOT NULL,
  signed_headers      TEXT[]      NOT NULL,
  body_hash_claimed   TEXT        NOT NULL,
  body_hash_computed  TEXT        NOT NULL,
  body_hash_match     BOOLEAN     NOT NULL,
  signature_timestamp TIMESTAMPTZ,
  signature_valid     BOOLEAN     NOT NULL DEFAULT false,
  key_found_in_dns    BOOLEAN     NOT NULL DEFAULT false,
  created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ── DKIM KEY CACHE ───────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS dkim_key_cache (
  id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  selector    TEXT        NOT NULL,
  domain      TEXT        NOT NULL,
  public_key_pem TEXT,
  algorithm   TEXT,
  fetched_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
  ttl_seconds INTEGER     NOT NULL DEFAULT 300,
  CONSTRAINT dkim_key_cache_selector_domain_key UNIQUE (selector, domain)
);

-- ── INDEXES ──────────────────────────────────────────────────────
CREATE INDEX IF NOT EXISTS idx_dkim_analyses_email_id
  ON dkim_analyses(email_id);
CREATE INDEX IF NOT EXISTS idx_dkim_analyses_tenant_id
  ON dkim_analyses(tenant_id);
CREATE INDEX IF NOT EXISTS idx_dkim_analyses_analyzed_at
  ON dkim_analyses(analyzed_at DESC);
CREATE INDEX IF NOT EXISTS idx_dkim_signatures_analysis_id
  ON dkim_signatures(analysis_id);
CREATE INDEX IF NOT EXISTS idx_dkim_signatures_domain_selector
  ON dkim_signatures(domain, selector);
CREATE INDEX IF NOT EXISTS idx_dkim_key_cache_selector_domain
  ON dkim_key_cache(selector, domain);
