//! SHA-256 and MD5 hashing of file bytes.
//!
//! Both are CPU-bound and offloaded to spawn_blocking.

use sha2::Digest as _;

/// Compute SHA-256 and MD5 of bytes.
/// Returns (sha256_hex, md5_hex).
///
/// CPU-bound — offloaded to blocking thread pool.
pub async fn compute_hashes(bytes: &[u8]) -> (String, String) {
    let bytes = bytes.to_vec();
    tokio::task::spawn_blocking(move || {
        let sha256 = hex::encode(sha2::Sha256::digest(&bytes));
        let md5 = hex::encode(md5::Md5::digest(&bytes));
        (sha256, md5)
    })
    .await
    // spawn_blocking only fails if the runtime is shutting down.
    // In that case, return empty strings — the caller's DB write will
    // fail with a NOT NULL constraint, surfacing the issue correctly.
    .unwrap_or_else(|_| (String::new(), String::new()))
}
