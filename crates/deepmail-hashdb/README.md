# `deepmail-hashdb` Crate Guide

`deepmail-hashdb` is the hash intelligence and verdict-cache service scaffold.

## Purpose

This crate is intended to store global hash records, short-circuit duplicate analyses, and coordinate fuzzy-hash clustering.

## Contents

| Path | Purpose |
| --- | --- |
| `Cargo.toml` | Service manifest using shared Rust backend dependencies. |
| `src/` | Hash intelligence service source code. |

## Current Key Files

- `src/main.rs`
  - `main()`: standard bootstrap for config, tracing, PostgreSQL, NATS, and graceful shutdown.
