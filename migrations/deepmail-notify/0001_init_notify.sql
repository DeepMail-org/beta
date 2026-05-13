-- deepmail-notify schema

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-------------------------------------------------------
-- notification_configs: per-tenant notification prefs
-------------------------------------------------------
CREATE TABLE notification_configs (
    id              UUID        PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id       UUID        NOT NULL UNIQUE,
    webhook_url     TEXT,
    webhook_secret  TEXT,
    webhook_active  BOOLEAN     NOT NULL DEFAULT false,
    smtp_enabled    BOOLEAN     NOT NULL DEFAULT true,
    min_severity    TEXT        NOT NULL DEFAULT 'PHISHING',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_notification_configs_tenant ON notification_configs (tenant_id);

-------------------------------------------------------
-- notification_logs: delivery audit log
-------------------------------------------------------
CREATE TABLE notification_logs (
    id              UUID        PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id       UUID        NOT NULL,
    email_id        UUID        NOT NULL,
    event_type      TEXT        NOT NULL,
    channel         TEXT        NOT NULL,
    status          TEXT        NOT NULL DEFAULT 'pending',
    recipient       TEXT,
    payload         JSONB,
    error_message   TEXT,
    attempt_count   INTEGER     NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_notification_logs_tenant   ON notification_logs (tenant_id);
CREATE INDEX idx_notification_logs_email    ON notification_logs (email_id);
CREATE INDEX idx_notification_logs_status   ON notification_logs (status);
CREATE INDEX idx_notification_logs_channel  ON notification_logs (channel);
CREATE INDEX idx_notification_logs_created  ON notification_logs (created_at);

-------------------------------------------------------
-- websocket_sessions: WS connection tracking
-------------------------------------------------------
CREATE TABLE websocket_sessions (
    id              UUID        PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id       UUID        NOT NULL,
    user_id         UUID        NOT NULL,
    session_token   TEXT        NOT NULL,
    connected_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    disconnected_at TIMESTAMPTZ,
    is_active       BOOLEAN     NOT NULL DEFAULT true
);

CREATE INDEX idx_ws_sessions_tenant  ON websocket_sessions (tenant_id);
CREATE INDEX idx_ws_sessions_active  ON websocket_sessions (is_active) WHERE is_active = true;
