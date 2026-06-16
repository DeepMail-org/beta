-- ─────────────────────────────────────────────────────────────────
-- deepmail_auth  ·  2FA, Sessions, Password Reset
-- ─────────────────────────────────────────────────────────────────

-- ── 2FA ──────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS user_2fa (
  id                 UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id            UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  secret_encrypted   TEXT        NOT NULL,   -- AES-256-GCM encrypted TOTP secret
  backup_codes_encrypted TEXT[]  NOT NULL,   -- 10 encrypted backup codes (alphanumeric 10-char)
  enabled            BOOLEAN     NOT NULL DEFAULT false,
  verified_at        TIMESTAMPTZ,
  created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
  CONSTRAINT user_2fa_user_id_key UNIQUE (user_id)
);

-- ── USER SESSIONS ───────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS user_sessions (
  id                 UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id            UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  session_token_hash TEXT        NOT NULL,   -- SHA-256 of session token
  device_info        TEXT,
  ip_address         INET,
  user_agent         TEXT,
  expires_at         TIMESTAMPTZ NOT NULL,
  revoked            BOOLEAN     NOT NULL DEFAULT false,
  revoked_at         TIMESTAMPTZ,
  created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
  last_activity_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Index for session cleanup and lookup
CREATE INDEX IF NOT EXISTS idx_sessions_user_active
  ON user_sessions(user_id) WHERE revoked = false;
CREATE INDEX IF NOT EXISTS idx_sessions_expires
  ON user_sessions(expires_at) WHERE revoked = false;
CREATE INDEX IF NOT EXISTS idx_sessions_token_hash
  ON user_sessions(session_token_hash);

-- ── PASSWORD RESET TOKENS ───────────────────────────────────────
CREATE TABLE IF NOT EXISTS password_reset_tokens (
  id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id         UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  token_hash      TEXT        NOT NULL,   -- SHA-256 of reset token
  expires_at      TIMESTAMPTZ NOT NULL,
  used            BOOLEAN     NOT NULL DEFAULT false,
  used_at         TIMESTAMPTZ,
  ip_address      INET,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Index for token lookup
CREATE INDEX IF NOT EXISTS idx_password_reset_token_hash
  ON password_reset_tokens(token_hash);

-- ── AUDIT LOG FOR SECURITY EVENTS ───────────────────────────────
CREATE TABLE IF NOT EXISTS security_audit_logs (
  id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id         UUID        REFERENCES users(id) ON DELETE SET NULL,
  action          TEXT        NOT NULL,   -- login, logout, 2fa_setup, 2fa_verify, password_change, password_reset_request, password_reset, 2fa_backup_used, session_revoked
  ip_address      INET,
  user_agent      TEXT,
  metadata        JSONB       NOT NULL DEFAULT '{}',
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_security_audit_user_time
  ON security_audit_logs(user_id, created_at);
CREATE INDEX IF NOT EXISTS idx_security_audit_action_time
  ON security_audit_logs(action, created_at);