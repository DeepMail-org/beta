# `deepmail-gateway/src` Folder Guide

This folder contains the current gateway runtime bootstrap.

## Files

| File | Purpose |
| --- | --- |
| `main.rs` | Starts the gateway scaffold, HTTP listener, middleware stack, and graceful shutdown flow. |

## Function Index

- `main()`: boots tracing, validates required configuration, builds the empty Axum router, and serves HTTP traffic.
- `shutdown_signal()`: waits for process shutdown and emits a shutdown log.

## Editing Guidance

- Keep external API concerns in this folder.
- If GraphQL, REST upload, or WebSocket proxying is implemented later, document the added modules here.
