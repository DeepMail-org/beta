# `deepmail-otp-smtp` Crate Guide

`deepmail-otp-smtp` is the transactional outbound email service scaffold.

## Purpose

This crate is intended to deliver OTP messages, alert emails, and other DeepMail outbound email notifications.

## Contents

| Path | Purpose |
| --- | --- |
| `Cargo.toml` | Service manifest using shared Rust backend dependencies. |
| `src/` | Outbound email service source code. |

## Current Key Files

- `src/main.rs`
  - `main()`: standard bootstrap for config, tracing, PostgreSQL, NATS, and graceful shutdown.
