#!/bin/sh
# H5 (docs/h5_falsifiable_prediction.md): the sieve proper's share of a complete
# factorization at the laptop's one-hour reach, and the ceiling on any sieve-side speedup.
#
#   scripts/h5_sieve_share.sh [--force]
#
# Nine las processes run at once, one per ninth of the special-q range RSA-130 sieved in H4,
# [7.5e5, 4.33e6). Each uses -t 2, as the run's nine clients did, and -random-sample 300 -T
# -seed 1. All use that run's polynomial and side-1 factor base (data/dumps/h4/rsa130, from
# scripts/h4_reach.sh) and its client parameters exactly. The nine run under one hw-sample.
# Kept in data/results/h5/: las's statistics lines per part (las.<k>.txt), the hashes of the
# polynomial and factor base, the CADO-NFS commit, the hw-sample summary, and the special-q
# count of every workunit of the H4 run (run_special_q.csv). Relations are discarded
# (data/dumps/h5/, gitignored). Then scripts/h5_summary.py.
#
# Env: CADO_SRC (default ~/external/cado-nfs), CADO_BUILD ($CADO_SRC/build/Mac.lan),
#      SAMPLE (300), PARTS (9).
set -eu
SELF=$(cd "$(dirname "$0")" && pwd)/$(basename "$0")
cd "$(dirname "$0")/.."
CADO_SRC=${CADO_SRC:-$HOME/external/cado-nfs}
CADO_BUILD=${CADO_BUILD:-$CADO_SRC/build/Mac.lan}
SAMPLE=${SAMPLE:-300}
PARTS=${PARTS:-9}
RUN=data/dumps/h4/rsa130
POLY=$RUN/c130.poly
FB=$RUN/c130.roots1.gz
Q0=750000
Q1=4330000
STEP=$(( (Q1 - Q0) / PARTS ))

if [ "${1:-}" = "--parts" ]; then
  k=1
  while [ "$k" -le "$PARTS" ]; do
    a=$((Q0 + (k - 1) * STEP))
    z=$((a + STEP))
    [ "$k" -eq "$PARTS" ] && z=$Q1
    "$CADO_BUILD/sieve/las" -poly "$POLY" -fb1 "$FB" -I 13 -lim0 6000000 -lim1 7500000 \
      -lpb0 28 -lpb1 29 -mfb0 54 -mfb1 57 -lambda0 1.93 -lambda1 1.96 -ncurves0 11 -ncurves1 13 \
      -sqside 1 -q0 "$a" -q1 "$z" -random-sample "$SAMPLE" -seed 1 -t 2 -T \
      -out "data/dumps/h5/rels.$k.txt" 2> "data/dumps/h5/las.$k.stderr" &
    k=$((k + 1))
  done
  wait
  exit 0
fi

FORCE=0
[ "${1:-}" = "--force" ] && FORCE=1
[ -f "$POLY" ] && [ -f "$FB" ] || { echo "missing $POLY or $FB: run scripts/h4_reach.sh first"; exit 1; }
mkdir -p data/dumps/h5 data/results/h5
(cd "$RUN" && shasum -a 256 c130.poly c130.roots1.gz) > data/results/h5/inputs.sha256
(cd "$CADO_SRC" && git rev-parse --short HEAD 2>/dev/null || echo unknown) > data/results/h5/cado.commit
{
  echo "q0,q1,special_q"
  for f in "$RUN"/c130.upload/c130.*-*.gz; do
    b=$(basename "$f" .gz)
    r=${b#c130.}
    n=$(gzip -dc "$f" | sed -n "s/^# Discarded [0-9]* special-q's out of \([0-9]*\) pushed$/\1/p" | tail -1)
    echo "${r%-*},${r#*-},${n:-0}"
  done | sort -t, -k1,1n
} > data/results/h5/run_special_q.csv
if [ "$FORCE" -eq 1 ] || [ ! -f "data/results/h5/las.$PARTS.txt" ]; then
  cargo build --release --bin hw-sample
  echo "=== $PARTS las processes, $SAMPLE special-q's each: started $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  target/release/hw-sample --run-id h5-c130 --interval-ms 1000 -- sh "$SELF" --parts > data/results/h5/hw.json
fi
# keep las's set-up lines and its closing statistics; the per-special-q blocks -T prints between
# them stay in data/dumps/h5/ (gitignored); the home directory is written as ~
k=1
while [ "$k" -le "$PARTS" ]; do
  grep '^#' "data/dumps/h5/rels.$k.txt" | awk -v home="$HOME" '
    { gsub(home, "~"); line[NR] = $0 }
    /^# Now sieving side/ && !first { first = NR }
    /^# Time for side/ { last = NR; n++ }
    END {
      for (i = 1; i < first; i++) print line[i]
      print "# [" n " per-special-q blocks omitted; full output in data/dumps/h5/]"
      for (i = last + 1; i <= NR; i++) print line[i]
    }' > "data/results/h5/las.$k.txt"
  k=$((k + 1))
done
python3 -I scripts/h5_summary.py
