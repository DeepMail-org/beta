# `deepmail-body` Crate Guide

`deepmail-body` is the body and URL analysis service scaffold.

## Purpose

This crate is intended to extract links, QR redirects, shortener chains, encoded URLs, and behavioral indicators from email bodies.

## Contents

| Path | Purpose |
| --- | --- |
| `Cargo.toml` | Service manifest using shared Rust backend dependencies. |
| `src/` | Body-analysis service source code. |

## Current Key Files

- `src/main.rs`
  - `main()`: standard bootstrap for config, tracing, PostgreSQL, NATS, and graceful shutdown.
