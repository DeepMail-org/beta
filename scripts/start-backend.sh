#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOG_DIR="${ROOT_DIR}/.runlogs"
mkdir -p "${LOG_DIR}"

SERVICES=(
  "deepmail-otp-smtp"
  "deepmail-auth"
  "deepmail-tenant"
  "deepmail-billing"
  "deepmail-ingest"
  "deepmail-scoring"
  "deepmail-ioc"
  "deepmail-graph"
  "deepmail-report"
  "deepmail-notify"
  "deepmail-gateway"
)

echo "[info] starting DeepMail backend services"

for svc in "${SERVICES[@]}"; do
  pid_file="${LOG_DIR}/${svc}.pid"
  if [[ -f "${pid_file}" ]]; then
    old_pid="$(cat "${pid_file}")"
    if kill -0 "${old_pid}" 2>/dev/null; then
      echo "[warn] ${svc} already running with PID ${old_pid}, skipping"
      continue
    fi
  fi

  echo "[info] starting ${svc}"
  nohup cargo run -p "${svc}" >"${LOG_DIR}/${svc}.log" 2>&1 &
  echo $! >"${pid_file}"
  sleep 1
done

echo "[info] done. logs at ${LOG_DIR}"
echo "[info] run verification: bash scripts/verify-backend.sh"
