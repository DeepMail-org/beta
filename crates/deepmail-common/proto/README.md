# `deepmail-common/proto` Folder Guide

This folder contains the protobuf source contracts used for DeepMail gRPC communication.

## Files

| File | Purpose |
| --- | --- |
| `auth.proto` | Authentication and session contracts. |
| `billing.proto` | Metering, pricing, and invoice contracts. |
| `dkim.proto` | DKIM analysis and replay-detection contracts. |
| `geo.proto` | Geolocation and hop-enrichment contracts. |
| `graph.proto` | Entity graph and traversal contracts. |
| `hashdb.proto` | Hash lookup and verdict-cache contracts. |
| `header.proto` | Header forensics contracts. |
| `homograph.proto` | Homograph and Unicode-risk contracts. |
| `intel.proto` | External intelligence provider contracts. |
| `ioc.proto` | IOC extraction and correlation contracts. |
| `ml.proto` | ML inference and augmentation contracts. |
| `notify.proto` | Notification fan-out contracts. |
| `report.proto` | Report generation and export contracts. |
| `sandbox.proto` | URL, static-file, and dynamic sandbox contracts. |
| `scoring.proto` | Threat scoring contracts. |

## Build Flow

- `../build.rs` compiles these files with `tonic_build`.
- Generated Rust modules are re-exported from `deepmail_common::proto`.
- Service crates should import generated types from the shared crate instead of generating protobuf code independently.

## Editing Guidance

- Keep package names and RPC signatures consistent across services.
- When adding a new contract, update `build.rs`, `src/lib.rs`, and this README together.
