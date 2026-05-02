CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE EXTENSION IF NOT EXISTS citext;

-- ── TENANTS ──────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS tenants (
  id                    UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  name                  TEXT        NOT NULL,
  slug                  TEXT        NOT NULL,
  owner_user_id         UUID        NOT NULL,
  status                TEXT        NOT NULL DEFAULT 'active',
  razorpay_customer_id  TEXT,
  billing_email         CITEXT      NOT NULL,
  country               CHAR(2)     NOT NULL DEFAULT 'IN',
  timezone              TEXT        NOT NULL DEFAULT 'Asia/Kolkata',
  created_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at            TIMESTAMPTZ,
  CONSTRAINT tenants_slug_key UNIQUE (slug),
  CONSTRAINT tenants_razorpay_key UNIQUE (razorpay_customer_id),
  CONSTRAINT tenants_status_check CHECK (
    status IN ('active','suspended','cancelled')
  )
);

-- ── TENANT MEMBERS ───────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS tenant_members (
  id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id   UUID        NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
  user_id     UUID        NOT NULL,
  role        TEXT        NOT NULL,
  invited_by  UUID,
  joined_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
  CONSTRAINT tenant_members_role_check CHECK (
    role IN ('owner','admin','analyst','viewer')
  ),
  CONSTRAINT tenant_members_unique UNIQUE (tenant_id, user_id)
);

-- ── TENANT INVITES ────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS tenant_invites (
  id           UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id    UUID        NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
  email        CITEXT      NOT NULL,
  role         TEXT        NOT NULL,
  token_hash   TEXT        NOT NULL,
  invited_by   UUID        NOT NULL,
  expires_at   TIMESTAMPTZ NOT NULL,
  accepted     BOOLEAN     NOT NULL DEFAULT false,
  accepted_at  TIMESTAMPTZ,
  created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
  CONSTRAINT tenant_invites_token_key UNIQUE (token_hash),
  CONSTRAINT tenant_invites_role_check CHECK (
    role IN ('admin','analyst','viewer')
  )
);

-- ── TENANT SETTINGS ───────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS tenant_settings (
  id                         UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id                  UUID        NOT NULL REFERENCES tenants(id)
                                           ON DELETE CASCADE,
  max_upload_size_mb         INTEGER     NOT NULL DEFAULT 25,
  sandbox_url_enabled        BOOLEAN     NOT NULL DEFAULT true,
  sandbox_file_enabled       BOOLEAN     NOT NULL DEFAULT true,
  sandbox_dynamic_enabled    BOOLEAN     NOT NULL DEFAULT true,
  auto_report_on_critical    BOOLEAN     NOT NULL DEFAULT true,
  retention_days             INTEGER     NOT NULL DEFAULT 90,
  notify_email               CITEXT,
  notify_slack_webhook       TEXT,
  notify_custom_webhook      TEXT,
  notify_on_critical         BOOLEAN     NOT NULL DEFAULT true,
  notify_on_high             BOOLEAN     NOT NULL DEFAULT false,
  notify_on_medium           BOOLEAN     NOT NULL DEFAULT false,
  updated_at                 TIMESTAMPTZ NOT NULL DEFAULT now(),
  CONSTRAINT tenant_settings_tenant_unique UNIQUE (tenant_id)
);

-- ── RAZORPAY SUBSCRIPTIONS ────────────────────────────────────────
CREATE TABLE IF NOT EXISTS subscriptions (
  id                       UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id                UUID        NOT NULL REFERENCES tenants(id)
                                         ON DELETE CASCADE,
  razorpay_subscription_id TEXT,
  razorpay_plan_id         TEXT,
  status                   TEXT        NOT NULL DEFAULT 'created',
  current_period_start     TIMESTAMPTZ,
  current_period_end       TIMESTAMPTZ,
  created_at               TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at               TIMESTAMPTZ NOT NULL DEFAULT now(),
  CONSTRAINT subscriptions_rz_id_key UNIQUE (razorpay_subscription_id),
  CONSTRAINT subscriptions_status_check CHECK (
    status IN (
      'created','authenticated','active','pending',
      'halted','cancelled','completed','expired'
    )
  )
);

-- ── RAZORPAY WEBHOOK EVENTS ───────────────────────────────────────
-- Idempotency: event_id is unique — Razorpay delivers the same event
-- multiple times on failure. ON CONFLICT DO NOTHING prevents double-processing.
CREATE TABLE IF NOT EXISTS razorpay_events (
  id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  event_id        TEXT        NOT NULL,
  event_type      TEXT        NOT NULL,
  tenant_id       UUID        REFERENCES tenants(id),
  payload         JSONB       NOT NULL,
  processed       BOOLEAN     NOT NULL DEFAULT false,
  processed_at    TIMESTAMPTZ,
  error           TEXT,
  received_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
  CONSTRAINT razorpay_events_event_id_key UNIQUE (event_id)
);

-- ── INDEXES ───────────────────────────────────────────────────────
CREATE INDEX IF NOT EXISTS idx_tenants_owner
  ON tenants(owner_user_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_tenants_status
  ON tenants(status) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_tenant_members_tid
  ON tenant_members(tenant_id);
CREATE INDEX IF NOT EXISTS idx_tenant_members_uid
  ON tenant_members(user_id);
CREATE INDEX IF NOT EXISTS idx_invites_email
  ON tenant_invites(email) WHERE accepted = false;
CREATE INDEX IF NOT EXISTS idx_invites_tenant
  ON tenant_invites(tenant_id) WHERE accepted = false;
CREATE INDEX IF NOT EXISTS idx_subscriptions_tenant
  ON subscriptions(tenant_id);
CREATE INDEX IF NOT EXISTS idx_rz_events_unprocessed
  ON razorpay_events(received_at) WHERE processed = false;
CREATE INDEX IF NOT EXISTS idx_rz_events_tenant
  ON razorpay_events(tenant_id, received_at DESC);
