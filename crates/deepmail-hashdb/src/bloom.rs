//! Redis bloom filter for sub-millisecond hash pre-checks.
//!
//! Primary: RedisBloom BF.ADD / BF.EXISTS commands.
//! Fallback: Plain Redis SET/EXISTS with TTL (when RedisBloom unavailable).
//!
//! False positives are acceptable — they cause an extra PostgreSQL lookup.
//! False negatives are NOT acceptable — they would skip analysis of known files.
//! The bloom filter is therefore write-only for new hashes,
//! and read-only for check operations.

use redis::AsyncCommands;

/// Add a SHA-256 hash to the bloom filter.
///
/// Tries RedisBloom BF.ADD first; falls back to SET with TTL if unavailable.
pub async fn bloom_add(
    conn: &mut redis::aio::ConnectionManager,
    bloom_key: &str,
    fallback_ttl: u64,
    sha256: &str,
) {
    // Try BF.ADD (RedisBloom)
    let bf_result: redis::RedisResult<i64> = redis::cmd("BF.ADD")
        .arg(bloom_key)
        .arg(sha256)
        .query_async(conn)
        .await;

    if bf_result.is_ok() {
        return; // RedisBloom succeeded
    }

    // Fallback: plain SET with TTL
    let fallback_key = format!("deepmail:hashdb:seen:{sha256}");
    let _: redis::RedisResult<()> = conn
        .set_ex(&fallback_key, 1u8, fallback_ttl)
        .await;
    // Silently ignore fallback errors — bloom is an optimization
}

/// Check if a SHA-256 hash might be in the bloom filter.
///
/// Returns:
///   true  = "maybe seen" — must check PostgreSQL to confirm
///   false = "definitely not seen" — skip PostgreSQL lookup
///
/// On any Redis error: returns true (safe default — forces PostgreSQL lookup).
pub async fn bloom_check(
    conn: &mut redis::aio::ConnectionManager,
    bloom_key: &str,
    sha256: &str,
) -> bool {
    // Try BF.EXISTS (RedisBloom)
    let bf_result: redis::RedisResult<i64> = redis::cmd("BF.EXISTS")
        .arg(bloom_key)
        .arg(sha256)
        .query_async(conn)
        .await;

    if let Ok(result) = bf_result {
        return result == 1;
    }

    // Fallback: plain EXISTS check
    let fallback_key = format!("deepmail:hashdb:seen:{sha256}");
    let exists: redis::RedisResult<bool> = conn.exists(&fallback_key).await;

    match exists {
        Ok(found) => found,
        Err(_) => true, // Safe default: assume possibly seen → check PostgreSQL
    }
}
