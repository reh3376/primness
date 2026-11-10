#!/bin/sh
# D5 end to end: the per-value dump of every instrument polynomial, then the
# region-by-region comparison (docs/d5_smooth_clusters.md).
#
#   scripts/d5_run_all.sh [--force]
#
# The dumps go to data/dumps/<id>.values.csv (gitignored, about 315 MB each).
# They come from d1-measure at the protocol's bounds 2^20, 2^24, 2^28 over the
# D1 region; its result files go to a temporary directory, since the committed
# results already hold the same counts. --force rewrites existing dumps.
# Needs data/results/root_tail.json (d1-tail) and the d1-measurement/4 results.
# Env: D1_MEASURE (binary), A_MAX, B_MAX.
set -eu
cd "$(dirname "$0")/.."
FORCE=0
[ "${1:-}" = "--force" ] && FORCE=1
MEASURE=${D1_MEASURE:-target/release/d1-measure}
A_MAX=${A_MAX:-3000}; B_MAX=${B_MAX:-1500}
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
mkdir -p data/dumps
for poly in data/instruments/d1-*/cado.poly; do
  id=$(basename "$(dirname "$poly")")
  dump="data/dumps/$id.values.csv"
  if [ "$FORCE" -eq 0 ] && [ -f "$dump" ]; then echo "$id: dump exists"; continue; fi
  echo "=== $id: dump at 2^20,2^24,2^28 $(date -u +%H:%M:%SZ)"
  "$MEASURE" --instance "$id" --source cado-nfs --a-max "$A_MAX" --b-max "$B_MAX" --bounds 2^20,2^24,2^28 \
    --no-hw-sample --out "$tmp/$id.json" --dump "$dump" "$poly" > "$tmp/$id.log"
  # The dump run must reproduce the committed cofactor counts.
  cmp -s "$tmp/$id.cofactor.csv" "data/results/$id.scaled-down.b28.cofactor.csv" \
    || { echo "$id: cofactor counts differ from data/results/$id.scaled-down.b28.cofactor.csv" >&2; exit 1; }
done
python3 -I scripts/d5_clusters.py
