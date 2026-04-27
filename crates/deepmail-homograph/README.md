# `deepmail-homograph` Crate Guide

`deepmail-homograph` is the homograph and Unicode-risk service scaffold.

## Purpose

This crate is intended to detect IDN homographs, confusable characters, script mixing, and brand impersonation patterns.

## Contents

| Path | Purpose |
| --- | --- |
| `Cargo.toml` | Service manifest using shared Rust backend dependencies. |
| `src/` | Homograph-analysis service source code. |

## Current Key Files

- `src/main.rs`
  - `main()`: standard bootstrap for config, tracing, PostgreSQL, NATS, and graceful shutdown.
