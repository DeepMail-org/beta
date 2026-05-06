//! Database operations for the parser service.
//!
//! All queries use the `sqlx::query!` macro for compile-time verification.

pub mod insert;
pub mod progress;
