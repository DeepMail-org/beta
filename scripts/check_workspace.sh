#!/usr/bin/env bash
# check_workspace.sh
#
# Runs `cargo check --workspace` and tees the full output to a log file
# under /tmp for quick inspection.

set -uo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
WORKSPACE_ROOT="$(cd -- "${SCRIPT_DIR}/.." &> /dev/null && pwd)"
cd "${WORKSPACE_ROOT}"

cargo check --workspace 2>&1 | tee /tmp/deepmail_check.log
echo "Exit code: $?"
