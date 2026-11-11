#!/bin/sh
# H2 (docs/h2_falsifiable_prediction.md): the GPU against the 18 CPU cores on
# the two kernels that dominate the 200-digit sieve, end to end.
#
#   scripts/h2_run.sh
#
#   1. h2-cpu --prepare: the primes with their roots and the Montgomery lanes
#      both sides read (data/dumps/h2/, gitignored);
#   2. h2-cpu --run and h2-gpu (tools/h2_gpu/main.swift, built with swiftc),
#      each under hw-sample; their CSV lines go to data/results/h2/;
#   3. scripts/h2_summary.py: medians, ratios, the cross-checks, the criterion
#      and the bound on the laptop-plus-GPU time (data/results/h2_kernels.{md,csv}).
set -eu
cd "$(dirname "$0")/.."
cargo build --release --bin h2-cpu --bin hw-sample
swiftc -O tools/h2_gpu/main.swift -o target/release/h2-gpu
mkdir -p data/dumps/h2 data/results/h2
target/release/h2-cpu --prepare data/dumps/h2
target/release/hw-sample --run-id h2-cpu --interval-ms 250 -- \
  sh -c 'target/release/h2-cpu --run data/dumps/h2 > data/results/h2/cpu.csv'
target/release/hw-sample --run-id h2-gpu --interval-ms 250 -- \
  sh -c 'target/release/h2-gpu data/dumps/h2 > data/results/h2/gpu.csv'
python3 -I scripts/h2_summary.py
