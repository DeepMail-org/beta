/// Per-provider circuit breaker with CLOSED → OPEN → HALF_OPEN state machine.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::RwLock;

use crate::error::IntelError;

const MAX_FAILURES: u32 = 5;
const OPEN_DURATION: Duration = Duration::from_secs(60);

#[derive(Debug, Clone)]
pub enum CbState {
    Closed { consecutive_failures: u32 },
    Open { opened_at: Instant },
    HalfOpen,
}

impl CbState {
    pub fn as_str(&self) -> &'static str {
        match self {
            CbState::Closed { .. } => "CLOSED",
            CbState::Open { .. } => "OPEN",
            CbState::HalfOpen => "HALF_OPEN",
        }
    }
}

pub struct CircuitBreaker {
    provider: &'static str,
    state: Arc<RwLock<CbState>>,
}

impl CircuitBreaker {
    pub fn new(provider: &'static str) -> Self {
        Self {
            provider,
            state: Arc::new(RwLock::new(CbState::Closed {
                consecutive_failures: 0,
            })),
        }
    }

    /// Execute `f` through the circuit breaker.
    pub async fn call<F, T, Fut>(&self, f: F) -> Result<T, IntelError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, IntelError>>,
    {
        // Check current state
        {
            let mut state = self.state.write().await;
            match *state {
                CbState::Open { opened_at } => {
                    if opened_at.elapsed() >= OPEN_DURATION {
                        tracing::info!(
                            provider = self.provider,
                            "circuit breaker: OPEN → HALF_OPEN"
                        );
                        *state = CbState::HalfOpen;
                    } else {
                        return Err(IntelError::CircuitOpen(self.provider.to_string()));
                    }
                }
                CbState::Closed { .. } | CbState::HalfOpen => {}
            }
        }

        // Execute the function
        let result = f().await;

        // Update state based on result
        let mut state = self.state.write().await;
        match result {
            Ok(val) => {
                if !matches!(*state, CbState::Closed { consecutive_failures: 0 }) {
                    tracing::info!(
                        provider = self.provider,
                        "circuit breaker: {} → CLOSED",
                        state.as_str()
                    );
                }
                *state = CbState::Closed {
                    consecutive_failures: 0,
                };
                Ok(val)
            }
            Err(e) => {
                match *state {
                    CbState::Closed {
                        consecutive_failures,
                    } => {
                        let new_failures = consecutive_failures + 1;
                        if new_failures >= MAX_FAILURES {
                            tracing::info!(
                                provider = self.provider,
                                failures = new_failures,
                                "circuit breaker: CLOSED → OPEN"
                            );
                            *state = CbState::Open {
                                opened_at: Instant::now(),
                            };
                        } else {
                            *state = CbState::Closed {
                                consecutive_failures: new_failures,
                            };
                        }
                    }
                    CbState::HalfOpen => {
                        tracing::info!(
                            provider = self.provider,
                            "circuit breaker: HALF_OPEN → OPEN"
                        );
                        *state = CbState::Open {
                            opened_at: Instant::now(),
                        };
                    }
                    CbState::Open { .. } => {
                        // Should not reach here, but keep open
                    }
                }
                Err(e)
            }
        }
    }

    pub async fn is_open(&self) -> bool {
        let state = self.state.read().await;
        matches!(*state, CbState::Open { .. })
    }

    pub async fn reset(&self) {
        let mut state = self.state.write().await;
        *state = CbState::Closed {
            consecutive_failures: 0,
        };
    }

    pub async fn state_str(&self) -> &'static str {
        let state = self.state.read().await;
        state.as_str()
    }
}

pub struct CircuitRegistry {
    breakers: HashMap<&'static str, Arc<CircuitBreaker>>,
}

impl CircuitRegistry {
    pub fn new() -> Self {
        let providers = [
            "virustotal",
            "abuseipdb",
            "greynoise",
            "ipinfo",
            "shodan",
            "otx",
        ];
        let mut breakers = HashMap::new();
        for name in providers {
            breakers.insert(name, Arc::new(CircuitBreaker::new(name)));
        }
        Self { breakers }
    }

    pub fn get(&self, provider: &str) -> Option<Arc<CircuitBreaker>> {
        self.breakers.get(provider).cloned()
    }

    pub async fn all_statuses(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        for (name, cb) in &self.breakers {
            map.insert(name.to_string(), cb.state_str().await.to_string());
        }
        map
    }
}
