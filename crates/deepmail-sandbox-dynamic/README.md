# `deepmail-sandbox-dynamic` Crate Guide

`deepmail-sandbox-dynamic` is the dynamic detonation service scaffold.

## Purpose

This crate is intended to orchestrate dynamic malware analysis jobs, virtualized execution, and behavioral result collection.

## Contents

| Path | Purpose |
| --- | --- |
| `Cargo.toml` | Service manifest using shared Rust backend dependencies. |
| `src/` | Dynamic sandbox service source code. |

## Current Key Files

- `src/main.rs`
  - `main()`: standard bootstrap for config, tracing, PostgreSQL, NATS, and graceful shutdown.
