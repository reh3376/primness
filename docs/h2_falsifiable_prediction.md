# Falsifiable Prediction under Obstacle H / H2: what the laptop's GPU adds

Written on 2026-11-10, after H1 and before any GPU statistic was computed.

**Scope**
H1 measured the relation collection for a 200-digit number on this laptop's CPU: 3,896 hours with CADO-NFS (`docs/h1_sieving_cost.md`). The success criterion counts the laptop's GPU as well, so the GPU has to be measured before the laptop-plus-GPU envelope can be called insufficient. In H1, 87 to 89 percent of the time goes to two kernels: the bucket sieve (50 to 64 percent) and the ECM cofactorization of the survivors (25 to 37 percent). H2 measures the ratio of GPU to CPU throughput on each, with both implementations written here and checked against each other.

**Claim (H2)**
On this machine, the GPU's throughput is at most 10 times that of the 18 CPU cores on each of the two kernels. Then the laptop with its GPU needs at least 3,896 / (1 + 10) ≈ 354 hours for the relations of a 200-digit number, even if both kernels were ported perfectly and the GPU and CPU worked at once.

**The kernels**
1. **Bucket sieve, one special-q's worth on one side.** The region is a line of 2^29 positions, the size of a 200-digit special-q region (I = 15). Every prime p between 2^15 and 2^27 has one root r drawn at random, and every position r + k p in the line receives an update of ⌊log2 p⌋. That is about 3.2 × 10^8 updates, the volume H1's bucket-sieved primes produce per side. The work has two phases:
   - fill: each update goes into the bucket of its 2^16-position region;
   - apply: each region adds its bucket's updates to a byte array.

   The checksum is the sum of all bytes, which both implementations must reproduce. The CPU version runs 18 threads with buckets per thread. The GPU version fills buckets through atomic counters in device memory and applies them in threadgroup memory.
2. **Montgomery multiplication modulo a 96-bit odd modulus.** This is the operation ECM stage 1 spends its time on for H1's cofactors, which are below 2^85 and 2^96 bits. Long dependent chains x ← x · y mod n run in many independent lanes. The CPU uses two 64-bit limbs on 18 threads, the GPU three 32-bit limbs. Both must produce the same chain results for the same inputs. The throughput is multiplications per second.

**Measurement protocol**
1. CPU: binary `h2-cpu` with 18 threads. GPU: `tools/h2_gpu`, a Swift program with the Metal kernels compiled at runtime. Both run under `hw-sample`.
2. Each kernel runs five times after one warm-up run, and the median throughput is taken. Sieve: updates per second over fill and apply together, with the region and the primes identical on both sides. Multiplication: modular multiplications per second.
3. The CPU numbers are also compared with CADO-NFS's own sieving time per special-q in H1. That shows how this harness's CPU sieve stands against a tuned one, and so whether the ratio flatters the GPU.

**Rejection criterion**
H2 holds if both ratios are at most 10. If either exceeds 10, H2 is rejected, and the lower bound on the laptop-plus-GPU time becomes 3,896 / (1 + X), with X the larger measured ratio.

**Reported beside the verdict, not part of it**
- Both checksums and the chain results, as the evidence that the two implementations do the same work.
- Power and temperature during each kernel, CPU against GPU.
- The memory bandwidth each side reaches in the sieve.

**Hardware envelope**
This machine: Apple M5 Max, 18 CPU cores, 40 GPU cores, 128 GB unified memory that the CPU and the GPU share.
