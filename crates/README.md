# `crates` Folder Guide

This folder contains the Rust workspace members for DeepMail: one shared crate and one binary crate per service.

## Shared Crate

| Path | Purpose |
| --- | --- |
| `deepmail-common/` | Shared config helpers, error types, database helpers, NATS helpers, and generated protobuf modules used across backend services. |

## Edge Layer

| Path | Purpose |
| --- | --- |
| `deepmail-gateway/` | External gateway scaffold for HTTP, GraphQL, WebSocket, and service fan-out orchestration. |
| `deepmail-auth/` | Authentication service scaffold for credentials, OTP, JWT, refresh tokens, and lockout logic. |
| `deepmail-tenant/` | Tenant and organization lifecycle service scaffold. |

## Ingestion Layer

| Path | Purpose |
| --- | --- |
| `deepmail-ingest/` | Upload intake, validation, hashing, quarantine, and job publication scaffold. |
| `deepmail-parser/` | RFC 5322 parsing and MIME extraction scaffold. |
| `deepmail-otp-smtp/` | Transactional email delivery scaffold for OTP and notification mail. |

## Header Intelligence Layer

| Path | Purpose |
| --- | --- |
| `deepmail-header/` | Header forensics scaffold for SPF, DKIM, DMARC, and anomaly analysis. |
| `deepmail-dkim/` | Focused DKIM replay and signature validation scaffold. |

## Network Intelligence Layer

| Path | Purpose |
| --- | --- |
| `deepmail-geo/` | IP geolocation and hop enrichment scaffold. |
| `deepmail-ip/` | Reputation, TOR/VPN, CIDR feed, and blocklist scaffold. |
| `deepmail-intel/` | External threat-intelligence provider aggregation scaffold. |

## Content Analysis Layer

| Path | Purpose |
| --- | --- |
| `deepmail-ioc/` | IOC extraction and cross-email correlation scaffold. |
| `deepmail-homograph/` | IDN and homograph analysis scaffold. |
| `deepmail-body/` | Body URL, QR, redirect, and beacon analysis scaffold. |

## Sandbox Layer

| Path | Purpose |
| --- | --- |
| `deepmail-sandbox-url/` | URL detonation scaffold for browser isolation work. |
| `deepmail-sandbox-file/` | Static attachment analysis scaffold. |
| `deepmail-sandbox-dynamic/` | Dynamic malware detonation scaffold. |

## Core Intelligence Layer

| Path | Purpose |
| --- | --- |
| `deepmail-hashdb/` | Global hash intelligence and cache scaffold. |
| `deepmail-scoring/` | Threat scoring and signal aggregation scaffold. |
| `deepmail-graph/` | Graph intelligence and Neo4j integration scaffold. |

## Platform Layer

| Path | Purpose |
| --- | --- |
| `deepmail-report/` | Report export scaffold. |
| `deepmail-billing/` | Cost-pass-through billing scaffold. |
| `deepmail-notify/` | Tenant notification fan-out scaffold. |

## Common Structure

Most service crates currently contain:

- `Cargo.toml`: service-specific dependency declaration
- `src/`: Rust source for service startup and future modules
- `src/main.rs`: current bootstrap entrypoint

## How To Read This Folder

1. Start with the specific service crate README.
2. Then open its `src/README.md` for file-by-file guidance.
3. Use `deepmail-common/` to understand shared startup and integration primitives before changing service code.
