# `deepmail-billing` Crate Guide

`deepmail-billing` is the cost-pass-through billing service scaffold.

## Purpose

This crate is intended to meter internal events, snapshot provider pricing, and generate tenant billing records without adding markup.

## Contents

| Path | Purpose |
| --- | --- |
| `Cargo.toml` | Service manifest using shared Rust backend dependencies. |
| `src/` | Billing service source code. |

## Current Key Files

- `src/main.rs`
  - `main()`: standard bootstrap for config, tracing, PostgreSQL, NATS, and graceful shutdown.
