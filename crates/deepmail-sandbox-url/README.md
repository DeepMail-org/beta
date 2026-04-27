# `deepmail-sandbox-url` Crate Guide

`deepmail-sandbox-url` is the browser-isolated URL sandbox scaffold.

## Purpose

This crate is intended to queue URL detonations, launch browser-isolated analysis jobs, and persist network, DOM, and screenshot evidence.

## Contents

| Path | Purpose |
| --- | --- |
| `Cargo.toml` | Service manifest using shared Rust backend dependencies. |
| `src/` | URL sandbox service source code. |

## Current Key Files

- `src/main.rs`
  - `main()`: standard bootstrap for config, tracing, PostgreSQL, NATS, and graceful shutdown.
