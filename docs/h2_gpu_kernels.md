# H2: what the laptop's GPU adds to the 200-digit sieve

`docs/h2_falsifiable_prediction.md` states the claim, the kernels and the criterion. It was committed (d183e59) before any GPU statistic was computed. This document records the implementations, the run and the result.

## 1. Method

**Inputs, shared.** `h2-cpu --prepare` writes the two input files both sides read. The first lists every prime p in (2^15, 2^27], 7,600,041 of them, each with a root r = splitmix64(p) mod p; on a line of 2^29 positions they make 315,362,460 updates. The second holds 131,072 Montgomery lanes, each with an odd 96-bit modulus n (top bit set), x0 and y below n, −n⁻¹ mod 2^32 and mod 2^64, and R² mod n for R = 2^96 and 2^128. The lane data come from num-bigint, and a test checks them.

**The CPU side** (`src/bin/h2_cpu.rs`, 18 threads):
- **Sieve.** Each thread fills its own buckets of 2^16 positions from a share of the primes with equal work. Each thread then applies the buckets of its regions as byte additions into a 64 KiB array.
- **Multiplication.** Montgomery with two 64-bit limbs and u128 products, eight interleaved chains per thread, 18,432 lanes of 2^16 steps.
- **Copy.** 1 GiB.

**The GPU side** (`tools/h2_gpu/main.swift`, Metal, the shaders compiled at runtime):
- **Sieve.** The fill splits each prime into work items of at most 256 updates (8,239,597 items) and appends to buckets of 2^14 positions through atomic counters in device memory. The apply runs one threadgroup of 1,024 threads per bucket, with two 16-bit counters per word in 32 KiB of threadgroup memory.
- **Multiplication.** Montgomery with three 32-bit limbs, one lane per thread over all 131,072 lanes.
- **Copy.** 1 GiB.

GPU times are the command buffers' own start and end times. Each kernel runs once to warm up and then five times; the median counts. `scripts/h2_run.sh` runs both sides under `hw-sample`, and `scripts/h2_summary.py` applies the criterion (`data/results/h2_kernels.md`).

## 2. Result

| Kernel | CPU, 18 threads | GPU, 40 cores | GPU / CPU |
|---|---:|---:|---:|
| bucket sieve, fill and apply (updates/s) | 2.61 × 10^9 | 2.15 × 10^10 | 8.2 |
| fill only | 3.04 × 10^9 | 2.53 × 10^10 | 8.3 |
| apply only | 1.86 × 10^10 | 1.42 × 10^11 | 7.6 |
| Montgomery multiplication mod 96-bit n (per s) | 4.14 × 10^9 | 8.10 × 10^10 | **19.6** |
| copy (bytes/s, read and write) | 3.54 × 10^11 | 5.42 × 10^11 | 1.5 |

**Both sides do the same work.** The sieve's checksum (the sum over the 2^29 positions of the byte modulo 256) is 6,282,484,675 on every run of both sides. All 18,432 lanes the two sides share give the same x0 · y^65536 mod n. A sample of the GPU's lanes also agrees with Python's integers, and the CPU's chain agrees with num-bigint.

**H2 is rejected.** The sieve is within the limit at 8.2, but the multiplication is 19.6 times the CPU, against a limit of 10 on each. An earlier manual run gave 7.4 and 18.4, so the ratios move by about ten percent between runs.

**The bound the protocol fixes for that case** uses the larger ratio, X = 19.6. The relations of a 200-digit number need at least 3,896 / (1 + 19.6) = **189 laptop-hours** with the GPU. That assumes both kernels were ported perfectly and the GPU and the CPU worked at once.

## 3. Reading

- **The GPU does not bring the laptop near one hour.** Each kernel can be sped up by its own ratio, using H1's time shares (sieving 57 percent, cofactorization 31 percent, the rest 12 percent). With the rest of the siever left on the CPU, the relation collection takes 755 laptop-hours. With the rest sped up like the sieve, it takes 350. The bound is 189. Before the linear algebra, the laptop with its GPU is 189 to 755 times too slow for a 200-digit number in an hour.
- **Where the GPU is strong.** Modular multiplication is arithmetic on 32-bit words, and 40 GPU cores do about 20 times as much of it as 18 CPU cores. The sieve is scattered memory traffic, and there the GPU gains 8 times. That is above the bare bandwidth ratio of 1.5, because its atomic updates in threadgroup memory are cheap. A GPU port would put the cofactorization on the GPU first.
- **What could move the ratios.** Both implementations are direct and unoptimized:
  - a tuned 96-bit multiplication could gain perhaps 1.5 to 2 times on the CPU, which lowers the ratio, and something on the GPU as well;
  - ECM is more than multiplication, with additions, a scalar ladder and a stage 2;
  - a real lattice siever enumerates a two-dimensional lattice per prime, sieves the small primes separately and rescans for survivors, which this kernel leaves out;
  - the CPU and the GPU share one memory system, so they do not add fully when both are busy.

  None of these moves the bound by a factor of 189.
- **Obstacle H is forced.** On the primary hardware envelope, a laptop with a GPU, relation collection at 200 digits takes at least 189 hours with the number field sieve and Obstacle D closed. The next question is H3: what a machine built for the two kernels would have to do. At one hour that is 8,300 special-q's, 4.5 × 10^12 sieve positions and about 4 × 10^7 cofactorizations per second, plus a linear algebra not yet measured.

## Files

| File | Role |
|---|---|
| `docs/h2_falsifiable_prediction.md` | The claim, the kernels and the criterion, committed before the run. |
| `src/bin/h2_cpu.rs` | Binary `h2-cpu`: the shared inputs (`--prepare`) and the CPU kernels (`--run`), with tests of the Montgomery chain against num-bigint and of the lane file. |
| `tools/h2_gpu/main.swift` | The GPU kernels in Metal, compiled at runtime; build with `swiftc -O`. |
| `scripts/h2_run.sh`, `scripts/h2_summary.py` | End to end under `hw-sample`; medians, ratios, cross-checks, the criterion and the bound. |
| `data/results/h2/`, `data/results/h2_kernels.{md,csv}` | Every run's CSV lines from both sides and the summary. |
