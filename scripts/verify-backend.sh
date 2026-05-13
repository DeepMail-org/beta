#!/usr/bin/env bash
set -uo pipefail

# DeepMail backend verification matrix runner.
#
# Usage:
#   bash scripts/verify-backend.sh
#
# Optional environment variables:
#   GATEWAY_URL=http://127.0.0.1:3001
#   ACCESS_TOKEN=<jwt>
#   TEST_EMAIL_ID=<uuid>
#   GRAPH_IOC_VALUE=<string>
#   GRAPH_IOC_TYPE=<string>

GATEWAY_URL="${GATEWAY_URL:-http://127.0.0.1:3001}"
ACCESS_TOKEN="${ACCESS_TOKEN:-}"
TEST_EMAIL_ID="${TEST_EMAIL_ID:-}"
GRAPH_IOC_VALUE="${GRAPH_IOC_VALUE:-example.com}"
GRAPH_IOC_TYPE="${GRAPH_IOC_TYPE:-domain}"

PASS_COUNT=0
FAIL_COUNT=0
WARN_COUNT=0

pass() {
  PASS_COUNT=$((PASS_COUNT + 1))
  printf "[PASS] %s\n" "$1"
}

fail() {
  FAIL_COUNT=$((FAIL_COUNT + 1))
  printf "[FAIL] %s\n" "$1"
}

warn() {
  WARN_COUNT=$((WARN_COUNT + 1))
  printf "[WARN] %s\n" "$1"
}

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    fail "Missing required command: $1"
    return 1
  fi
  return 0
}

check_port() {
  local name="$1"
  local port="$2"
  if ss -ltn | rg -q ":${port}\\b"; then
    pass "${name} listening on :${port}"
  else
    fail "${name} not listening on :${port}"
  fi
}

check_http_200() {
  local name="$1"
  local url="$2"
  if curl -fsS "$url" >/dev/null 2>&1; then
    pass "${name} reachable (${url})"
  else
    fail "${name} unreachable (${url})"
  fi
}

check_gateway_status() {
  local code
  code="$(curl -sS -o /tmp/deepmail_gateway_health.json -w "%{http_code}" "${GATEWAY_URL}/health" 2>/dev/null || true)"
  if [[ "$code" == "200" ]]; then
    pass "gateway /health returns 200"
  else
    fail "gateway /health returns ${code:-<none>}"
    return
  fi

  if rg -q '"status"\s*:\s*"(ok|degraded|down)"' /tmp/deepmail_gateway_health.json; then
    pass "gateway /health includes status field"
  else
    fail "gateway /health missing expected status field"
  fi

  local svc
  for svc in auth scoring report ioc graph tenant billing notify; do
    if rg -q "\"${svc}\"\s*:" /tmp/deepmail_gateway_health.json; then
      pass "gateway /health reports ${svc}"
    else
      fail "gateway /health missing ${svc}"
    fi
  done
}

check_auth_open_routes() {
  local endpoint code
  for endpoint in register login verify-otp refresh; do
    code="$(curl -sS -o /dev/null -w "%{http_code}" -X POST "${GATEWAY_URL}/auth/${endpoint}" -H 'content-type: application/json' -d '{}' 2>/dev/null || true)"
    if [[ "$code" != "401" ]]; then
      pass "/auth/${endpoint} does not require Bearer token (http ${code:-<none>})"
    else
      fail "/auth/${endpoint} unexpectedly requires Bearer token"
    fi
  done
}

check_protected_unauthorized() {
  local code
  code="$(curl -sS -o /tmp/deepmail_tenant_unauth.json -w "%{http_code}" "${GATEWAY_URL}/api/v1/tenant" 2>/dev/null || true)"
  if [[ "$code" == "401" ]]; then
    pass "protected route rejects missing token (401)"
  else
    fail "protected route expected 401, got ${code:-<none>}"
  fi
}

check_authorized_endpoint() {
  local name="$1"
  local method="$2"
  local path="$3"
  local body="${4:-}"
  local code

  if [[ -z "$ACCESS_TOKEN" ]]; then
    warn "Skipping ${name} (ACCESS_TOKEN not set)"
    return
  fi

  if [[ -n "$body" ]]; then
    code="$(curl -sS -o /dev/null -w "%{http_code}" -X "$method" "${GATEWAY_URL}${path}" -H "Authorization: Bearer ${ACCESS_TOKEN}" -H 'content-type: application/json' -d "$body" 2>/dev/null || true)"
  else
    code="$(curl -sS -o /dev/null -w "%{http_code}" -X "$method" "${GATEWAY_URL}${path}" -H "Authorization: Bearer ${ACCESS_TOKEN}" 2>/dev/null || true)"
  fi

  if [[ "$code" =~ ^2|404$ ]]; then
    pass "${name} returned acceptable code ${code}"
  else
    fail "${name} returned unexpected code ${code:-<none>}"
  fi
}

main() {
  require_cmd ss || exit 1
  require_cmd curl || exit 1
  require_cmd rg || exit 1

  echo "== Infra checks =="
  check_port "PostgreSQL" "5432"
  check_port "Redis" "6379"
  check_port "NATS" "4222"
  check_port "NATS monitor" "8222"
  check_port "MinIO" "9000"
  check_http_200 "NATS /healthz" "http://127.0.0.1:8222/healthz"
  check_http_200 "MinIO liveness" "http://127.0.0.1:9000/minio/health/live"

  echo
  echo "== Service port checks =="
  check_port "deepmail-auth" "50051"
  check_port "deepmail-tenant" "50052"
  check_port "deepmail-ioc" "50057"
  check_port "deepmail-scoring" "50063"
  check_port "deepmail-graph" "50064"
  check_port "deepmail-report" "50065"
  check_port "deepmail-notify" "50066"
  check_port "deepmail-billing" "50067"
  check_port "deepmail-ingest" "8090"
  check_port "deepmail-gateway" "3001"

  echo
  echo "== Gateway behavior checks =="
  check_gateway_status
  check_auth_open_routes
  check_protected_unauthorized

  echo
  echo "== Authenticated route checks (requires ACCESS_TOKEN) =="
  check_authorized_endpoint "tenant profile" "GET" "/api/v1/tenant"
  check_authorized_endpoint "tenant usage" "GET" "/api/v1/tenant/usage"
  check_authorized_endpoint "notify get config" "GET" "/api/v1/notify/config"
  check_authorized_endpoint "notify upsert config" "PUT" "/api/v1/notify/config" '{"webhook_url":"","webhook_secret":"","smtp_enabled":false,"min_severity":"medium"}'

  if [[ -n "$TEST_EMAIL_ID" ]]; then
    check_authorized_endpoint "email status" "GET" "/api/v1/emails/${TEST_EMAIL_ID}/status"
    check_authorized_endpoint "email score" "GET" "/api/v1/emails/${TEST_EMAIL_ID}/score"
    check_authorized_endpoint "email report" "GET" "/api/v1/emails/${TEST_EMAIL_ID}/report"
    check_authorized_endpoint "email iocs" "GET" "/api/v1/emails/${TEST_EMAIL_ID}/iocs"
    check_authorized_endpoint "email graph" "GET" "/api/v1/emails/${TEST_EMAIL_ID}/graph?ioc_value=${GRAPH_IOC_VALUE}&ioc_type=${GRAPH_IOC_TYPE}&depth=2"
  else
    warn "Skipping email-by-id checks (TEST_EMAIL_ID not set)"
  fi

  echo
  echo "== GraphQL checks (requires ACCESS_TOKEN) =="
  if [[ -z "$ACCESS_TOKEN" ]]; then
    warn "Skipping GraphQL checks (ACCESS_TOKEN not set)"
  else
    local gql_code
    gql_code="$(curl -sS -o /dev/null -w "%{http_code}" -X POST "${GATEWAY_URL}/graphql" -H "Authorization: Bearer ${ACCESS_TOKEN}" -H 'content-type: application/json' -d '{"query":"query{tenantUsage{billingPeriod totalPaise eventCount}}"}' 2>/dev/null || true)"
    if [[ "$gql_code" == "200" ]]; then
      pass "GraphQL tenantUsage query returned 200"
    else
      fail "GraphQL tenantUsage query returned ${gql_code:-<none>}"
    fi
  fi

  echo
  echo "== Summary =="
  echo "PASS: ${PASS_COUNT}"
  echo "FAIL: ${FAIL_COUNT}"
  echo "WARN: ${WARN_COUNT}"

  if [[ "$FAIL_COUNT" -gt 0 ]]; then
    exit 1
  fi
  exit 0
}

main "$@"
