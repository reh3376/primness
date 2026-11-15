#!/bin/sh
# H6 (docs/h6_falsifiable_prediction.md): block Wiedemann at 200 digits on this laptop.
#
#   scripts/h6_linalg.sh [--force]
#
# Steps (each skipped when its result exists, unless --force):
#   merge   CADO-NFS's merge reruns on H4's RSA-130 purged relations with -target_density 170;
#           kappa = rows at 170 / rows at 110 (data/results/h6/merge170.txt, merge110.txt).
#   calib   bwc krylov (m = n = 64, thr=3x6, interval 500, end 3000, skip_online_checks=1, after
#           bwc prep) on RSA-130's real matrix and on h6-matrix's synthetic one with its N and
#           density 111.4.
#   mid     the same on synthetic matrices of N_200/4 and N_200/2 rows at density 170 (end 2000).
#   big     the same on a synthetic matrix of N_200 rows at density 170, end 6000 (>= 20 min).
#   lingen  lingen_b64 m=64 n=64 thr=1x1 random-input-with-length=N_200/32, as CADO-NFS runs it.
# Every krylov and lingen run is under hw-sample. Per run, data/results/h6/ keeps the matrix line
# from prep, krylov's progress and timing lines, the checkpoint times (<tag>.checkpoints.csv: the
# iteration and mtime of each V0-64.<k>, from which the per-interval times come) and the
# hw-sample summary. Matrices, caches and vectors stay in data/dumps/h6/ (gitignored).
# Then scripts/h6_summary.py writes data/results/h6_linalg.{md,csv}.
#
# Env: CADO_SRC (default ~/external/cado-nfs), CADO_BUILD ($CADO_SRC/build/Mac.lan), THR (3x6).
set -eu
cd "$(dirname "$0")/.."
ROOT=$PWD
FORCE=0
[ "${1:-}" = "--force" ] && FORCE=1
CADO_SRC=${CADO_SRC:-$HOME/external/cado-nfs}
CADO_BUILD=${CADO_BUILD:-$CADO_SRC/build/Mac.lan}
BWC=$CADO_BUILD/linalg/bwc
RUN=$ROOT/data/dumps/h4/rsa130
D=$ROOT/data/dumps/h6
R=data/results/h6
THR=${THR:-3x6}
INTERVAL=500
[ -f "$RUN/c130.purged.gz" ] && [ -f "$RUN/c130.sparse.bin" ] || { echo "missing H4's RSA-130 work directory: run scripts/h4_reach.sh"; exit 1; }
mkdir -p "$D" "$R"
cargo build --release --bin h6-matrix --bin hw-sample
(cd "$CADO_SRC" && git rev-parse --short HEAD 2>/dev/null || echo unknown) > "$R/cado.commit"

clean() { sed -e "s|$ROOT|.|g" -e "s|$HOME|~|g"; }

done_or_force() { [ "$FORCE" -eq 1 ] || [ ! -f "$1" ]; }

# krylov_run TAG MATRIX END: bwc prep, then krylov to END under hw-sample.
krylov_run() {
  tag=$1; m=$2; end=$3; w=$D/$tag
  done_or_force "$R/$tag.checkpoints.csv" || { echo "$tag: done"; return 0; }
  rm -rf "$w/wdir"
  mkdir -p "$w/wdir"
  set -- m=64 n=64 prime=2 thr="$THR" interleaving=0 nullspace=left interval="$INTERVAL" \
    matrix="$m" wdir="$w/wdir" cpubinding="$CADO_SRC/parameters/misc/cpubinding.conf"
  echo "=== $tag: prep $(date -u +%H:%M:%SZ)"
  "$BWC/bwc.pl" prep "$@" sequential_cache_build=1 > "$w/prep.out" 2>&1
  echo "=== $tag: krylov to $end $(date -u +%H:%M:%SZ)"
  target/release/hw-sample --run-id "h6-$tag" --interval-ms 1000 -- \
    sh -c 'out=$1; shift; exec "$@" > "$out" 2>&1' sh "$w/krylov.out" \
    "$BWC/bwc.pl" krylov "$@" ys=0..64 start=0 end="$end" skip_online_checks=1 > "$R/$tag.hw.json"
  grep -E '^[^ ]+: [0-9]+ rows [0-9]+ cols( \([0-9]+ extra rows\))? weight [0-9]+$|column permutation|row permutation' \
    "$w/prep.out" | clean > "$R/$tag.matrix.txt" || true
  grep -E 'N=[0-9]+ ; ETA|Timings for krylov|krylov.*(wct|cpu)' "$w/krylov.out" | clean > "$R/$tag.krylov.txt" || true
  { echo "iteration,mtime"
    for f in "$w"/wdir/V0-64.*; do
      echo "${f##*.},$(stat -f %Fm "$f")"
    done | sort -t, -k1,1n; } > "$R/$tag.checkpoints.csv"
}

# merge: kappa from RSA-130's purged relations at c200's target density
if done_or_force "$R/merge170.txt"; then
  mkdir -p "$D/merge170"
  echo "=== merge at density 170 $(date -u +%H:%M:%SZ)"
  "$CADO_BUILD/filter/merge" -mat "$RUN/c130.purged.gz" -out "$D/merge170/c130.history.gz" \
    -skip 32 -target_density 170.0 -t 18 > "$D/merge170/merge.stdout" 2> "$D/merge170/merge.stderr"
  grep -E '^Final matrix has' "$D/merge170/merge.stdout" > "$R/merge170.txt"
  grep -E '^Final matrix has' "$RUN/c130.merge.stdout.1" > "$R/merge110.txt"
fi
N200=$(python3 -I scripts/h6_summary.py --n200)
echo "N_200 = $N200"

# calib: the real RSA-130 matrix and its synthetic twin
krylov_run real130 "$RUN/c130.sparse.bin" 3000
if done_or_force "$R/syn130.checkpoints.csv"; then
  mkdir -p "$D/syn130"
  target/release/h6-matrix --rows 1183823 --density 111.4 --seed 1 --out "$D/syn130/m.bin" > "$R/syn130.generator.txt"
fi
krylov_run syn130 "$D/syn130/m.bin" 3000

# mid: N_200/4 and N_200/2 at density 170
for q in 4 2; do
  n=$((N200 / q)); tag=synq$q
  if done_or_force "$R/$tag.checkpoints.csv"; then
    mkdir -p "$D/$tag"
    target/release/h6-matrix --rows "$n" --density 170 --seed 1 --out "$D/$tag/m.bin" > "$R/$tag.generator.txt"
  fi
  krylov_run "$tag" "$D/$tag/m.bin" 2000
  rm -f "$D/$tag/m.bin"
done

# big: N_200 at density 170, at least 20 minutes of krylov
if done_or_force "$R/syn200.checkpoints.csv"; then
  mkdir -p "$D/syn200"
  target/release/h6-matrix --rows "$N200" --density 170 --seed 1 --out "$D/syn200/m.bin" > "$R/syn200.generator.txt"
fi
krylov_run syn200 "$D/syn200/m.bin" 6000

# lingen at the 200-digit sequence length, single-threaded as CADO-NFS runs it
L=$(( (N200 + 31) / 32 ))
if done_or_force "$R/lingen.txt"; then
  mkdir -p "$D/lingen"
  echo "=== lingen, length $L $(date -u +%H:%M:%SZ)"
  target/release/hw-sample --run-id h6-lingen --interval-ms 1000 -- \
    sh -c 'out=$1; shift; exec "$@" > "$out" 2>&1' sh "$D/lingen/lingen.out" \
    "$BWC/lingen_b64" m=64 n=64 prime=2 thr=1x1 random-input-with-length="$L" \
    tuning_thresholds=recursive:128,ternary:6400,cantor:6400,notiming:0 > "$R/lingen.hw.json"
  { echo "length $L"; grep -E 'Timings for lingen_b64|checksum' "$D/lingen/lingen.out"; } | clean > "$R/lingen.txt"
fi
python3 -I scripts/h6_summary.py
