#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOG_DIR="${ROOT_DIR}/.runlogs"

if [[ ! -d "${LOG_DIR}" ]]; then
  echo "[info] no .runlogs directory found"
  exit 0
fi

echo "[info] stopping tracked backend services"

shopt -s nullglob
for pid_file in "${LOG_DIR}"/*.pid; do
  pid="$(cat "${pid_file}")"
  svc="$(basename "${pid_file}" .pid)"
  if kill -0 "${pid}" 2>/dev/null; then
    echo "[info] stopping ${svc} (pid ${pid})"
    kill "${pid}" || true
  else
    echo "[warn] ${svc} pid ${pid} not running"
  fi
done

sleep 2

for pid_file in "${LOG_DIR}"/*.pid; do
  pid="$(cat "${pid_file}")"
  svc="$(basename "${pid_file}" .pid)"
  if kill -0 "${pid}" 2>/dev/null; then
    echo "[warn] force stopping ${svc} (pid ${pid})"
    kill -9 "${pid}" || true
  fi
done

echo "[info] backend stop completed"
