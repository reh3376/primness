#!/bin/sh
# Obstacle D, the closing checks end to end (docs/obstacle_d_verdict.md):
#
#   1. the five irreducible toy quartics of the structure ladder, Galois groups
#      A4, S4, C4, D4 and V4 (data/instruments/galois/): root counts at every
#      prime up to 2^28 (d1-tail, data/results/galois/root_tail.json) and a
#      measurement at 2^14..2^28 over the D1 region with the per-value dump
#      (data/results/galois/<k>.*, data/dumps/galois/<k>.values.csv);
#   2. scripts/d_checks.py: the per-value checks (d-checks) over the ten
#      instruments, their rational sides, the thirty rotated fields of D2 and
#      the quartics; the pooled statistics of D1, D2 part 1 and D5; the gap
#      against the digits of N; the quartics against the model.
#
#   scripts/d_checks_run_all.sh [--force]
#
# Needs the instrument and D2 dumps (scripts/d5_run_all.sh, scripts/d2_run_all.sh).
# --force redoes step 1. The dumps are gitignored, about 300 MB each.
set -eu
cd "$(dirname "$0")/.."
FORCE=0
[ "${1:-}" = "--force" ] && FORCE=1
MEASURE=${D1_MEASURE:-target/release/d1-measure}
BOUNDS=2^14,2^16,2^18,2^20,2^24,2^28
cargo build --release --bin d1-measure --bin d1-tail --bin d-checks
if [ "$FORCE" -eq 1 ] || [ ! -f data/results/galois/root_tail.json ]; then
  mkdir -p data/results/galois
  target/release/d1-tail --instruments data/instruments/galois --out data/results/galois/root_tail.json
fi
mkdir -p data/dumps/galois
for poly in data/instruments/galois/*/cado.poly; do
  id=$(basename "$(dirname "$poly")")
  dump="data/dumps/galois/$id.values.csv"
  result="data/results/galois/$id.json"
  if [ "$FORCE" -eq 0 ] && [ -f "$dump" ] && [ -f "$result" ]; then echo "$id: measured"; continue; fi
  echo "=== $id: measurement at $BOUNDS"
  "$MEASURE" --instance "$id" --source unspecified --a-max 3000 --b-max 1500 --bounds "$BOUNDS" --no-hw-sample \
    --out "$result" --dump "$dump" "$poly" | grep '^evaluated' || true
done
python3 -I scripts/d_checks.py
