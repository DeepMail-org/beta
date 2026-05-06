CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- ── PARSED EMAILS ────────────────────────────────────────────────
-- One row per successfully parsed email. Holds the envelope-level
-- fields extracted by mail-parser (RFC 5322 / MIME).
CREATE TABLE IF NOT EXISTS parsed_emails (
  id                  UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  email_id            UUID        NOT NULL UNIQUE,
  tenant_id           UUID        NOT NULL,
  message_id          TEXT,
  subject             TEXT,
  from_address        TEXT,
  to_addresses        TEXT[],
  cc_addresses        TEXT[],
  bcc_addresses       TEXT[],
  reply_to            TEXT,
  date_sent           TIMESTAMPTZ,
  body_text           TEXT,
  body_html           TEXT,
  attachment_count    INTEGER     NOT NULL DEFAULT 0,
  parsed_at           TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ── EMAIL HEADERS ────────────────────────────────────────────────
-- Every header from the root MIME part is stored as a separate row
-- so queries can filter/aggregate on arbitrary header names.
CREATE TABLE IF NOT EXISTS email_headers (
  id                  UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  parsed_email_id     UUID        NOT NULL REFERENCES parsed_emails(id) ON DELETE CASCADE,
  header_name         TEXT        NOT NULL,
  header_value        TEXT        NOT NULL,
  ordinal             INTEGER     NOT NULL DEFAULT 0
);

-- ── RECEIVED HOPS ────────────────────────────────────────────────
-- Each "Received:" header is decomposed into from / by / for / via
-- fields and an optional timestamp. Ordered by hop_index.
CREATE TABLE IF NOT EXISTS received_hops (
  id                  UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  parsed_email_id     UUID        NOT NULL REFERENCES parsed_emails(id) ON DELETE CASCADE,
  hop_index           INTEGER     NOT NULL,
  from_host           TEXT,
  by_host             TEXT,
  for_address         TEXT,
  via_protocol        TEXT,
  received_at         TIMESTAMPTZ,
  raw_value           TEXT        NOT NULL
);

-- ── ATTACHMENTS ──────────────────────────────────────────────────
-- Metadata for each attachment extracted from the parsed email.
-- The actual blob is stored in S3 under s3_key.
CREATE TABLE IF NOT EXISTS attachments (
  id                  UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  parsed_email_id     UUID        NOT NULL REFERENCES parsed_emails(id) ON DELETE CASCADE,
  email_id            UUID        NOT NULL,
  tenant_id           UUID        NOT NULL,
  filename            TEXT,
  content_type        TEXT,
  size_bytes          BIGINT      NOT NULL DEFAULT 0,
  sha256_hash         TEXT        NOT NULL,
  s3_bucket           TEXT        NOT NULL,
  s3_key              TEXT        NOT NULL,
  entropy             DOUBLE PRECISION,
  ordinal             INTEGER     NOT NULL DEFAULT 0,
  created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ── INDEXES ──────────────────────────────────────────────────────
CREATE INDEX IF NOT EXISTS idx_parsed_emails_tenant
  ON parsed_emails(tenant_id, parsed_at DESC);

CREATE INDEX IF NOT EXISTS idx_parsed_emails_email_id
  ON parsed_emails(email_id);

CREATE INDEX IF NOT EXISTS idx_email_headers_parsed
  ON email_headers(parsed_email_id);

CREATE INDEX IF NOT EXISTS idx_email_headers_name
  ON email_headers(header_name);

CREATE INDEX IF NOT EXISTS idx_received_hops_parsed
  ON received_hops(parsed_email_id, hop_index);

CREATE INDEX IF NOT EXISTS idx_attachments_parsed
  ON attachments(parsed_email_id);

CREATE INDEX IF NOT EXISTS idx_attachments_email
  ON attachments(email_id);

CREATE INDEX IF NOT EXISTS idx_attachments_sha256
  ON attachments(sha256_hash);
