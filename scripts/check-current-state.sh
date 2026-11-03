#!/bin/sh
# Gate for auto-pr. The requirements are the const values of the JSON Schema
# named by the head document. Version current-state/1 lives at
# schemas/current-state/1.json. This wrapper keeps the workflow call:
#   sh scripts/check-current-state.sh [base-ref] [head-ref]
#   sh scripts/check-current-state.sh --print-section [head-ref]
set -eu
cd "$(dirname "$0")/.."
exec python3 scripts/check_current_state.py "$@"
