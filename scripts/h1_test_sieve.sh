#!/bin/sh
# H1 (docs/h1_falsifiable_prediction.md): sample the sieving cost of RSA-200 on
# this machine with CADO-NFS's own polynomial (parameters/polynomials/c200.poly)
# and 200-digit parameters (params.c200), then integrate it to the relation target.
#
#   scripts/h1_test_sieve.sh [--force]
#
#   1. makefb: the side-1 factor base to lim1 = 1e8 with maxbits 15, as cadofactor does;
#   2. las -random-sample $SAMPLE on [q, 1.05 q) for every q in $QS, each under
#      hw-sample. The relations are discarded (data/dumps/h1/, gitignored); the
#      statistics lines are kept (data/results/h1/las.<q>.txt);
#   3. scripts/h1_cost.py: y(q), t(q), the integral to the target, the breakdown
#      (data/results/h1_cost.{csv,md}).
#
# Nothing is factored and no relation is kept. RSA-200's factorization has been
# public since 2005; the run measures how long this machine takes to sieve.
#
# Env: CADO_SRC (default ~/external/cado-nfs), CADO_BUILD ($CADO_SRC/build/Mac.lan),
#      SAMPLE (40), THREADS (18), QS (5e7 to 1.6e9 by doublings).
set -eu
cd "$(dirname "$0")/.."
FORCE=0
[ "${1:-}" = "--force" ] && FORCE=1
CADO_SRC=${CADO_SRC:-$HOME/external/cado-nfs}
CADO_BUILD=${CADO_BUILD:-$CADO_SRC/build/Mac.lan}
SAMPLE=${SAMPLE:-40}
THREADS=${THREADS:-18}
QS=${QS:-"50000000 100000000 200000000 400000000 800000000 1600000000"}
POLY=$CADO_SRC/parameters/polynomials/c200.poly
[ -f "$POLY" ] || { echo "missing $POLY (CADO_SRC)"; exit 1; }
mkdir -p data/dumps/h1 data/results/h1
shasum -a 256 "$POLY" | awk '{print $1 "  parameters/polynomials/c200.poly"}' > data/results/h1/c200.poly.sha256
(cd "$CADO_SRC" && git rev-parse --short HEAD 2>/dev/null || echo unknown) > data/results/h1/cado.commit
FB=data/dumps/h1/c200.roots1.gz
if [ "$FORCE" -eq 1 ] || [ ! -f "$FB" ]; then
  "$CADO_BUILD/sieve/makefb" -poly "$POLY" -lim 100000000 -maxbits 15 -out "$FB" -side 1 -t "$THREADS"
fi
cargo build --release --bin hw-sample
for q in $QS; do
  out=data/results/h1/las.$q.txt
  if [ "$FORCE" -eq 0 ] && [ -f "$out" ]; then echo "q=$q: sampled"; continue; fi
  q1=$((q + q / 20))
  rels=data/dumps/h1/rels.$q.txt
  echo "=== q in [$q, $q1): $SAMPLE special-q's at random, $(date -u +%H:%M:%SZ)"
  target/release/hw-sample --run-id "h1-c200-q$q" --interval-ms 1000 -- \
    "$CADO_BUILD/sieve/las" -poly "$POLY" -fb1 "$FB" -lim0 130000000 -lim1 100000000 -lpb0 32 -lpb1 33 \
    -mfb0 85 -mfb1 96 -ncurves0 13 -ncurves1 13 -I 15 -sqside 1 -q0 "$q" -q1 "$q1" \
    -random-sample "$SAMPLE" -seed 1 -t "$THREADS" -T -out "$rels"
  grep '^#' "$rels" > "$out"
  grep '^# Total [0-9]* reports' "$out" || true
done
python3 -I scripts/h1_cost.py
