# `deepmail-header` Crate Guide

`deepmail-header` is the header forensics service scaffold.

## Purpose

This crate is intended to evaluate SPF, DKIM, DMARC, reply-path mismatches, message ID anomalies, and time-based header signals.

## Contents

| Path | Purpose |
| --- | --- |
| `Cargo.toml` | Service manifest using shared Rust backend dependencies. |
| `src/` | Header analysis service source code. |

## Current Key Files

- `src/main.rs`
  - `main()`: standard bootstrap for config, tracing, PostgreSQL, NATS, and graceful shutdown.
