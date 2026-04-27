# `deepmail-auth/src` Folder Guide

This folder contains the current authentication service entrypoint.

## Files

| File | Purpose |
| --- | --- |
| `main.rs` | Service bootstrap for config, tracing, database, NATS, and graceful shutdown. |

## Function Index

- `main()`: initializes the auth service process and validates that shared infrastructure dependencies are reachable.

## Editing Guidance

- Add auth handlers, OTP flows, JWT logic, and persistence modules here as the service grows.
