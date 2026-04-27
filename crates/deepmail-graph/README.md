# `deepmail-graph` Crate Guide

`deepmail-graph` is the entity-graph service scaffold.

## Purpose

This crate is intended to map emails, indicators, campaigns, infrastructure, and actor relationships into a graph store and expose traversal operations.

## Contents

| Path | Purpose |
| --- | --- |
| `Cargo.toml` | Service manifest using shared Rust backend dependencies. |
| `src/` | Graph service source code. |

## Current Key Files

- `src/main.rs`
  - `main()`: standard bootstrap for config, tracing, PostgreSQL, NATS, and graceful shutdown.
