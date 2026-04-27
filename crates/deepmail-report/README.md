# `deepmail-report` Crate Guide

`deepmail-report` is the report generation service scaffold.

## Purpose

This crate is intended to generate PDF, JSON, CSV, and STIX exports and coordinate report storage and delivery.

## Contents

| Path | Purpose |
| --- | --- |
| `Cargo.toml` | Service manifest using shared Rust backend dependencies. |
| `src/` | Report service source code. |

## Current Key Files

- `src/main.rs`
  - `main()`: standard bootstrap for config, tracing, PostgreSQL, NATS, and graceful shutdown.
