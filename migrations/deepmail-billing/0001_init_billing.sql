CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Per-event cost lookup (seeded with 8 pipeline event types)
CREATE TABLE billing_event_costs (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    event_type  TEXT NOT NULL UNIQUE,
    cost_paise  INTEGER NOT NULL DEFAULT 0,
    description TEXT NOT NULL DEFAULT '',
    is_active   BOOLEAN NOT NULL DEFAULT TRUE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Metered pipeline events with UUIDv5 idempotency
CREATE TABLE meter_events (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    idempotency_key UUID NOT NULL UNIQUE,
    tenant_id       UUID NOT NULL,
    email_id        UUID NOT NULL,
    event_type      TEXT NOT NULL,
    cost_paise      INTEGER NOT NULL DEFAULT 0,
    billing_period  TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Invoices per tenant per billing period
CREATE TABLE invoices (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id       UUID NOT NULL,
    billing_period  TEXT NOT NULL,
    razorpay_id     TEXT,
    status          TEXT NOT NULL DEFAULT 'draft'
                    CHECK (status IN ('draft', 'issued', 'paid', 'cancelled', 'expired')),
    total_paise     BIGINT NOT NULL DEFAULT 0,
    line_items_json JSONB NOT NULL DEFAULT '[]'::jsonb,
    issued_at       TIMESTAMPTZ,
    paid_at         TIMESTAMPTZ,
    due_at          TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, billing_period)
);

-- Razorpay webhook events for idempotent processing
CREATE TABLE razorpay_events (
    id         UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    event_id   TEXT NOT NULL UNIQUE,
    event_type TEXT NOT NULL,
    payload    JSONB NOT NULL DEFAULT '{}'::jsonb,
    processed  BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Indexes
CREATE INDEX idx_meter_events_tenant_period ON meter_events (tenant_id, billing_period);
CREATE INDEX idx_meter_events_email         ON meter_events (email_id);
CREATE INDEX idx_meter_events_event_type    ON meter_events (event_type);
CREATE INDEX idx_invoices_tenant            ON invoices (tenant_id);
CREATE INDEX idx_invoices_razorpay_id       ON invoices (razorpay_id) WHERE razorpay_id IS NOT NULL;
CREATE INDEX idx_invoices_status            ON invoices (status);
CREATE INDEX idx_razorpay_events_processed  ON razorpay_events (processed) WHERE processed = FALSE;

-- Seed billing_event_costs with 8 pipeline event types
INSERT INTO billing_event_costs (event_type, cost_paise, description) VALUES
    ('email_ingested',      100,  'Email ingested into pipeline'),
    ('header_analyzed',      50,  'Header analysis completed'),
    ('body_analyzed',        50,  'Body content analysis completed'),
    ('url_sandboxed',       200,  'URL sandbox scan'),
    ('file_sandboxed',      300,  'File sandbox scan'),
    ('dynamic_sandboxed',   500,  'Dynamic sandbox analysis'),
    ('ioc_extracted',       150,  'IOC extraction and enrichment'),
    ('ml_inference',        250,  'ML model inference');
