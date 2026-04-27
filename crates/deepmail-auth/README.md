# `deepmail-auth` Crate Guide

`deepmail-auth` is the authentication service scaffold.

## Purpose

This crate is intended to own password hashing, OTP verification, JWT issuance, refresh-token rotation, API key validation, and lockout/audit flows.

## Contents

| Path | Purpose |
| --- | --- |
| `Cargo.toml` | Service manifest using shared Rust backend dependencies. |
| `src/` | Authentication service source code. |

## Current Key Files

- `src/main.rs`
  - `main()`: loads shared service configuration, initializes JSON tracing, creates PostgreSQL and NATS connections, and waits for shutdown.

## Notes

- The current code is a startup scaffold only; auth business logic is not implemented yet.
