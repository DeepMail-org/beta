# `deepmail-ip` Crate Guide

`deepmail-ip` is the IP reputation and blocklist service scaffold.

## Purpose

This crate is intended to track TOR exit nodes, VPN/proxy ranges, botnet feeds, CIDR blocklists, and IP-specific enrichment signals.

## Contents

| Path | Purpose |
| --- | --- |
| `Cargo.toml` | Service manifest using shared Rust backend dependencies. |
| `src/` | IP-intelligence service source code. |

## Current Key Files

- `src/main.rs`
  - `main()`: standard bootstrap for config, tracing, PostgreSQL, NATS, and graceful shutdown.
