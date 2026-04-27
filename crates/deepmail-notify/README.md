# `deepmail-notify` Crate Guide

`deepmail-notify` is the tenant notification fan-out service scaffold.

## Purpose

This crate is intended to distribute real-time and outbound notification events across WebSocket, Slack, email, and webhook channels.

## Contents

| Path | Purpose |
| --- | --- |
| `Cargo.toml` | Service manifest using shared Rust backend dependencies. |
| `src/` | Notification service source code. |

## Current Key Files

- `src/main.rs`
  - `main()`: standard bootstrap for config, tracing, PostgreSQL, NATS, and graceful shutdown.
