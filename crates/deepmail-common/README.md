# `deepmail-common` Crate Guide

`deepmail-common` is the shared Rust crate used by every DeepMail backend service.

## Purpose

This crate centralizes startup and integration utilities so service crates do not duplicate configuration, tracing, database, NATS, and protobuf wiring.

## Contents

| Path | Purpose |
| --- | --- |
| `Cargo.toml` | Shared crate manifest and build dependencies for protobuf generation. |
| `build.rs` | Runs `tonic_build` to compile all protobuf contracts under `proto/`. |
| `proto/` | Source `.proto` files for inter-service gRPC contracts. |
| `src/` | Shared Rust modules exported to service crates. |

## Important Files And Functions

- `build.rs`
  - `main()`: compiles protobuf files and marks them as Cargo rebuild inputs.
- `src/config.rs`
  - `require_env()`: loads required environment variables with typed errors.
  - `optional_env()`: loads optional environment variables.
  - `env_or_default()`: parses optional environment variables with fallback defaults.
  - `ServiceConfig::from_env()`: loads standard service bootstrap configuration.
  - `init_tracing()`: configures JSON tracing output for services.
- `src/db.rs`
  - `create_pg_pool()`: creates the shared SQLx PostgreSQL connection pool.
  - `run_migrations()`: runs SQLx migrations through a provided migrator.
- `src/nats.rs`
  - `create_jetstream_context()`: creates a NATS JetStream context.
  - `ensure_stream()`: idempotently creates or updates a JetStream stream.
- `src/error.rs`
  - `DeepMailError`: shared backend error type.
  - `From<DeepMailError> for tonic::Status`: maps internal errors to gRPC status codes.
- `src/lib.rs`
  - exports shared modules and the generated `proto` namespace.

## How This Crate Fits Into DeepMail

- Service `main.rs` files depend on it for startup scaffolding.
- Generated protobuf modules are imported through `deepmail_common::proto::*`.
- Common error and connection logic lives here so services can remain focused on domain behavior.

## Read Next

- `src/README.md`
- `proto/README.md`
