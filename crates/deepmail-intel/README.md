# `deepmail-intel` Crate Guide

`deepmail-intel` is the external provider enrichment service scaffold.

## Purpose

This crate is intended to broker lookups to providers such as AbuseIPDB, VirusTotal, IPInfo, GreyNoise, and Shodan while managing caching and circuit-breaking.

## Contents

| Path | Purpose |
| --- | --- |
| `Cargo.toml` | Service manifest using shared Rust backend dependencies. |
| `src/` | Intelligence aggregation service source code. |

## Current Key Files

- `src/main.rs`
  - `main()`: standard bootstrap for config, tracing, PostgreSQL, NATS, and graceful shutdown.
