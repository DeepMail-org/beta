//! gRPC service implementation for HashDbService.
//!
//! Implements three RPCs from hashdb.proto:
//!   CheckHash    — single hash lookup with bloom filter pre-check
//!   RegisterHash — upsert + bloom add + async ssdeep clustering
//!   BulkCheck    — up to 100 hashes, bloom-filtered batch query

use std::sync::Arc;

use tonic::{Request, Response, Status};

use deepmail_common::proto::hashdb::{
    hash_db_service_server::HashDbService,
    BulkCheckRequest, BulkCheckResponse,
    CheckHashRequest, CheckHashResponse,
    FileVerdict,
    RegisterHashRequest, RegisterHashResponse,
};

use crate::{
    bloom::{bloom_add, bloom_check},
    config::Config,
    db,
    fuzzy::run_ssdeep_clustering,
};

/// Convert a verdict string to the proto FileVerdict enum integer.
fn verdict_to_proto(verdict: &str) -> i32 {
    match verdict {
        "clean"      => FileVerdict::Clean as i32,
        "suspicious" => FileVerdict::Suspicious as i32,
        "malicious"  => FileVerdict::Malicious as i32,
        _            => FileVerdict::Unknown as i32,
    }
}

/// Convert a proto FileVerdict integer to a verdict string.
fn proto_to_verdict(verdict_i32: i32) -> &'static str {
    match verdict_i32 {
        v if v == FileVerdict::Clean as i32      => "clean",
        v if v == FileVerdict::Suspicious as i32 => "suspicious",
        v if v == FileVerdict::Malicious as i32  => "malicious",
        _                                         => "unknown",
    }
}

/// Convert a chrono DateTime to prost_types::Timestamp.
fn to_proto_ts(dt: chrono::DateTime<chrono::Utc>) -> prost_types::Timestamp {
    prost_types::Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    }
}

/// The HashDbService gRPC server implementation.
pub struct HashDbServiceImpl {
    pub pool: Arc<sqlx::PgPool>,
    pub redis: Arc<tokio::sync::Mutex<redis::aio::ConnectionManager>>,
    pub config: Arc<Config>,
}

#[tonic::async_trait]
impl HashDbService for HashDbServiceImpl {
    /// Look up a single hash. Uses bloom filter + PostgreSQL.
    #[tracing::instrument(skip(self, request))]
    async fn check_hash(
        &self,
        request: Request<CheckHashRequest>,
    ) -> Result<Response<CheckHashResponse>, Status> {
        let sha256 = request.into_inner().sha256.to_lowercase();

        if sha256.len() != 64 || !sha256.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(Status::invalid_argument("sha256 must be 64 hex characters"));
        }

        // ── Bloom filter pre-check ─────────────────────────────────
        let maybe_seen = {
            let mut conn = self.redis.lock().await;
            bloom_check(&mut conn, &self.config.bloom_filter_key, &sha256).await
        };

        if !maybe_seen {
            // Definitely not seen — skip PostgreSQL
            return Ok(Response::new(CheckHashResponse {
                found: false,
                verdict: FileVerdict::Unknown as i32,
                verdict_confidence: 0.0,
                analysis_required: true,
                malware_family: String::new(),
                seen_count: 0,
                first_seen: None,
            }));
        }

        // ── PostgreSQL exact lookup ────────────────────────────────
        let row = db::hashes::get_by_sha256(&self.pool, &sha256)
            .await
            .map_err(Status::from)?;

        match row {
            Some(h) => Ok(Response::new(CheckHashResponse {
                found: true,
                verdict: verdict_to_proto(&h.verdict),
                verdict_confidence: h.verdict_confidence,
                analysis_required: h.analysis_required,
                malware_family: h.malware_family.unwrap_or_default(),
                seen_count: h.seen_count,
                first_seen: Some(to_proto_ts(h.first_seen)),
            })),
            None => {
                // Bloom false positive — not actually in DB
                Ok(Response::new(CheckHashResponse {
                    found: false,
                    verdict: FileVerdict::Unknown as i32,
                    verdict_confidence: 0.0,
                    analysis_required: true,
                    malware_family: String::new(),
                    seen_count: 0,
                    first_seen: None,
                }))
            }
        }
    }

    /// Register a new hash (or update existing). Runs ssdeep clustering async.
    #[tracing::instrument(skip(self, request))]
    async fn register_hash(
        &self,
        request: Request<RegisterHashRequest>,
    ) -> Result<Response<RegisterHashResponse>, Status> {
        let req = request.into_inner();

        let sha256 = req.sha256.to_lowercase();
        if sha256.len() != 64 || !sha256.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(Status::invalid_argument("sha256 must be 64 hex characters"));
        }

        let verdict_str = proto_to_verdict(req.verdict);

        let (hash_id, was_new) = db::hashes::upsert_hash(
            &self.pool,
            db::hashes::HashRegisterInput {
                sha256: &sha256,
                md5: &req.md5.to_lowercase(),
                sha1: if req.sha1.is_empty() { None } else { Some(&req.sha1) },
                ssdeep: if req.ssdeep.is_empty() { None } else { Some(&req.ssdeep) },
                tlsh: if req.tlsh.is_empty() { None } else { Some(&req.tlsh) },
                imphash: if req.imphash.is_empty() { None } else { Some(&req.imphash) },
                file_type: &req.file_type,
                file_size_bytes: req.file_size_bytes,
                verdict: verdict_str,
                verdict_confidence: req.verdict_confidence,
                verdict_source: if req.verdict_source.is_empty() {
                    None
                } else {
                    Some(&req.verdict_source)
                },
                malware_family: if req.malware_family.is_empty() {
                    None
                } else {
                    Some(&req.malware_family)
                },
            },
        )
        .await
        .map_err(Status::from)?;

        // ── Add to bloom filter ────────────────────────────────────
        {
            let mut conn = self.redis.lock().await;
            bloom_add(
                &mut conn,
                &self.config.bloom_filter_key,
                self.config.bloom_fallback_ttl_seconds,
                &sha256,
            )
            .await;
        }

        // ── Async ssdeep clustering (fire-and-forget) ──────────────
        if !req.ssdeep.is_empty() {
            let pool = Arc::clone(&self.pool);
            let config = Arc::clone(&self.config);
            let ssdeep_val = req.ssdeep.clone();
            let sha256_val = sha256.clone();

            tokio::spawn(async move {
                if let Err(e) = run_ssdeep_clustering(
                    pool,
                    hash_id,
                    ssdeep_val,
                    sha256_val,
                    config,
                )
                .await
                {
                    tracing::warn!(
                        hash_id = %hash_id,
                        error = %e,
                        "ssdeep clustering failed (non-fatal)"
                    );
                }
            });
        }

        tracing::info!(
            sha256 = %sha256,
            hash_id = %hash_id,
            was_new = was_new,
            verdict = %verdict_str,
            "hash registered"
        );

        Ok(Response::new(RegisterHashResponse {
            registered: true,
            was_duplicate: !was_new,
            hash_id: hash_id.to_string(),
        }))
    }

    /// Bulk check up to 100 hashes.
    #[tracing::instrument(skip(self, request))]
    async fn bulk_check(
        &self,
        request: Request<BulkCheckRequest>,
    ) -> Result<Response<BulkCheckResponse>, Status> {
        let hashes = request.into_inner().sha256_hashes;

        if hashes.is_empty() {
            return Ok(Response::new(BulkCheckResponse { results: vec![] }));
        }
        if hashes.len() > 100 {
            return Err(Status::invalid_argument(
                "bulk_check accepts at most 100 hashes per call",
            ));
        }

        // Normalize all hashes
        let normalized: Vec<String> =
            hashes.iter().map(|h| h.to_lowercase()).collect();

        // ── Bloom filter pre-check (individual) ───────────────────
        let mut bloom_positives: Vec<String> = Vec::new();
        {
            let mut conn = self.redis.lock().await;
            for sha256 in &normalized {
                if bloom_check(&mut conn, &self.config.bloom_filter_key, sha256).await {
                    bloom_positives.push(sha256.clone());
                }
            }
        }

        // ── Batch PostgreSQL lookup for bloom positives ────────────
        let db_rows = if bloom_positives.is_empty() {
            Vec::new()
        } else {
            db::hashes::get_many_by_sha256(&self.pool, &bloom_positives)
                .await
                .map_err(Status::from)?
        };

        // Index DB results by sha256 for O(1) lookup
        let mut db_map: std::collections::HashMap<String, _> =
            db_rows.into_iter().map(|r| (r.sha256.clone(), r)).collect();

        // Build results in original input order
        let results = normalized
            .iter()
            .map(|sha256| {
                if let Some(row) = db_map.remove(sha256.as_str()) {
                    CheckHashResponse {
                        found: true,
                        verdict: verdict_to_proto(&row.verdict),
                        verdict_confidence: row.verdict_confidence,
                        analysis_required: row.analysis_required,
                        malware_family: row.malware_family.unwrap_or_default(),
                        seen_count: row.seen_count,
                        first_seen: Some(to_proto_ts(row.first_seen)),
                    }
                } else {
                    CheckHashResponse {
                        found: false,
                        verdict: FileVerdict::Unknown as i32,
                        verdict_confidence: 0.0,
                        analysis_required: true,
                        malware_family: String::new(),
                        seen_count: 0,
                        first_seen: None,
                    }
                }
            })
            .collect();

        Ok(Response::new(BulkCheckResponse { results }))
    }
}
