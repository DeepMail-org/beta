# DeepMail

DeepMail is a production-grade email threat intelligence SaaS platform. Users upload `.eml` or `.msg` samples, 23 backend microservices analyze them across multiple intelligence layers (header forensics, DKIM replay detection, geolocation, IP reputation, threat intel, IOC extraction, homograph detection, body analysis, URL/file/dynamic sandboxing, ML scoring), and the frontend presents threat reports, relationship graphs, maps, and sandbox outputs.

## Architecture

```
Internet → [Load Balancer :443] → deepmail-gateway :3001
                                        │
                    ┌───────────────────────────────────────┐
                    │         Internal Network Only          │
                    │                                        │
                    │  auth:50051  scoring:50063  report:50065│
                    │  tenant:50052  ioc:50057  graph:50064  │
                    │  billing:50067  notify:50066            │
                    │  ingest:8090                            │
                    │                                        │
                    │  [NATS :4222] [Redis :6379]            │
                    │  [PostgreSQL :5432] [Neo4j :7687]      │
                    │  [MinIO :9000]                         │
                    └───────────────────────────────────────┘
```

Only `deepmail-gateway` is exposed to the public internet. All other services communicate internally via gRPC, NATS JetStream, and direct database connections.

## Top-Level Layout

| Path | Purpose |
| --- | --- |
| `crates/` | Rust workspace: shared library + 23 microservice binaries |
| `services/deepmail-ml/` | Python ML inference service (FastAPI) |
| `migrations/` | PostgreSQL migrations per service (22 databases) |
| `frontend-beta/` | Next.js prototype frontend |
| `scripts/` | Workspace build/start/stop helper scripts |
| `Cargo.toml` | Rust workspace manifest with pinned dependency versions |

## Backend Services

| Service | Port | Protocol | Responsibility |
|---------|------|----------|----------------|
| **deepmail-gateway** | 3001 | HTTP | API gateway, JWT auth, rate limiting, REST→gRPC, GraphQL |
| deepmail-auth | 50051 | gRPC | Authentication, JWT issuance, OTP |
| deepmail-tenant | 8081 | HTTP | Tenant management, Razorpay webhooks |
| deepmail-ingest | 8090 | HTTP | File upload, validation, S3 quarantine |
| deepmail-parser | — | NATS | EML/MSG parsing |
| deepmail-otp-smtp | — | NATS | OTP email delivery |
| deepmail-header | — | NATS | Header forensics (SPF/DKIM/DMARC) |
| deepmail-dkim | — | NATS | DKIM replay detection |
| deepmail-geo | — | NATS | IP geolocation, Tor/VPN detection |
| deepmail-ip | — | NATS | IP reputation scoring |
| deepmail-intel | — | NATS | Threat intel (VirusTotal, AbuseIPDB, Shodan, GreyNoise) |
| deepmail-ioc | — | NATS | IOC extraction & campaign clustering |
| deepmail-homograph | — | NATS | IDN homograph detection |
| deepmail-body | — | NATS | HTML/text body analysis, phishing scoring |
| deepmail-sandbox-url | — | NATS | URL sandboxing (headless Chrome in Docker) |
| deepmail-sandbox-file | — | NATS | Static file analysis (YARA, strings, binwalk) |
| deepmail-sandbox-dynamic | — | NATS | Dynamic analysis (CAPEv2 integration) |
| deepmail-hashdb | — | NATS | File hash reputation |
| deepmail-scoring | 50063 | gRPC | Weighted threat score aggregation |
| deepmail-graph | 50064 | gRPC | Neo4j relationship graph |
| deepmail-report | 50065 | gRPC | Report generation (JSON/HTML → S3) |
| deepmail-billing | 50067 | gRPC+HTTP | Usage metering, invoicing, Razorpay |
| deepmail-notify | 50066 | gRPC | Alerts (email, webhook, WebSocket) |
| deepmail-ml | 8050 | HTTP | Python ML inference (phishing classifier) |

## Build Status

All 24 Rust crates compile with zero errors. Python ML service passes 8/8 tests.

```bash
SQLX_OFFLINE=true cargo build --workspace --release  # ✅ exit 0
```

## Quick Start

### Prerequisites

- Rust 1.75+ with `cargo`
- PostgreSQL 15+
- Redis 7+
- NATS 2.10+ with JetStream enabled
- MinIO or S3-compatible storage
- Docker (for URL sandbox)
- `sqlx-cli`: `cargo install sqlx-cli`

### Setup

```bash
# 1. Create all databases
for svc in auth tenant ingest parser header dkim geo ip intel ioc homograph body sandbox-url sandbox-file sandbox-dynamic hashdb scoring graph report billing notify gateway; do
  sqlx database create --database-url "postgres://deepmail:deepmailpw@localhost:5432/deepmail_${svc//-/_}"
done

# 2. Run all migrations
for svc in auth tenant ingest parser header dkim geo ip intel ioc homograph body sandbox-url sandbox-file sandbox-dynamic hashdb scoring graph report billing notify gateway; do
  sqlx migrate run --source migrations/deepmail-$svc \
    --database-url "postgres://deepmail:deepmailpw@localhost:5432/deepmail_${svc//-/_}"
done

# 3. Build
SQLX_OFFLINE=true cargo build --workspace --release

# 4. Start (set env vars first — see .env section below)
./target/release/deepmail-gateway &
./target/release/deepmail-auth &
# ... start all services
```

### Verify

```bash
curl http://localhost:3001/health
# {"status":"ok","services":{"auth":true,"scoring":true,...},"timestamp":"..."}
```

## Environment Variables

### Gateway (required)

```env
DATABASE_URL=postgres://deepmail:deepmailpw@localhost:5432/deepmail_gateway
INGEST_DATABASE_URL=postgres://deepmail:deepmailpw@localhost:5432/deepmail_ingest
REDIS_URL=redis://localhost:6379
HTTP_PORT=3001
CORS_ALLOW_ORIGIN=https://your-domain.com
GRAPHIQL_ENABLED=false
RATE_LIMIT_PER_MINUTE=100
MAX_UPLOAD_MB=25
AUTH_GRPC_URL=http://127.0.0.1:50051
SCORING_GRPC_URL=http://127.0.0.1:50063
REPORT_GRPC_URL=http://127.0.0.1:50065
IOC_GRPC_URL=http://127.0.0.1:50057
GRAPH_GRPC_URL=http://127.0.0.1:50064
TENANT_GRPC_URL=http://127.0.0.1:50052
BILLING_GRPC_URL=http://127.0.0.1:50067
NOTIFY_GRPC_URL=http://127.0.0.1:50066
INGEST_HTTP_URL=http://127.0.0.1:8090
```

### External API Keys

| Variable | Service | Provider |
|----------|---------|----------|
| `DEEPMAIL_INTEL_VIRUSTOTAL_API_KEY` | intel | [VirusTotal](https://www.virustotal.com/gui/my-apikey) |
| `DEEPMAIL_INTEL_ABUSEIPDB_API_KEY` | intel | [AbuseIPDB](https://www.abuseipdb.com/account/api) |
| `DEEPMAIL_INTEL_GREYNOISE_API_KEY` | intel | [GreyNoise](https://viz.greynoise.io/account/api-key) |
| `DEEPMAIL_INTEL_SHODAN_API_KEY` | intel | [Shodan](https://account.shodan.io/) |
| `DEEPMAIL_BILLING_RAZORPAY_KEY_ID` | billing | [Razorpay](https://dashboard.razorpay.com/app/keys) |
| `DEEPMAIL_BILLING_RAZORPAY_KEY_SECRET` | billing | Razorpay |
| `DEEPMAIL_AUTH_JWT_PRIVATE_KEY_PEM` | auth | `openssl genrsa -out private.pem 2048` |
| `DEEPMAIL_AUTH_JWT_PUBLIC_KEY_PEM` | auth | `openssl rsa -in private.pem -pubout -out public.pem` |

## API Endpoints

### Public (no auth)
- `POST /auth/register` — Create account
- `POST /auth/login` — Login (returns OTP session)
- `POST /auth/verify-otp` — Verify OTP → access token
- `POST /auth/refresh` — Refresh access token
- `GET /health` — Service health aggregator

### Protected (Bearer JWT required)
- `POST /api/v1/emails/upload` — Upload .eml/.msg for analysis
- `GET /api/v1/emails/{id}/status` — Pipeline progress
- `GET /api/v1/emails/{id}/score` — Threat score
- `GET /api/v1/emails/{id}/report` — Full report
- `GET /api/v1/emails/{id}/iocs` — Extracted IOCs
- `GET /api/v1/emails/{id}/graph` — Relationship graph
- `GET /api/v1/tenant` — Tenant info
- `POST /api/v1/tenant/invite` — Invite member
- `GET /api/v1/tenant/usage` — Billing usage
- `GET /api/v1/notify/config` — Notification config
- `PUT /api/v1/notify/config` — Update notification config
- `POST /graphql` — GraphQL endpoint

## Development

```bash
# Check all services compile
SQLX_OFFLINE=true cargo build --workspace

# Run a specific service
DATABASE_URL="postgres://..." cargo run -p deepmail-gateway

# Run ML tests
cd services/deepmail-ml && python -m pytest tests/ -q
```

## License

UNLICENSED — Proprietary software.
