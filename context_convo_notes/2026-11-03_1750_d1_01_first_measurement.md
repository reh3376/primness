# First Scaled-Down D1 Measurement on d1-01 (2026-11-03 17:50 EST)

**Metadata**
- **Created**: 2026-11-03 17:50 EST
- **Last Modified**: 2026-11-03 17:50 EST
- **Version**: 1.0
- **Tags**: obstacle-D, D1, measurement, dickman, cado-nfs, hardware-telemetry, scaled-down
- **Related Files**: data/results/d1-01.scaled-down.json, data/instruments/d1-01/cado.poly, docs/d1_falsifiable_prediction.md, docs/hardware_telemetry.md

## What was done

1. CADO-NFS was built outside this tree (`~/external/cado-nfs`, commit 692ecb7, `GMP=/opt/homebrew`, its own Python venv). Its driver has no parameter file within 3 digits of 40, so `scripts/cado-polyselect.sh` calls `polyselect` and `polyselect_ropt` directly (degree 4, P = 420, admax 20000, incr 60, nq 1000, keep 40, ropteffort 5; 16 threads; 37 s). 111 root-optimized candidates; `scripts/cado-best-poly.py` kept the largest MurphyE (4.749e-06) as `data/instruments/d1-01/cado.poly`. The score stays in a comment; the harness never reads it.
2. The harness gained `dickman` (ρ(u) by Simpson's rule on the integral equation, exact on [0, 2]; tests pin ρ(1) = 1, ρ(2) = 1 − ln 2, and ρ(3), ρ(4), ρ(5), ρ(10) from the literature), `eval` (homogeneous BigInt F(a, b)), and `sieve` (primes by bit sieve; largest prime factor by trial division in BigUint, u128 and u64 phases). `d1-measure` on an instrument file evaluates both sides over a coprime rectangle, counts smooth values at every bound from one pass, and writes a `d1-measurement/1` result with the Dickman prediction (mean of ρ(ln|F| / ln B)), the gap, the binomial standard error, mean u, and the hardware summary. Events carry `scale` `scaled-down`.
3. Hardware telemetry (`hardware-sample/1`, `docs/hardware_telemetry.md`) recorded both the polynomial selection and the measurement.

## Run

`data/results/d1-01.scaled-down.json`: a in [−3000, 3000], b in [1, 1500], gcd(a, b) = 1, 5,472,915 values, 17567 ms on 18 threads, no α.

| B | alg smooth | alg observed | alg predicted | alg gap | alg stderr | mean u | rat smooth | rat observed | rat predicted | rat gap | mean u | both |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 2^14 | 22791 | 0.00416 | 0.00145 | +0.00271 | 0.00003 | 4.59 | 451983 | 0.08259 | 0.10505 | -0.02246 | 2.63 | 1794 |
| 2^16 | 72269 | 0.01320 | 0.00566 | +0.00755 | 0.00005 | 4.01 | 853498 | 0.15595 | 0.18876 | -0.03281 | 2.30 | 10952 |
| 2^18 | 169705 | 0.03101 | 0.01552 | +0.01549 | 0.00007 | 3.57 | 1347627 | 0.24624 | 0.28808 | -0.04184 | 2.04 | 41095 |
| 2^20 | 322263 | 0.05888 | 0.03340 | +0.02548 | 0.00010 | 3.21 | 1900746 | 0.34730 | 0.39202 | -0.04472 | 1.84 | 110465 |

## Reading

- The algebraic side is above the raw Dickman prediction at every bound, by 100 to 250 standard errors, so the excess is not Poisson noise on this region. It is also not evidence for D1. The prediction carries no root-property correction, and the claim compares against Dickman adjusted by the measured Murphy α. CADO's own comment line reports α = −3.96 for this polynomial; that figure is an instrument score and is not used, but it says the excess is of the size root properties alone would produce. Only an α computed in this harness, from a definition recorded in this repository, can settle that.
- The rational side is below its prediction at every bound. Values of a linear form over coprime pairs are not random integers of their size; the same α treatment applies.
- The run is `scaled-down`. The protocol's bounds are 2^20, 2^24 and 2^28; trial division reaches 2^20 at 311,000 values a second here and would not reach 2^28. A sieve over the region is the way to the protocol's bounds.

## Hardware

15 samples over 16580 ms: CPU active ratio min 0.83, mean 0.90, max 0.98; CPU temperature 65.5 to 77.0 °C; GPU device utilization mean 0.6%; memory used ratio mean 0.62; GPU share of unified memory mean 0.125 (held by other processes; the harness does not use the GPU); total power 46.9 to 65.1 W. The polynomial selection trace is `data/instruments/d1-01/polyselect.hardware.json`.

## Open

- Murphy α: record the definition (Murphy 1999; Bai, Bouvier, Kruppa, Zimmermann 2016) in the repository, implement it over primes up to a stated bound for both sides, and apply it to the prediction. Then the rejection criterion applies.
- The protocol's region per digit length is not fixed, so no run can be labeled `full` yet.
- d1-02 through d1-10 have no polynomial yet.
- `docs/d1_falsifiable_prediction.md` step 2 still names msieve as a second source; the 264-bit gate rules it out for these instances.
