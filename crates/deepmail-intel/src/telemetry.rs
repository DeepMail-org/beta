/// In-memory telemetry accumulator + periodic flush to DB.

use std::collections::HashMap;
use std::sync::Arc;

use sqlx::PgPool;
use tokio::sync::RwLock;

use crate::error::IntelError;

#[derive(Debug, Default)]
struct ProviderCounters {
    requests: u64,
    successes: u64,
    failures: u64,
    cache_hits: u64,
    total_latency_ms: u64,
}

pub struct TelemetryAccumulator {
    counters: Arc<RwLock<HashMap<String, ProviderCounters>>>,
    window_start: Arc<RwLock<chrono::DateTime<chrono::Utc>>>,
}

impl TelemetryAccumulator {
    pub fn new() -> Self {
        Self {
            counters: Arc::new(RwLock::new(HashMap::new())),
            window_start: Arc::new(RwLock::new(chrono::Utc::now())),
        }
    }

    pub async fn record_request(
        &self,
        provider: &str,
        latency_ms: u64,
        success: bool,
        cache_hit: bool,
    ) {
        let mut counters = self.counters.write().await;
        let entry = counters
            .entry(provider.to_string())
            .or_insert_with(ProviderCounters::default);

        entry.requests += 1;
        if success {
            entry.successes += 1;
        } else {
            entry.failures += 1;
        }
        if cache_hit {
            entry.cache_hits += 1;
        }
        entry.total_latency_ms += latency_ms;
    }

    pub async fn flush_to_db(&self, pool: &PgPool) -> Result<(), IntelError> {
        let now = chrono::Utc::now();

        let (drained, window_start) = {
            let mut counters = self.counters.write().await;
            let mut ws = self.window_start.write().await;
            let start = *ws;
            *ws = now;
            let drained: HashMap<String, ProviderCounters> = counters.drain().collect();
            (drained, start)
        };

        for (provider, c) in &drained {
            if c.requests == 0 {
                continue;
            }

            sqlx::query(
                r#"INSERT INTO provider_telemetry
                       (provider, window_start, window_end,
                        request_count, success_count, failure_count,
                        cache_hit_count, total_latency_ms)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
            )
            .bind(provider)
            .bind(window_start)
            .bind(now)
            .bind(c.requests as i32)
            .bind(c.successes as i32)
            .bind(c.failures as i32)
            .bind(c.cache_hits as i32)
            .bind(c.total_latency_ms as i64)
            .execute(pool)
            .await?;
        }

        Ok(())
    }
}

/// Background task: flush telemetry every N seconds.
pub async fn telemetry_flush_loop(
    accumulator: Arc<TelemetryAccumulator>,
    pool: PgPool,
    interval_secs: u64,
) {
    let interval = std::time::Duration::from_secs(interval_secs);
    loop {
        tokio::time::sleep(interval).await;
        match accumulator.flush_to_db(&pool).await {
            Ok(()) => tracing::debug!("telemetry flushed"),
            Err(e) => tracing::warn!(error = %e, "telemetry flush failed"),
        }
    }
}
