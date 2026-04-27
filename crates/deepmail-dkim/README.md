# `deepmail-dkim` Crate Guide

`deepmail-dkim` is the dedicated DKIM analysis service scaffold.

## Purpose

This crate is intended to recompute body hashes, validate signatures, detect replay conditions, and evaluate key rotation evidence.

## Contents

| Path | Purpose |
| --- | --- |
| `Cargo.toml` | Service manifest using shared Rust backend dependencies. |
| `src/` | DKIM service source code. |

## Current Key Files

- `src/main.rs`
  - `main()`: standard bootstrap for config, tracing, PostgreSQL, NATS, and graceful shutdown.
