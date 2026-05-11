-- deepmail-scoring: threat score aggregation tables

CREATE TABLE IF NOT EXISTS threat_scores (
    id                    UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email_id              UUID NOT NULL,
    tenant_id             UUID NOT NULL,
    final_score           REAL NOT NULL DEFAULT 0.0
                          CHECK (final_score BETWEEN 0.0 AND 1.0),
    final_verdict         TEXT NOT NULL DEFAULT 'CLEAN'
                          CHECK (final_verdict IN ('CLEAN','LOW_RISK','SUSPICIOUS',
                                                   'PHISHING','MALICIOUS')),
    signals_available     INTEGER NOT NULL DEFAULT 0,
    signals_total         INTEGER NOT NULL DEFAULT 11,
    header_score          REAL,
    dkim_score            REAL,
    geo_score             REAL,
    ip_score              REAL,
    intel_score           REAL,
    ioc_score             REAL,
    homograph_score       REAL,
    body_score            REAL,
    sandbox_url_score     REAL,
    sandbox_file_score    REAL,
    sandbox_dynamic_score REAL,
    weight_total          REAL NOT NULL DEFAULT 0.0,
    is_final              BOOLEAN NOT NULL DEFAULT false,
    computed_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(email_id)
);

CREATE INDEX idx_threat_scores_tenant_time ON threat_scores(tenant_id, computed_at DESC);
CREATE INDEX idx_threat_scores_verdict     ON threat_scores(final_verdict);
CREATE INDEX idx_threat_scores_score       ON threat_scores(final_score DESC);
CREATE INDEX idx_threat_scores_is_final    ON threat_scores(is_final);

CREATE TABLE IF NOT EXISTS signal_scores (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email_id      UUID NOT NULL,
    tenant_id     UUID NOT NULL,
    signal_name   TEXT NOT NULL,
    score         REAL NOT NULL,
    raw_data      JSONB NOT NULL DEFAULT '{}',
    received_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(email_id, signal_name)
);

CREATE INDEX idx_signal_scores_email      ON signal_scores(email_id);
CREATE INDEX idx_signal_scores_tenant_time ON signal_scores(tenant_id, received_at DESC);

CREATE TABLE IF NOT EXISTS analyst_feedback (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email_id          UUID NOT NULL,
    tenant_id         UUID NOT NULL,
    analyst_id        UUID NOT NULL,
    predicted_verdict TEXT NOT NULL,
    correct_verdict   TEXT NOT NULL
                      CHECK (correct_verdict IN ('CLEAN','LOW_RISK','SUSPICIOUS',
                                                 'PHISHING','MALICIOUS')),
    feedback_notes    TEXT,
    signal_overrides  JSONB NOT NULL DEFAULT '{}',
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_analyst_feedback_email   ON analyst_feedback(email_id);
CREATE INDEX idx_analyst_feedback_tenant  ON analyst_feedback(tenant_id);
CREATE INDEX idx_analyst_feedback_analyst ON analyst_feedback(analyst_id);
