# `deepmail-parser` Crate Guide

`deepmail-parser` is the RFC 5322 and MIME parsing service scaffold.

## Purpose

This crate is intended to parse uploaded email samples, extract headers and bodies, reconstruct received hops, and enumerate attachments.

## Contents

| Path | Purpose |
| --- | --- |
| `Cargo.toml` | Service manifest using shared Rust backend dependencies. |
| `src/` | Parser service source code. |

## Current Key Files

- `src/main.rs`
  - `main()`: standard bootstrap for config, tracing, PostgreSQL, NATS, and graceful shutdown.
