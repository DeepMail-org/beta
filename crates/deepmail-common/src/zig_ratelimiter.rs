//! Zig Rate Limiter FFI Wrapper
//!
//! Provides safe Rust bindings to the Zig sliding-window rate limiter.
//! This is a fast, in-process rate limiter that acts as a first layer
//! before the distributed Redis check.

use std::ffi::{c_char, c_int, c_uint, c_ulonglong, c_void, CStr, CString};
use std::ptr::NonNull;
use std::sync::OnceLock;

/// Opaque pointer to the Zig rate limiter
#[repr(transparent)]
pub struct ZigRateLimiter(NonNull<c_void>);

unsafe impl Send for ZigRateLimiter {}
unsafe impl Sync for ZigRateLimiter {}

extern "C" {
    fn ratelimit_create(max_requests: c_uint, window_ms: c_ulonglong) -> *mut c_void;
    fn ratelimit_check(limiter: *mut c_void, key: *const u8, key_len: usize) -> c_int;
    fn ratelimit_reset(limiter: *mut c_void, key: *const u8, key_len: usize);
    fn ratelimit_cleanup(limiter: *mut c_void);
    fn ratelimit_destroy(limiter: *mut c_void);
}

impl ZigRateLimiter {
    /// Create a new rate limiter instance
    pub fn new(max_requests: u32, window_ms: u64) -> Option<Self> {
        let ptr = unsafe { ratelimit_create(max_requests, window_ms) };
        NonNull::new(ptr).map(ZigRateLimiter)
    }

    /// Check if a request is allowed for the given key
    /// Returns true if allowed, false if rate limited
    pub fn check(&self, key: &str) -> bool {
        let key_bytes = key.as_bytes();
        let result = unsafe {
            ratelimit_check(
                self.0.as_ptr(),
                key_bytes.as_ptr(),
                key_bytes.len(),
            )
        };
        result == 1
    }

    /// Reset the rate limit counter for a specific key
    pub fn reset(&self, key: &str) {
        let key_bytes = key.as_bytes();
        unsafe {
            ratelimit_reset(self.0.as_ptr(), key_bytes.as_ptr(), key_bytes.len());
        }
    }

    /// Clean up expired entries
    pub fn cleanup(&self) {
        unsafe { ratelimit_cleanup(self.0.as_ptr()) };
    }
}

impl Drop for ZigRateLimiter {
    fn drop(&mut self) {
        unsafe { ratelimit_destroy(self.0.as_ptr()) };
    }
}

/// Rate limit configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub max_requests: u32,
    pub window_ms: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window_ms: 60_000, // 1 minute
        }
    }
}

/// Global rate limiter instances (one per rate limit policy)
static GLOBAL_LIMITERS: OnceLock<GlobalLimiters> = OnceLock::new();

struct GlobalLimiters {
    // Per-IP login attempts: 5 per 5 minutes
    login_ip: ZigRateLimiter,
    // Per-email login attempts: 5 per 5 minutes
    login_email: ZigRateLimiter,
    // Per-email OTP requests: 3 per 10 minutes
    otp_email: ZigRateLimiter,
    // Per-IP password reset: 3 per 10 minutes
    reset_ip: ZigRateLimiter,
    // General API: 1000 per minute
    api_general: ZigRateLimiter,
    // Per-tenant API: configurable
    tenant_api: ZigRateLimiter,
}

impl GlobalLimiters {
    fn init() -> Self {
        Self {
            login_ip: ZigRateLimiter::new(5, 5 * 60 * 1000).expect("login_ip limiter"),
            login_email: ZigRateLimiter::new(5, 5 * 60 * 1000).expect("login_email limiter"),
            otp_email: ZigRateLimiter::new(3, 10 * 60 * 1000).expect("otp_email limiter"),
            reset_ip: ZigRateLimiter::new(3, 10 * 60 * 1000).expect("reset_ip limiter"),
            api_general: ZigRateLimiter::new(1000, 60 * 1000).expect("api_general limiter"),
            tenant_api: ZigRateLimiter::new(100, 60 * 1000).expect("tenant_api limiter"),
        }
    }

    pub fn global() -> &'static Self {
        GLOBAL_LIMITERS.get_or_init(Self::init)
    }
}

/// Rate limit check helpers for common auth scenarios
pub mod auth {
    use super::*;

    /// Check login rate limit by IP
    pub fn check_login_ip(ip: &str) -> bool {
        GlobalLimiters::global().login_ip.check(&format!("login:ip:{}", ip))
    }

    /// Check login rate limit by email
    pub fn check_login_email(email: &str) -> bool {
        GlobalLimiters::global().login_email.check(&format!("login:email:{}", email))
    }

    /// Check OTP request rate limit by email
    pub fn check_otp_email(email: &str) -> bool {
        GlobalLimiters::global().otp_email.check(&format!("otp:email:{}", email))
    }

    /// Check password reset rate limit by IP
    pub fn check_reset_ip(ip: &str) -> bool {
        GlobalLimiters::global().reset_ip.check(&format!("reset:ip:{}", ip))
    }
}

/// Rate limit check for general API endpoints
pub fn check_api(key: &str) -> bool {
    GlobalLimiters::global().api_general.check(key)
}

/// Rate limit check for tenant-scoped API endpoints
pub fn check_tenant_api(tenant_id: &str, key: &str) -> bool {
    GlobalLimiters::global().tenant_api.check(&format!("tenant:{}:{}", tenant_id, key))
}

/// Initialize the rate limiter system (call on startup)
pub fn init() {
    GlobalLimiters::global();
    tracing::info!("Zig rate limiter initialized");
}

/// Cleanup all rate limiters (call on shutdown)
pub fn cleanup_all() {
    if let Some(limiters) = GLOBAL_LIMITERS.get() {
        limiters.login_ip.cleanup();
        limiters.login_email.cleanup();
        limiters.otp_email.cleanup();
        limiters.reset_ip.cleanup();
        limiters.api_general.cleanup();
        limiters.tenant_api.cleanup();
    }
}