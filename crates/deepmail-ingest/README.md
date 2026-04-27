# `deepmail-ingest` Crate Guide

`deepmail-ingest` is the email upload and quarantine service scaffold.

## Purpose

This crate is intended to accept `.eml` and `.msg` uploads, validate them, compute hashes, quarantine files, persist metadata, and publish ingest jobs.

## Contents

| Path | Purpose |
| --- | --- |
| `Cargo.toml` | Service manifest using shared Rust backend dependencies. |
| `src/` | Ingest service source code. |

## Current Key Files

- `src/main.rs`
  - `main()`: standard bootstrap for config, tracing, PostgreSQL, NATS, and graceful shutdown.
