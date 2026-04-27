# `deepmail-sandbox-file` Crate Guide

`deepmail-sandbox-file` is the static attachment analysis scaffold.

## Purpose

This crate is intended to orchestrate file inspection tools such as YARA, oletools, pdfid, binwalk, and metadata extraction workflows.

## Contents

| Path | Purpose |
| --- | --- |
| `Cargo.toml` | Service manifest using shared Rust backend dependencies. |
| `src/` | Static file-analysis service source code. |

## Current Key Files

- `src/main.rs`
  - `main()`: standard bootstrap for config, tracing, PostgreSQL, NATS, and graceful shutdown.
