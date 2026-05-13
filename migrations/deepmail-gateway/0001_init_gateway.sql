CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE IF NOT EXISTS gateway_request_log (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id   UUID,
    user_id     UUID,
    method      TEXT        NOT NULL,
    path        TEXT        NOT NULL,
    status_code INTEGER     NOT NULL,
    latency_ms  INTEGER     NOT NULL,
    ip_address  TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_gateway_request_log_tenant_created
    ON gateway_request_log (tenant_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_gateway_request_log_created
    ON gateway_request_log (created_at DESC);

CREATE INDEX IF NOT EXISTS idx_gateway_request_log_status
    ON gateway_request_log (status_code);
