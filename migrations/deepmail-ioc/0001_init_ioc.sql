-- deepmail-ioc schema: IOC nodes, occurrences, relations, campaign clustering.

CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- ============================================================================
-- IOC nodes: one row per unique (tenant, type, value) triple
-- ============================================================================
CREATE TABLE ioc_nodes (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id         UUID NOT NULL,
    ioc_type          TEXT NOT NULL
                      CHECK (ioc_type IN ('ip','domain','url','hash','email',
                                          'cve','wallet','mutex','registry')),
    ioc_value         TEXT NOT NULL,
    threat_level      TEXT NOT NULL DEFAULT 'UNKNOWN'
                      CHECK (threat_level IN ('CLEAN','MODERATE','SUSPICIOUS',
                                              'MALICIOUS','UNKNOWN')),
    intel_score       REAL NOT NULL DEFAULT 0.0
                      CHECK (intel_score BETWEEN 0.0 AND 1.0),
    intel_json        JSONB NOT NULL DEFAULT '{}',
    sighting_count    INTEGER NOT NULL DEFAULT 1,
    email_ids         UUID[] NOT NULL DEFAULT '{}',
    first_seen        TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen         TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(tenant_id, ioc_type, ioc_value)
);

CREATE INDEX idx_ioc_nodes_tenant_type ON ioc_nodes(tenant_id, ioc_type);
CREATE INDEX idx_ioc_nodes_value ON ioc_nodes(ioc_value);
CREATE INDEX idx_ioc_nodes_tenant_last_seen ON ioc_nodes(tenant_id, last_seen DESC);
CREATE INDEX idx_ioc_nodes_threat_level ON ioc_nodes(threat_level);

-- ============================================================================
-- Per-email IOC occurrence (one per email+node pair)
-- ============================================================================
CREATE TABLE email_ioc_occurrences (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email_id          UUID NOT NULL,
    tenant_id         UUID NOT NULL,
    ioc_node_id       UUID NOT NULL REFERENCES ioc_nodes(id) ON DELETE CASCADE,
    extraction_source TEXT NOT NULL
                      CHECK (extraction_source IN ('header','body','attachment',
                                                   'url','subject')),
    raw_value         TEXT NOT NULL,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(email_id, ioc_node_id)
);

CREATE INDEX idx_email_ioc_occ_email ON email_ioc_occurrences(email_id);
CREATE INDEX idx_email_ioc_occ_tenant ON email_ioc_occurrences(tenant_id);
CREATE INDEX idx_email_ioc_occ_node ON email_ioc_occurrences(ioc_node_id);

-- ============================================================================
-- IOC relationship graph (for deepmail-graph to consume)
-- ============================================================================
CREATE TABLE ioc_relations (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id         UUID NOT NULL,
    source_ioc_id     UUID NOT NULL REFERENCES ioc_nodes(id) ON DELETE CASCADE,
    target_ioc_id     UUID NOT NULL REFERENCES ioc_nodes(id) ON DELETE CASCADE,
    relation_type     TEXT NOT NULL
                      CHECK (relation_type IN ('RESOLVES_TO','HOSTED_ON',
                                               'SUBDOMAIN_OF','SENT_FROM',
                                               'HAS_HASH','COMMUNICATES_WITH',
                                               'REDIRECTS_TO','DROPS')),
    email_id          UUID NOT NULL,
    confidence        REAL NOT NULL DEFAULT 1.0
                      CHECK (confidence BETWEEN 0.0 AND 1.0),
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(source_ioc_id, target_ioc_id, relation_type, email_id)
);

CREATE INDEX idx_ioc_relations_source ON ioc_relations(source_ioc_id);
CREATE INDEX idx_ioc_relations_target ON ioc_relations(target_ioc_id);
CREATE INDEX idx_ioc_relations_tenant_type ON ioc_relations(tenant_id, relation_type);

-- ============================================================================
-- Campaign clustering
-- ============================================================================
CREATE TABLE campaign_clusters (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id         UUID NOT NULL,
    campaign_name     TEXT NOT NULL,
    status            TEXT NOT NULL DEFAULT 'CANDIDATE'
                      CHECK (status IN ('CANDIDATE','CONFIRMED','ARCHIVED')),
    ioc_fingerprint   TEXT[] NOT NULL DEFAULT '{}',
    member_count      INTEGER NOT NULL DEFAULT 0,
    first_email_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_email_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_campaign_clusters_tenant_status ON campaign_clusters(tenant_id, status);
CREATE INDEX idx_campaign_clusters_tenant_last ON campaign_clusters(tenant_id, last_email_at DESC);

CREATE TABLE campaign_members (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    campaign_id       UUID NOT NULL REFERENCES campaign_clusters(id) ON DELETE CASCADE,
    email_id          UUID NOT NULL,
    tenant_id         UUID NOT NULL,
    similarity_score  REAL NOT NULL,
    joined_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(campaign_id, email_id)
);

CREATE INDEX idx_campaign_members_campaign ON campaign_members(campaign_id);
CREATE INDEX idx_campaign_members_email ON campaign_members(email_id);
CREATE INDEX idx_campaign_members_tenant ON campaign_members(tenant_id);
