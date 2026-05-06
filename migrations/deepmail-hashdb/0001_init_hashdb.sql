CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- ── FILE HASHES ───────────────────────────────────────────────────
-- Global registry. Not tenant-scoped.
-- SHA-256 is the primary key for deduplication.
CREATE TABLE IF NOT EXISTS file_hashes (
  id                  UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  sha256              TEXT        NOT NULL,
  md5                 TEXT        NOT NULL,
  sha1                TEXT,
  ssdeep              TEXT,
  tlsh                TEXT,
  imphash             TEXT,
  file_type           TEXT        NOT NULL,
  file_size_bytes     BIGINT      NOT NULL,
  verdict             TEXT        NOT NULL DEFAULT 'unknown',
  verdict_confidence  REAL        NOT NULL DEFAULT 0.0,
  verdict_source      TEXT,
  malware_family      TEXT,
  analysis_required   BOOLEAN     NOT NULL DEFAULT true,
  seen_count          INTEGER     NOT NULL DEFAULT 1,
  first_seen          TIMESTAMPTZ NOT NULL DEFAULT now(),
  last_seen           TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
  CONSTRAINT file_hashes_sha256_key UNIQUE (sha256),
  CONSTRAINT file_hashes_verdict_check CHECK (
    verdict IN ('unknown','clean','suspicious','malicious')
  )
);

-- ── HASH CLUSTERS ─────────────────────────────────────────────────
-- Fuzzy similarity groups.
-- Two files are clustered when ssdeep similarity >= 70.
-- representative_hash_id is the earlier-registered hash.
CREATE TABLE IF NOT EXISTS hash_clusters (
  id                       UUID    PRIMARY KEY DEFAULT gen_random_uuid(),
  representative_hash_id   UUID    NOT NULL REFERENCES file_hashes(id),
  cluster_hash_id          UUID    NOT NULL REFERENCES file_hashes(id),
  similarity_pct           INTEGER NOT NULL,
  method                   TEXT    NOT NULL,
  created_at               TIMESTAMPTZ NOT NULL DEFAULT now(),
  CONSTRAINT hash_clusters_unique UNIQUE
    (representative_hash_id, cluster_hash_id, method),
  CONSTRAINT hash_clusters_method_check CHECK (
    method IN ('ssdeep','tlsh','imphash')
  ),
  CONSTRAINT hash_clusters_similarity_check CHECK (
    similarity_pct BETWEEN 0 AND 100
  )
);

-- ── INDEXES ───────────────────────────────────────────────────────
CREATE INDEX IF NOT EXISTS idx_hashes_sha256
  ON file_hashes(sha256);

CREATE INDEX IF NOT EXISTS idx_hashes_md5
  ON file_hashes(md5);

CREATE INDEX IF NOT EXISTS idx_hashes_imphash
  ON file_hashes(imphash)
  WHERE imphash IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_hashes_verdict
  ON file_hashes(verdict);

CREATE INDEX IF NOT EXISTS idx_hashes_analysis_required
  ON file_hashes(analysis_required)
  WHERE analysis_required = true;

CREATE INDEX IF NOT EXISTS idx_hashes_last_seen
  ON file_hashes(last_seen DESC);

CREATE INDEX IF NOT EXISTS idx_clusters_rep
  ON hash_clusters(representative_hash_id);

CREATE INDEX IF NOT EXISTS idx_clusters_member
  ON hash_clusters(cluster_hash_id);
