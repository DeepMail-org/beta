CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE EXTENSION IF NOT EXISTS citext;

-- ── HEADER ANALYSES ───────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS header_analyses (
  id                      UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  email_id                UUID        NOT NULL,
  tenant_id               UUID        NOT NULL,
  overall_risk_score      REAL        NOT NULL DEFAULT 0.0,
  spf_result              TEXT,
  spf_domain              TEXT,
  spf_sender_ip           INET,
  dkim_result             TEXT,
  dkim_domain             TEXT,
  dkim_selector           TEXT,
  dmarc_result            TEXT,
  dmarc_policy            TEXT,
  dmarc_domain            TEXT,
  return_path_domain      TEXT,
  return_path_mismatch    BOOLEAN     NOT NULL DEFAULT false,
  reply_to_domain         TEXT,
  reply_to_mismatch       BOOLEAN     NOT NULL DEFAULT false,
  from_domain             TEXT,
  message_id_valid        BOOLEAN,
  message_id_domain       TEXT,
  message_id_domain_match BOOLEAN,
  message_id_fake_pattern BOOLEAN     NOT NULL DEFAULT false,
  x_mailer                TEXT,
  x_mailer_risk           TEXT,
  x_mailer_known_kit      TEXT,
  sender_spoofing_likely  BOOLEAN     NOT NULL DEFAULT false,
  analyzed_at             TIMESTAMPTZ NOT NULL DEFAULT now(),
  CONSTRAINT header_analyses_email_id_key UNIQUE (email_id),
  CONSTRAINT header_spf_check CHECK (
    spf_result IS NULL OR spf_result IN (
      'pass','fail','softfail','neutral','none','permerror','temperror'
    )
  ),
  CONSTRAINT header_dkim_check CHECK (
    dkim_result IS NULL OR dkim_result IN (
      'pass','fail','none','policy','neutral','temperror','permerror'
    )
  ),
  CONSTRAINT header_dmarc_check CHECK (
    dmarc_result IS NULL OR dmarc_result IN (
      'pass','fail','none','temperror','permerror'
    )
  ),
  CONSTRAINT header_xmailer_risk_check CHECK (
    x_mailer_risk IS NULL OR x_mailer_risk IN (
      'none','low','medium','high','critical'
    )
  )
);

-- ── HEADER FINDINGS ───────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS header_findings (
  id           UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  email_id     UUID        NOT NULL,
  analysis_id  UUID        NOT NULL
                 REFERENCES header_analyses(id) ON DELETE CASCADE,
  check_name   TEXT        NOT NULL,
  severity     TEXT        NOT NULL,
  passed       BOOLEAN     NOT NULL,
  evidence     TEXT        NOT NULL,
  raw_value    TEXT,
  risk_weight  REAL        NOT NULL DEFAULT 0.0,
  created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
  CONSTRAINT hf_severity_check CHECK (
    severity IN ('info','low','medium','high','critical')
  ),
  CONSTRAINT hf_check_name_check CHECK (
    check_name IN (
      'spf','dkim','dmarc','return_path_mismatch',
      'reply_to_mismatch','message_id_fake','x_mailer_kit',
      'time_anomaly','sender_spoofing'
    )
  )
);

-- ── TIME ANOMALIES ────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS time_anomalies (
  id                   UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  email_id             UUID        NOT NULL,
  date_header_ts       TIMESTAMPTZ,
  earliest_received_ts TIMESTAMPTZ,
  latest_received_ts   TIMESTAMPTZ,
  future_dated         BOOLEAN     NOT NULL DEFAULT false,
  date_predates_recv   BOOLEAN     NOT NULL DEFAULT false,
  max_hop_delay_sec    INTEGER,
  suspicious_delay     BOOLEAN     NOT NULL DEFAULT false,
  total_transit_sec    INTEGER,
  hop_count            INTEGER,
  created_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
  CONSTRAINT time_anomalies_email_id_key UNIQUE (email_id)
);

-- ── DNS LOOKUPS ───────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS dns_lookups (
  id           UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  email_id     UUID        NOT NULL,
  record_type  TEXT        NOT NULL,
  queried_name TEXT        NOT NULL,
  purpose      TEXT        NOT NULL,
  answers      TEXT[]      NOT NULL DEFAULT '{}',
  ttl_seconds  INTEGER,
  nxdomain     BOOLEAN     NOT NULL DEFAULT false,
  error        TEXT,
  latency_ms   INTEGER,
  queried_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
  CONSTRAINT dns_purpose_check CHECK (
    purpose IN ('spf','dmarc','dkim','mx','ptr','a')
  ),
  CONSTRAINT dns_type_check CHECK (
    record_type IN ('TXT','MX','A','AAAA','PTR','CNAME')
  )
);

-- ── MTA FINGERPRINTS ──────────────────────────────────────────────
-- Seeded at startup via a migration; updateable via admin API later.
CREATE TABLE IF NOT EXISTS mta_fingerprints (
  id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  pattern     TEXT        NOT NULL UNIQUE,
  match_type  TEXT        NOT NULL DEFAULT 'contains',
  kit_name    TEXT        NOT NULL,
  risk_level  TEXT        NOT NULL,
  created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
  CONSTRAINT mta_match_type_check CHECK (
    match_type IN ('exact','contains','regex')
  ),
  CONSTRAINT mta_risk_check CHECK (
    risk_level IN ('none','low','medium','high','critical')
  )
);

-- Seed known malicious MTA patterns
INSERT INTO mta_fingerprints (pattern, match_type, kit_name, risk_level)
VALUES
  ('exploit',            'contains', 'Generic exploit kit',    'critical'),
  ('inject',             'contains', 'Header injection kit',   'critical'),
  ('spamtool',           'contains', 'Generic spam tool',      'critical'),
  ('bulk mailer',        'contains', 'Bulk mailer kit',        'high'),
  ('mass mailer',        'contains', 'Mass mailer kit',        'high'),
  ('advanced mass',      'contains', 'Advanced Mass Sender',   'high'),
  ('sendinblue',         'contains', 'SendInBlue (abused)',    'medium'),
  ('phpmailer',          'contains', 'PHPMailer',              'medium'),
  ('swiftmailer',        'contains', 'SwiftMailer (abused)',   'medium'),
  ('zend_mail',          'contains', 'Zend_Mail',              'medium'),
  ('the bat!',           'contains', 'The Bat!',               'low'),
  ('microsoft cdo',      'contains', 'Microsoft CDO',          'low'),
  ('libmutt',            'contains', 'libmutt',                'low')
ON CONFLICT (pattern) DO NOTHING;

-- ── INDEXES ───────────────────────────────────────────────────────
CREATE INDEX IF NOT EXISTS idx_ha_email_id
  ON header_analyses(email_id);
CREATE INDEX IF NOT EXISTS idx_ha_tenant
  ON header_analyses(tenant_id, analyzed_at DESC);
CREATE INDEX IF NOT EXISTS idx_ha_spoofing
  ON header_analyses(sender_spoofing_likely)
  WHERE sender_spoofing_likely = true;
CREATE INDEX IF NOT EXISTS idx_hf_email_id
  ON header_findings(email_id);
CREATE INDEX IF NOT EXISTS idx_hf_analysis_id
  ON header_findings(analysis_id);
CREATE INDEX IF NOT EXISTS idx_hf_severity
  ON header_findings(severity)
  WHERE passed = false;
CREATE INDEX IF NOT EXISTS idx_dns_email_id
  ON dns_lookups(email_id);
CREATE INDEX IF NOT EXISTS idx_time_email_id
  ON time_anomalies(email_id);
