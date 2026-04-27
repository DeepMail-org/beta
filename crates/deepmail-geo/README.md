# `deepmail-geo` Crate Guide

`deepmail-geo` is the geolocation enrichment service scaffold.

## Purpose

This crate is intended to resolve IPs to geographic, ASN, and hosting metadata and expose geo-analysis outputs.

## Contents

| Path | Purpose |
| --- | --- |
| `Cargo.toml` | Service manifest using shared Rust backend dependencies. |
| `src/` | Geolocation service source code. |

## Current Key Files

- `src/main.rs`
  - `main()`: standard bootstrap for config, tracing, PostgreSQL, NATS, and graceful shutdown.
