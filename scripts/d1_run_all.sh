#!/bin/sh
# Polynomial selection, scaled-down measurement and report for every D1
# instance that has no result yet. Instances and N come from
# data/d1_test_instances.md. Polyselect parameters follow CADO's params.c60
# through params.c80 (degree 4; P and admax by digit count).
#
#   scripts/d1_run_all.sh [--force] [--remeasure] [--only d1-03,d1-04]
#
# --force repeats polynomial selection and the measurements. --remeasure
# repeats the measurements on the polynomials already chosen.
#
# Each instance is measured twice: at BOUNDS (default 2^14..2^20) and at
# BOUNDS28 (default the protocol's 2^20, 2^24, 2^28).
# Env: D1_MEASURE (binary), HW_SAMPLE (binary), A_MAX, B_MAX, BOUNDS, BOUNDS28.
set -eu
cd "$(dirname "$0")/.."
FORCE=0; REMEASURE=0; ONLY=""
while [ $# -gt 0 ]; do
  case "$1" in
    --force) FORCE=1 ;;
    --remeasure) REMEASURE=1 ;;
    --only) ONLY=$2; shift ;;
    *) echo "unknown argument $1" >&2; exit 2 ;;
  esac
  shift
done
MEASURE=${D1_MEASURE:-target/release/d1-measure}
export HW_SAMPLE=${HW_SAMPLE:-target/release/hw-sample}
A_MAX=${A_MAX:-3000}; B_MAX=${B_MAX:-1500}; BOUNDS=${BOUNDS:-2^14,2^16,2^18,2^20}; BOUNDS28=${BOUNDS28:-2^20,2^24,2^28}

params_for_digits() {
  d=$1
  if [ "$d" -le 60 ]; then echo "420 20000 1000"
  elif [ "$d" -le 64 ]; then echo "950 22000 256"
  elif [ "$d" -le 68 ]; then echo "1800 44000 256"
  elif [ "$d" -le 72 ]; then echo "3600 84000 128"
  else echo "10000 100000 128"
  fi
}

grep -E '^d1-[0-9]+ n ' data/d1_test_instances.md | while read -r id _ n; do
  case ",$ONLY," in *,"$id",*|",,") ;; *) continue ;; esac
  digits=${#n}
  out="data/results/$id.scaled-down.json"
  if [ "$FORCE" -eq 0 ] && [ "$REMEASURE" -eq 0 ] && [ -f "$out" ]; then echo "$id: result exists, skipping"; continue; fi
  set -- $(params_for_digits "$digits")
  P=$1; ADMAX=$2; NQ=$3
  echo "=== $id ($digits digits): polyselect P=$P admax=$ADMAX nq=$NQ $(date -u +%H:%M:%SZ)"
  if [ ! -f "data/instruments/$id/cado.poly" ] || [ "$FORCE" -eq 1 ]; then
    NQ=$NQ KEEP=40 ROPTEFFORT=5 sh scripts/cado-polyselect.sh "$id" "$n" 4 "$P" "$ADMAX"
    python3 -I scripts/cado-best-poly.py "data/instruments/$id/ropt.polys" "data/instruments/$id/cado.poly"
  fi
  echo "=== $id: measurement at $BOUNDS $(date -u +%H:%M:%SZ)"
  "$MEASURE" --instance "$id" --source cado-nfs --a-max "$A_MAX" --b-max "$B_MAX" --bounds "$BOUNDS" --out "$out" "data/instruments/$id/cado.poly"
  python3 -I scripts/d1_report.py "$out"
  out28="data/results/$id.scaled-down.b28.json"
  echo "=== $id: measurement at $BOUNDS28 $(date -u +%H:%M:%SZ)"
  "$MEASURE" --instance "$id" --source cado-nfs --a-max "$A_MAX" --b-max "$B_MAX" --bounds "$BOUNDS28" --out "$out28" "data/instruments/$id/cado.poly"
  python3 -I scripts/d1_report.py "$out28"
  echo "=== $id: done $(date -u +%H:%M:%SZ)"
done
