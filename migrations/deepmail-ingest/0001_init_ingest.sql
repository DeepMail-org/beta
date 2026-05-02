CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- ── EMAILS ───────────────────────────────────────────────────────
-- Central email record. Created by ingest. Referenced by all other
-- services via email_id UUID (no FK — cross-service isolation).
CREATE TABLE IF NOT EXISTS emails (
  id                  UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id           UUID        NOT NULL,
  uploaded_by         UUID        NOT NULL,
  original_filename   TEXT        NOT NULL,
  quarantine_name     TEXT        NOT NULL,
  s3_bucket           TEXT        NOT NULL,
  s3_key              TEXT        NOT NULL,
  sha256_hash         TEXT        NOT NULL,
  md5_hash            TEXT        NOT NULL,
  file_size_bytes     BIGINT      NOT NULL,
  file_extension      TEXT        NOT NULL,
  mime_type           TEXT        NOT NULL,
  magic_bytes_valid   BOOLEAN     NOT NULL DEFAULT false,
  status              TEXT        NOT NULL DEFAULT 'queued',
  rejection_reason    TEXT,
  nats_message_id     TEXT,
  uploaded_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
  completed_at        TIMESTAMPTZ,
  deleted_at          TIMESTAMPTZ,
  CONSTRAINT emails_quarantine_name_key UNIQUE (quarantine_name),
  CONSTRAINT emails_status_check CHECK (
    status IN (
      'queued','parsing','analyzing','completed','failed','rejected'
    )
  )
);

-- ── FILE VALIDATIONS ──────────────────────────────────────────────
-- One row per validation step per email.
-- Enables detailed rejection reason display in the UI.
CREATE TABLE IF NOT EXISTS file_validations (
  id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  email_id    UUID        NOT NULL REFERENCES emails(id) ON DELETE CASCADE,
  step        TEXT        NOT NULL,
  passed      BOOLEAN     NOT NULL,
  detail      TEXT,
  checked_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
  CONSTRAINT file_validations_step_check CHECK (
    step IN (
      'extension','size','magic_bytes','mime_type','path_traversal'
    )
  )
);

-- ── JOB PROGRESS ──────────────────────────────────────────────────
-- Tracks pipeline stage status for each email.
-- One row per (email_id, stage). All start as 'pending'.
-- Each downstream service updates its own row via its own DB.
-- This table lives in deepmail-ingest's DB as the source of truth
-- for overall pipeline progress, updated via NATS events.
CREATE TABLE IF NOT EXISTS job_progress (
  id               UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  email_id         UUID        NOT NULL REFERENCES emails(id) ON DELETE CASCADE,
  tenant_id        UUID        NOT NULL,
  stage            TEXT        NOT NULL,
  status           TEXT        NOT NULL DEFAULT 'pending',
  worker_id        TEXT,
  nats_message_id  TEXT,
  retry_count      INTEGER     NOT NULL DEFAULT 0,
  error_message    TEXT,
  started_at       TIMESTAMPTZ,
  completed_at     TIMESTAMPTZ,
  duration_ms      INTEGER,
  created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
  CONSTRAINT job_progress_unique UNIQUE (email_id, stage),
  CONSTRAINT job_progress_status_check CHECK (
    status IN ('pending','running','completed','failed','skipped')
  ),
  CONSTRAINT job_progress_stage_check CHECK (
    stage IN (
      'parse','header','dkim','geo','ip','intel','ioc',
      'body','homograph','sandbox_url','sandbox_file',
      'sandbox_dynamic','hashdb','scoring','ml','graph','report'
    )
  )
);

-- ── INDEXES ───────────────────────────────────────────────────────
CREATE INDEX IF NOT EXISTS idx_emails_tenant
  ON emails(tenant_id, uploaded_at DESC)
  WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_emails_sha256
  ON emails(sha256_hash);

CREATE INDEX IF NOT EXISTS idx_emails_status
  ON emails(status)
  WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_emails_uploader
  ON emails(uploaded_by, uploaded_at DESC);

CREATE INDEX IF NOT EXISTS idx_job_progress_email
  ON job_progress(email_id);

CREATE INDEX IF NOT EXISTS idx_job_progress_status
  ON job_progress(status)
  WHERE status IN ('pending','running');

CREATE INDEX IF NOT EXISTS idx_job_progress_tenant
  ON job_progress(tenant_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_file_val_email
  ON file_validations(email_id);
