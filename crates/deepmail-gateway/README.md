# `deepmail-gateway` Crate Guide

`deepmail-gateway` is the planned external entrypoint for DeepMail.

## Purpose

This crate will eventually own HTTP ingress, GraphQL exposure, WebSocket proxying, authentication enforcement, and synchronous fan-out to internal services. Today it contains a lightweight Axum bootstrap scaffold.

## Contents

| Path | Purpose |
| --- | --- |
| `Cargo.toml` | Gateway-specific dependencies such as Axum, async-graphql, tower, and WebSocket support. |
| `src/` | Gateway source code. |

## Current Key Files

- `src/main.rs`
  - `main()`: initializes tracing, loads required environment variables, builds an Axum router scaffold, and starts graceful shutdown handling.
  - `shutdown_signal()`: waits for `CTRL+C` and logs shutdown.

## Notes

- This crate is the only backend crate currently wired around an HTTP server bootstrap instead of the generic gRPC bootstrap.
- `shutdown_signal()` currently uses `.expect(...)` and should be hardened before this crate is considered production-ready.
