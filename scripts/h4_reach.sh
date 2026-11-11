#!/bin/sh
# H4 (docs/h4_falsifiable_prediction.md): the laptop's one-hour reach. Factor RSA-100,
# RSA-110, ... end to end with CADO-NFS as shipped (the default parameter file for the
# size, 18 threads), in ascending order, and stop after the first run over one hour.
#
#   scripts/h4_reach.sh [--force]
#
# Per instance, under hw-sample, in an empty work directory (data/dumps/h4/rsa<d>, gitignored):
#   cado-nfs.py N server.address=localhost -t 18 --workdir <dir>
# CADO-NFS's work server listens on every interface by default. server.address binds it to
# the loopback interface. That is a network setting, not a factoring parameter.
# N is the product of the published factors (scripts/h4_summary.py --n <d>).
# Kept in data/results/h4/:
#   rsa<d>.time     BSD `time -p -l` of the whole cado-nfs.py process;
#   rsa<d>.factors  what cado-nfs.py prints;
#   rsa<d>.log.txt  the log's Info lines without paths, addresses or certificate hashes;
#   rsa<d>.hw.json  hw-sample's summary (the trace stays in data/hardware/h4-rsa<d>.jsonl);
#   params.c<d>.sha256, cado.commit.
# Then scripts/h4_summary.py writes data/results/h4_reach.{md,csv}.
#
# Env: CADO_SRC (default ~/external/cado-nfs); CADO_PYTHON (default
#      $CADO_SRC/cado-nfs.venv/bin/python3, the venv CADO-NFS's scripts/setup-venv.sh makes
#      with flask and requests, which cado-nfs.py imports); THREADS (18);
#      SIZES (100 110 120 130 140).
set -eu
cd "$(dirname "$0")/.."
ROOT=$PWD
FORCE=0
[ "${1:-}" = "--force" ] && FORCE=1
CADO_SRC=${CADO_SRC:-$HOME/external/cado-nfs}
CADO_PYTHON=${CADO_PYTHON:-$CADO_SRC/cado-nfs.venv/bin/python3}
THREADS=${THREADS:-18}
SIZES=${SIZES:-"100 110 120 130 140"}
[ -x "$CADO_PYTHON" ] || { echo "missing $CADO_PYTHON (set CADO_PYTHON)"; exit 1; }
"$CADO_PYTHON" -c 'import flask, requests' || { echo "$CADO_PYTHON cannot import flask and requests"; exit 1; }
mkdir -p data/dumps/h4 data/results/h4
(cd "$CADO_SRC" && git rev-parse --short HEAD 2>/dev/null || echo unknown) > data/results/h4/cado.commit
cargo build --release --bin hw-sample
for d in $SIZES; do
  out=data/results/h4/rsa$d.time
  if [ "$FORCE" -eq 1 ] || [ ! -f "$out" ]; then
    N=$(python3 -I scripts/h4_summary.py --n "$d")
    W=$ROOT/data/dumps/h4/rsa$d
    rm -rf "$W"
    mkdir -p "$W"
    shasum -a 256 "$CADO_SRC/parameters/factor/params.c$d" | awk -v d="$d" '{print $1 "  parameters/factor/params.c" d}' > "data/results/h4/params.c$d.sha256"
    echo "=== RSA-$d: started $(date -u +%Y-%m-%dT%H:%M:%SZ)"
    target/release/hw-sample --run-id "h4-rsa$d" --interval-ms 1000 -- sh -c \
      'exec /usr/bin/time -p -l -o "$1/time.txt" "$2" "$3" "$4" server.address=localhost -t "$5" --workdir "$1" > "$1/stdout.txt" 2> "$1/stderr.txt"' \
      sh "$W" "$CADO_PYTHON" "$CADO_SRC/cado-nfs.py" "$N" "$THREADS" > "data/results/h4/rsa$d.hw.json"
    cp "$W/stdout.txt" "data/results/h4/rsa$d.factors"
    { python3 -I scripts/h4_summary.py --filter-log "$W/stderr.txt" | grep 'Using default parameter file' || true
      python3 -I scripts/h4_summary.py --filter-log "$W/c$d.log"; } > "data/results/h4/rsa$d.log.txt"
    cp "$W/time.txt" "$out"
  fi
  real=$(awk '/^real/ {print $2}' "$out")
  echo "RSA-$d: $real s wall"
  if awk -v r="$real" 'BEGIN { exit !(r > 3600) }'; then
    echo "RSA-$d took more than one hour: the protocol stops here"
    break
  fi
done
python3 -I scripts/h4_summary.py
