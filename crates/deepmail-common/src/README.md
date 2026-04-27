# `deepmail-common/src` Folder Guide

This folder contains the shared Rust modules that backend services import directly.

## Files

| File | Purpose |
| --- | --- |
| `lib.rs` | Public module surface and generated protobuf namespace exports. |
| `config.rs` | Shared environment loading and tracing initialization helpers. |
| `db.rs` | SQLx PostgreSQL pool and migration helpers. |
| `error.rs` | Shared application error type and gRPC status mapping. |
| `nats.rs` | NATS JetStream bootstrap and stream management helpers. |

## Function Index

- `lib.rs`
  - `proto::*`: generated gRPC modules for service-to-service contracts.
- `config.rs`
  - `require_env()`: enforces required configuration.
  - `optional_env()`: returns optional configuration values.
  - `env_or_default()`: parses values with a fallback default.
  - `ServiceConfig::from_env()`: loads shared service bootstrap config.
  - `init_tracing()`: configures JSON logging.
- `db.rs`
  - `create_pg_pool()`: opens the shared PostgreSQL pool.
  - `run_migrations()`: applies pending SQLx migrations.
- `error.rs`
  - `DeepMailError`: standard internal service error enum.
  - `From<DeepMailError> for tonic::Status`: converts internal errors to transport-safe gRPC status codes.
- `nats.rs`
  - `create_jetstream_context()`: connects to NATS.
  - `ensure_stream()`: ensures stream configuration exists.

## Editing Guidance

- Changes here affect every backend service.
- Keep APIs narrow and production-safe.
- Avoid introducing service-specific domain logic into this folder.
