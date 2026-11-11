# H1: the sieving cost of a 200-digit number on this laptop

`docs/h1_falsifiable_prediction.md` states the claim, the protocol and the criterion. It was committed (c6dd75a) before any 200-digit sieving statistic was computed. This document records the run and the result.

## 1. Method

**Instance and parameters.** RSA-200 with the degree-5 polynomial CADO-NFS ships as `parameters/polynomials/c200.poly` (sha256 in `data/results/h1/c200.poly.sha256`, CADO-NFS commit in `data/results/h1/cado.commit`). The parameters are `params.c200` exactly: lim0 = 1.3e8, lim1 = 1e8, lpb 32 and 33, mfb 85 and 96, 13 curves per side, I = 15 (a sieve region of 2^29 per special-q), special-q on the algebraic side. `makefb` wrote the side-1 factor base, and `las` built side 0's. Nothing was factored and the relations were discarded.

**Sampling.** `scripts/h1_test_sieve.sh` runs `las -random-sample 40 -t 18 -T` on [q, 1.05 q) for q = 5e7, 1e8, 2e8, 4e8, 8e8 and 1.6e9, each under `hw-sample`. `las` keeps its statistics lines in `data/results/h1/las.<q>.txt`. The six runs took 17 to 21 s each, at 1,590 to 1,620 percent CPU.

**Integral.** `scripts/h1_cost.py` takes the relations per special-q y(q) and the wall seconds per special-q t(q) of each range. It interpolates ln y and ln t linearly in ln q and integrates y dq / ln q and t dq / ln q from qmin = 5e7, at one special-q per root of f mod q and one root per prime on average. The integration stops at CADO-NFS's own relation target, 0.81 × Σ 2^lpb / ln 2^lpb = 461,027,928 raw relations.

## 2. Result

| q | relations per special-q | wall s per special-q (18 threads) | sieving share | cofactorization share | mean power | max CPU temperature |
|---|---:|---:|---:|---:|---:|---:|
| 5e7 | 25.9 | 0.53 | 50% | 37% | 66.5 W | 79.5 °C |
| 1e8 | 20.2 | 0.51 | 53% | 35% | 64.6 W | 81.1 °C |
| 2e8 | 15.5 | 0.48 | 56% | 32% | 63.6 W | 81.6 °C |
| 4e8 | 14.4 | 0.46 | 59% | 30% | 63.2 W | 82.5 °C |
| 8e8 | 10.5 | 0.42 | 62% | 27% | 62.6 W | 82.8 °C |
| 1.6e9 | 8.2 | 0.40 | 64% | 25% | 61.9 W | 83.2 °C |

The target is reached at Q_max = 6.32e8, after 29.9 million special-q's. The integral is **3,896 laptop-hours**, 0.44 laptop-years. **H1 holds**: the prediction was 6,300 laptop-hours within a factor of 2, and the measurement is 0.62 of it.

Per special-q, a region of 5.4 × 10^8 positions leaves about 3 × 10^5 to 7 × 10^5 candidates after sieving. Of those, 3,000 to 8,000 enter cofactorization (ECM on both sides), and 8 to 26 become relations. Peak memory is 6.4 GB. The rest of `las`'s counted time is the search for survivors (4 to 5 percent) and bookkeeping (6 to 9 percent). Norm initialization is below 1 percent.

## 3. Reading

- **The gap at 200 digits, measured.** Relation collection alone needs 3,896 times this machine's throughput to fit in one hour. In the records, linear algebra and polynomial selection together are 9 to 17 percent of the total (RSA-250 and RSA-240), which brings the whole factorization to about 4,300 to 4,700 laptop-hours. The roadmap's earlier figure of 6,800 to 7,800 assumed the laptop equals 18 of the records' Xeon Gold 6130 cores. Measured, it does the work of about 30 of them.
- **What one hour needs.** 29.9 million special-q's in 3,600 s is 8,300 special-q's per second. That is 4.5 × 10^12 sieve positions, and about 4 × 10^7 cofactorizations, per second. At this laptop's efficiency (about 64 W for 2.1 special-q's per second), the run would draw about 250 kW for an hour. The 240-to-260-digit figures of the roadmap scale by the same 0.62: about 3 × 10^5, 8 × 10^5 and 2 × 10^6 laptop-hours at 240, 250 and 260 digits.
- **Where a faster machine must be faster.** Sieving and cofactorization together are 87 to 89 percent of the time, in a ratio that moves from 1.4 to 2.6 as q grows. No single kernel dominates. A device that sped up only one of the two would gain at most about 3 times.
- **What this does not measure.** The GPU of this laptop, which the success criterion includes; the linear algebra at 200 digits; and duplicates, since CADO-NFS's filtering usually asks for some more relations than its first target.

## Files

| File | Role |
|---|---|
| `docs/h1_falsifiable_prediction.md` | The claim, the protocol and the criterion, committed before the run. |
| `scripts/h1_test_sieve.sh` | The factor base, the six sampled ranges under `hw-sample`, then the cost. |
| `scripts/h1_cost.py` | Parses `las`'s statistics and integrates the cost to the target; writes `data/results/h1_cost.{md,csv}`. |
| `data/results/h1/` | `las`'s statistics lines per range, the polynomial's hash and the CADO-NFS commit. |
