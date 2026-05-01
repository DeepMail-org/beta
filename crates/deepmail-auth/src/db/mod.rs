//! Database access layer for deepmail-auth.
//!
//! Each sub-module corresponds to one database table.
//! All queries use `sqlx::query!` for compile-time verification.

pub mod audit;
pub mod lockouts;
pub mod otp;
pub mod refresh_tokens;
pub mod users;
