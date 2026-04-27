# `deepmail-tenant` Crate Guide

`deepmail-tenant` is the tenant and organization management service scaffold.

## Purpose

This crate is intended to own tenant creation, membership roles, invite flows, subscription state, and tenant-level settings.

## Contents

| Path | Purpose |
| --- | --- |
| `Cargo.toml` | Service manifest using shared Rust backend dependencies. |
| `src/` | Tenant service source code. |

## Current Key Files

- `src/main.rs`
  - `main()`: standard bootstrap for config, tracing, PostgreSQL, NATS, and graceful shutdown.
