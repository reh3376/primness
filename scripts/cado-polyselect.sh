#!/bin/sh
# CADO-NFS polynomial selection for one D1 instance, run with the binaries
# built outside this tree, under the hardware sampler.
#
#   scripts/cado-polyselect.sh <instance-id> <N> [degree] [P] [admax]
#
# Writes data/instruments/<id>/: stage1.polys (size-optimized candidates),
# ropt.polys (root-optimized candidates with MurphyE), polyselect.log (the
# exact commands, CADO commit, and the instruments' stderr), and
# polyselect.hardware.json (the hardware-sample/1 finish record of the
# trace under $PRIMNESS_HARDWARE_DIR). The best polynomial is chosen
# afterwards by scripts/cado-best-poly.py, not here.
set -eu
cd "$(dirname "$0")/.."

ID=${1:?instance id}
N=${2:?N}
DEGREE=${3:-4}
P=${4:-420}
ADMAX=${5:-2000}
CADO=${CADO_BUILD:-$HOME/external/cado-nfs/build/$(hostname -s).lan}
[ -x "$CADO/polyselect/polyselect" ] || CADO=${CADO_BUILD:-$(ls -d "$HOME"/external/cado-nfs/build/*/ | head -1)}
THREADS=${THREADS:-16}
INCR=${INCR:-60}
NQ=${NQ:-256}
KEEP=${KEEP:-20}
ROPTEFFORT=${ROPTEFFORT:-1}
BF=${BF:-131072}
BG=${BG:-131072}
AREA=${AREA:-1e12}
SAMPLER=${HW_SAMPLE:-target/release/hw-sample}
OUT=data/instruments/$ID
mkdir -p "$OUT"
RUN="$ID-polyselect-$(date +%s)"
STAGE1="$CADO/polyselect/polyselect -N $N -degree $DEGREE -P $P -admin 0 -admax $ADMAX -incr $INCR -nq $NQ -keep $KEEP -sopteffort 1 -t $THREADS"
ROPT="$CADO/polyselect/polyselect_ropt -inputpolys $OUT/stage1.polys -ropteffort $ROPTEFFORT -t $THREADS -Bf $BF -Bg $BG -area $AREA"
{
  echo "# cado-nfs polynomial selection for $ID"
  echo "# cado-nfs commit $(git -C "$HOME/external/cado-nfs" rev-parse --short HEAD 2>/dev/null || echo unknown), build $CADO"
  echo "# started $(date -u +%Y-%m-%dT%H:%M:%SZ), hardware run id $RUN"
  echo "# stage1: $STAGE1"
  echo "# ropt:   $ROPT"
} > "$OUT/polyselect.log"
"$SAMPLER" --run-id "$RUN" -- sh -c "
  $STAGE1 > '$OUT/stage1.polys' 2>> '$OUT/polyselect.log' &&
  $ROPT > '$OUT/ropt.polys' 2>> '$OUT/polyselect.log'
" > /dev/null
tail -n 1 "${PRIMNESS_HARDWARE_DIR:-data/hardware}/$RUN.jsonl" > "$OUT/polyselect.hardware.json"
echo "# finished $(date -u +%Y-%m-%dT%H:%M:%SZ)" >> "$OUT/polyselect.log"
echo "stage1 candidates: $(grep -c '^c0' "$OUT/stage1.polys" || true)"
echo "ropt candidates:   $(grep -c '^c0' "$OUT/ropt.polys" || true)"
echo "hardware record:   $OUT/polyselect.hardware.json"
