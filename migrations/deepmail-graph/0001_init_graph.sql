CREATE TABLE IF NOT EXISTS graph_sync_log (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email_id        UUID NOT NULL,
    tenant_id       UUID NOT NULL,
    nodes_created   INTEGER NOT NULL DEFAULT 0,
    edges_created   INTEGER NOT NULL DEFAULT 0,
    sync_status     TEXT NOT NULL DEFAULT 'pending'
                    CHECK (sync_status IN ('pending', 'completed', 'failed')),
    error_message   TEXT,
    synced_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(email_id)
);

CREATE INDEX IF NOT EXISTS idx_graph_sync_log_tenant_synced
    ON graph_sync_log (tenant_id, synced_at DESC);
CREATE INDEX IF NOT EXISTS idx_graph_sync_log_status
    ON graph_sync_log (sync_status);

CREATE TABLE IF NOT EXISTS graph_node_cache (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID NOT NULL,
    node_type       TEXT NOT NULL
                    CHECK (node_type IN ('ip', 'domain', 'url', 'hash',
                                         'email', 'campaign', 'sender', 'attachment')),
    node_value      TEXT NOT NULL,
    neo4j_id        TEXT NOT NULL,
    properties_json JSONB NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(tenant_id, node_type, node_value)
);

CREATE INDEX IF NOT EXISTS idx_graph_node_cache_tenant_type
    ON graph_node_cache (tenant_id, node_type);
CREATE INDEX IF NOT EXISTS idx_graph_node_cache_value
    ON graph_node_cache (node_value);
