-- ─────────────────────────────────────────────────────────────────
-- deepmail_auth  ·  Authentication & Identity
-- PostgreSQL 16 · pgcrypto · citext
-- ─────────────────────────────────────────────────────────────────

CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE EXTENSION IF NOT EXISTS citext;

-- ── USERS ────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS users (
  id               UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  username         TEXT        NOT NULL,
  email            CITEXT      NOT NULL,
  password_hash    TEXT        NOT NULL,
  is_active        BOOLEAN     NOT NULL DEFAULT false,
  email_verified   BOOLEAN     NOT NULL DEFAULT false,
  is_flagged       BOOLEAN     NOT NULL DEFAULT false,
  flagged_reason   TEXT,
  created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
  last_login_at    TIMESTAMPTZ,
  deleted_at       TIMESTAMPTZ,
  CONSTRAINT users_username_key UNIQUE (username),
  CONSTRAINT users_email_key    UNIQUE (email)
);

-- ── OTP CODES ────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS otp_codes (
  id             UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id        UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  code_hash      TEXT        NOT NULL,
  otp_type       TEXT        NOT NULL,
  expires_at     TIMESTAMPTZ NOT NULL,
  attempts       INTEGER     NOT NULL DEFAULT 0,
  max_attempts   INTEGER     NOT NULL DEFAULT 5,
  used           BOOLEAN     NOT NULL DEFAULT false,
  used_at        TIMESTAMPTZ,
  ip_address     INET,
  user_agent     TEXT,
  created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
  CONSTRAINT otp_type_check CHECK (
    otp_type IN ('login','registration','password_reset','email_change')
  )
);

-- ── REFRESH TOKENS ───────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS refresh_tokens (
  id             UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id        UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  token_hash     TEXT        NOT NULL,
  device_info    TEXT,
  ip_address     INET,
  expires_at     TIMESTAMPTZ NOT NULL,
  revoked        BOOLEAN     NOT NULL DEFAULT false,
  revoked_at     TIMESTAMPTZ,
  revoke_reason  TEXT,
  created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
  last_used_at   TIMESTAMPTZ,
  CONSTRAINT refresh_tokens_token_hash_key UNIQUE (token_hash)
);

-- ── API KEYS ─────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS api_keys (
  id           UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id      UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  tenant_id    UUID        NOT NULL,
  name         TEXT        NOT NULL,
  key_prefix   TEXT        NOT NULL,
  key_hash     TEXT        NOT NULL,
  scopes       TEXT[]      NOT NULL DEFAULT '{}',
  last_used_at TIMESTAMPTZ,
  expires_at   TIMESTAMPTZ,
  revoked      BOOLEAN     NOT NULL DEFAULT false,
  revoked_at   TIMESTAMPTZ,
  created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
  CONSTRAINT api_keys_key_hash_key UNIQUE (key_hash)
);

-- ── LOGIN ATTEMPTS ───────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS login_attempts (
  id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  email           CITEXT,
  user_id         UUID        REFERENCES users(id) ON DELETE SET NULL,
  ip_address      INET        NOT NULL,
  user_agent      TEXT,
  success         BOOLEAN     NOT NULL,
  failure_reason  TEXT,
  otp_stage       BOOLEAN     NOT NULL DEFAULT false,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ── IP LOCKOUTS ──────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS ip_lockouts (
  id             UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  ip_address     INET        NOT NULL,
  locked_until   TIMESTAMPTZ NOT NULL,
  attempt_count  INTEGER     NOT NULL DEFAULT 0,
  lockout_count  INTEGER     NOT NULL DEFAULT 0,
  created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
  CONSTRAINT ip_lockouts_ip_key UNIQUE (ip_address)
);

-- ── AUDIT LOGS ───────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS audit_logs (
  id             UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id        UUID        REFERENCES users(id) ON DELETE SET NULL,
  action         TEXT        NOT NULL,
  resource_type  TEXT,
  resource_id    UUID,
  ip_address     INET,
  user_agent     TEXT,
  metadata       JSONB       NOT NULL DEFAULT '{}',
  created_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ── INDEXES ──────────────────────────────────────────────────────
CREATE INDEX IF NOT EXISTS idx_users_email
  ON users(email) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_users_username
  ON users(username) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_otp_user_type
  ON otp_codes(user_id, otp_type) WHERE used = false;
CREATE INDEX IF NOT EXISTS idx_otp_expires
  ON otp_codes(expires_at) WHERE used = false;
CREATE INDEX IF NOT EXISTS idx_refresh_user
  ON refresh_tokens(user_id) WHERE revoked = false;
CREATE INDEX IF NOT EXISTS idx_refresh_expires
  ON refresh_tokens(expires_at) WHERE revoked = false;
CREATE INDEX IF NOT EXISTS idx_api_keys_user
  ON api_keys(user_id) WHERE revoked = false;
CREATE INDEX IF NOT EXISTS idx_api_keys_tenant
  ON api_keys(tenant_id) WHERE revoked = false;
CREATE INDEX IF NOT EXISTS idx_login_ip_time
  ON login_attempts(ip_address, created_at);
CREATE INDEX IF NOT EXISTS idx_login_email_time
  ON login_attempts(email, created_at);
CREATE INDEX IF NOT EXISTS idx_audit_user_time
  ON audit_logs(user_id, created_at);
CREATE INDEX IF NOT EXISTS idx_audit_action
  ON audit_logs(action, created_at);
CREATE INDEX IF NOT EXISTS idx_lockouts_ip
  ON ip_lockouts(ip_address);
