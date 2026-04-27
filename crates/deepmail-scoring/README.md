# `deepmail-scoring` Crate Guide

`deepmail-scoring` is the threat-score aggregation service scaffold.

## Purpose

This crate is intended to merge signals from all analysis stages, normalize them into final threat scores, and publish final verdicts.

## Contents

| Path | Purpose |
| --- | --- |
| `Cargo.toml` | Service manifest using shared Rust backend dependencies. |
| `src/` | Scoring service source code. |

## Current Key Files

- `src/main.rs`
  - `main()`: standard bootstrap for config, tracing, PostgreSQL, NATS, and graceful shutdown.
