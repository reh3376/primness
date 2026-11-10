#!/bin/sh
# D2 part 1 end to end (docs/d2_falsifiable_prediction.md, docs/d2_multiple_fields.md):
#
#   1. d2-fields: three rotations f + v*g of every instrument polynomial,
#      chosen by a rule fixed before measuring (data/instruments/d2/,
#      data/results/d2/fields.csv);
#   2. d1-tail: their root counts at every prime up to 2^28
#      (data/results/d2/root_tail.json);
#   3. d1-measure per rotated field at 2^14..2^28 over the D1 region, with the
#      per-value dump (data/results/d2/<field>.*, data/dumps/d2/<field>.values.csv);
#   4. the instruments' own dumps, from scripts/d5_run_all.sh when missing;
#   5. scripts/d2_joint.py: joint smoothness of the fields and the rational
#      side against the product of their predictions, with the control.
#
#   scripts/d2_run_all.sh [--force]
#
# --force redoes steps 1 to 3. The dumps are gitignored, about 315 MB each.
# Env: D1_MEASURE (binary), A_MAX, B_MAX.
set -eu
cd "$(dirname "$0")/.."
FORCE=0
[ "${1:-}" = "--force" ] && FORCE=1
MEASURE=${D1_MEASURE:-target/release/d1-measure}
A_MAX=${A_MAX:-3000}; B_MAX=${B_MAX:-1500}
BOUNDS=2^14,2^16,2^18,2^20,2^24,2^28
if [ "$FORCE" -eq 1 ] || [ ! -f data/results/d2/fields.csv ]; then
  target/release/d2-fields --a-max "$A_MAX" --b-max "$B_MAX"
fi
if [ "$FORCE" -eq 1 ] || [ ! -f data/results/d2/root_tail.json ]; then
  target/release/d1-tail --instruments data/instruments/d2 --out data/results/d2/root_tail.json
fi
mkdir -p data/dumps/d2
log=$(mktemp)
trap 'rm -f "$log"' EXIT
for poly in data/instruments/d2/*/cado.poly; do
  id=$(basename "$(dirname "$poly")")
  dump="data/dumps/d2/$id.values.csv"
  result="data/results/d2/$id.json"
  if [ "$FORCE" -eq 0 ] && [ -f "$dump" ] && [ -f "$result" ]; then echo "$id: measured"; continue; fi
  echo "=== $id: measurement at $BOUNDS $(date -u +%H:%M:%SZ)"
  "$MEASURE" --instance "$id" --source unspecified --a-max "$A_MAX" --b-max "$B_MAX" --bounds "$BOUNDS" \
    --out "$result" --dump "$dump" "$poly" > "$log"
  grep '^evaluated' "$log" || true
done
for poly in data/instruments/d1-*/cado.poly; do
  id=$(basename "$(dirname "$poly")")
  if [ ! -f "data/dumps/$id.values.csv" ]; then sh scripts/d5_run_all.sh; break; fi
done
python3 -I scripts/d2_joint.py
