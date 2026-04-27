# DeepMail

DeepMail is a production-oriented email threat intelligence platform organized as a Rust workspace with one Python ML service planned outside the workspace. Users upload `.eml` or `.msg` samples, backend services analyze them across multiple intelligence layers, and the frontend presents threat reports, graphs, maps, and sandbox outputs.

## Workspace Purpose

This repository currently contains:

- the Rust microservice workspace under `crates/`
- shared architecture and schema reference documents under `.claude/`
- a Next.js prototype frontend under `frontend-beta/`
- a placeholder backend artifact area under `backend-beta/`

The repository is still in an early scaffold phase. Many backend crates currently only bootstrap config, tracing, database, and NATS connectivity, then wait for shutdown. The READMEs in this repo are meant to help humans and LLM agents quickly understand what already exists, what each folder owns, and where implementation work should happen next.

## Top-Level Layout

| Path | Purpose |
| --- | --- |
| `.claude/` | Generated architecture and schema reference documents used as planning artifacts. |
| `backend-beta/` | Placeholder backend output area; currently only contains ignored build artifacts. |
| `crates/` | Rust workspace members: shared crate plus one crate per DeepMail service. |
| `frontend-beta/` | Next.js 16 prototype frontend for upload, analysis, map, graph, and report views. |
| `Cargo.toml` | Rust workspace manifest listing all crates and shared dependency versions. |
| `Cargo.lock` | Cargo dependency lockfile for deterministic Rust builds. |
| `.gitignore` | Repository-wide ignore policy for build outputs, package installs, and local secrets. |

## Backend Service Inventory

The workspace currently contains `deepmail-common` plus these service crates:

- `deepmail-gateway`
- `deepmail-auth`
- `deepmail-tenant`
- `deepmail-ingest`
- `deepmail-parser`
- `deepmail-otp-smtp`
- `deepmail-header`
- `deepmail-dkim`
- `deepmail-geo`
- `deepmail-ip`
- `deepmail-intel`
- `deepmail-ioc`
- `deepmail-homograph`
- `deepmail-body`
- `deepmail-sandbox-url`
- `deepmail-sandbox-file`
- `deepmail-sandbox-dynamic`
- `deepmail-hashdb`
- `deepmail-scoring`
- `deepmail-graph`
- `deepmail-report`
- `deepmail-billing`
- `deepmail-notify`

Each crate has its own `README.md` and `src/README.md` explaining its current files and intended service responsibility.

## Current Implementation Status

- `crates/deepmail-common/` contains shared config, error, PostgreSQL, NATS, and protobuf generation utilities.
- Most service crates contain a single `src/main.rs` scaffold that:
  - loads shared service configuration
  - initializes JSON tracing
  - creates a PostgreSQL pool
  - creates a NATS JetStream context
  - waits for shutdown
- `crates/deepmail-gateway/src/main.rs` currently starts an Axum HTTP server scaffold instead of a gRPC scaffold.
- `frontend-beta/` contains the richest implemented surface today: routes, map views, graph components, API helpers, and contract tests.

## Important Architectural Notes

- The `.claude/` HTML documents are planning references, not executable contracts.
- The frontend prototype currently uses REST-style routes and local token storage, while the planned DeepMail platform description is GraphQL-first at the gateway. Treat that mismatch as active architectural drift to resolve during implementation.
- Some scaffolds are intentionally incomplete and exist only to prove workspace wiring and startup behavior.

## Where To Start

For backend work:

1. Read `crates/README.md`.
2. Read the target service crate README.
3. Read that crate's `src/README.md`.
4. Check `crates/deepmail-common/README.md` for shared utilities and generated protobuf modules.

For frontend work:

1. Read `frontend-beta/README.md`.
2. Read `frontend-beta/src/README.md`.
3. Continue into the relevant route, component, hook, or lib folder README.

For planning and design context:

1. Read `.claude/README.md`.
2. Then inspect `deepmail_architecture.html` and `deepmail_schema_complete.html`.
# beta
# beta
# beta
