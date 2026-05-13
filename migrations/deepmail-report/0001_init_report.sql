CREATE TABLE IF NOT EXISTS report_exports (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email_id        UUID NOT NULL,
    tenant_id       UUID NOT NULL,
    report_format   TEXT NOT NULL
                    CHECK (report_format IN ('json','html')),
    s3_key          TEXT NOT NULL,
    file_size_bytes BIGINT NOT NULL DEFAULT 0,
    generated_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at      TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(email_id, report_format)
);

CREATE INDEX idx_re_tenant_gen ON report_exports(tenant_id, generated_at DESC);
CREATE INDEX idx_re_expires    ON report_exports(expires_at);

CREATE TABLE IF NOT EXISTS digest_schedules (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id        UUID NOT NULL UNIQUE,
    frequency        TEXT NOT NULL DEFAULT 'daily'
                     CHECK (frequency IN ('daily','weekly','never')),
    recipient_emails TEXT[] NOT NULL DEFAULT '{}',
    last_sent_at     TIMESTAMPTZ,
    next_send_at     TIMESTAMPTZ,
    is_active        BOOLEAN NOT NULL DEFAULT true,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_ds_next_send ON digest_schedules(next_send_at);
CREATE INDEX idx_ds_active    ON digest_schedules(is_active);
