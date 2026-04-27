# `deepmail-ioc` Crate Guide

`deepmail-ioc` is the indicator extraction and correlation service scaffold.

## Purpose

This crate is intended to extract IPs, domains, URLs, hashes, and email indicators, then build cross-email relationships and clustering hints.

## Contents

| Path | Purpose |
| --- | --- |
| `Cargo.toml` | Service manifest using shared Rust backend dependencies. |
| `src/` | IOC service source code. |

## Current Key Files

- `src/main.rs`
  - `main()`: standard bootstrap for config, tracing, PostgreSQL, NATS, and graceful shutdown.
