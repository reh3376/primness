# The Exact First-Order Term: The Per-Instance Deviations Are Root Counts (2026-11-10 13:05 EST)

**Metadata**
- **Created**: 2026-11-10 13:05 EST
- **Last Modified**: 2026-11-10 13:05 EST
- **Version**: 1.0
- **Tags**: obstacle-D, D1, root-counts, alpha-tail, conditional-density, null-result
- **Related Files**: docs/baseline_calibration.md, src/bin/d1_tail.rs, src/fp.rs, src/poly.rs, src/bin/d1_alpha.rs, scripts/d1_conditional.py, schemas/root-tail/1.json, data/results/root_tail.json, data/results/root_tail.csv, data/results/summary.conditional.md, data/results/summary.conditional.csv

## Why

After the exact small-prime measurement (note of 05:15) single instances still deviated by up to +1.7% and −1.2% at 2^28. The deviations followed α's tail over the primes between 2000 and B, but the linear prediction was too small by a factor of 1.5 to 3.4. The open question was whether the exact first-order term of those primes closes the gap, or whether something remains for D1.

## What was built

- `fp::count_roots`: the number of distinct roots of f mod p as the degree of gcd(x^p − x, f), with x^p computed in fixed arrays of machine words for p < 2^32 and degree up to 8. Tested against `roots_mod_p` at every prime to 20,000 and at three large primes, and the word powering against the general one at degrees 2 to 8.
- `d1-tail` (`src/bin/d1_tail.rs`, result `root-tail/1`): for every instrument polynomial and side, at every prime in (2000, 2^28], the projective root count r_p, summed per bin of ln p (width ln 2/64), with Σ η_p, Σ η_p ln p, the sum over affine roots alone, and the sum for the root 0. 14,630,540 primes, both sides of ten polynomials, 22 s on 18 threads. Bins are summed in prime order by one thread each, so the output does not depend on the thread count. Tested against evaluation at every point of P^1(F_p) to 400, and η against Murphy's term from the exact lifting for p in (2000, 3000].
- `poly::instrument_files`: the instrument discovery shared by `d1-alpha` and `d1-tail`.
- `scripts/d1_conditional.py`: the exact first-order term η_p R(c/p)/R(c) (Q_B(c/p) − Q_B(c)) summed over the primes and the values; Buchstab's ω for R; the α tail from the root counts when they reach B; the χ² of the gaps about zero and the noise-weighted mean beside the plain mean.

## What was measured

Algebraic side, ten instances:

| | 2^14 | 2^16 | 2^18 | 2^20 | 2^24 | 2^28 |
|---|---:|---:|---:|---:|---:|---:|
| regression slope of the gap on the term | 1.19 | 1.10 | 1.05 | 1.11 | 1.19 | 1.01 |
| χ² about zero, before (10 dof) | 50.3 | 57.6 | 69.4 | 103.4 | 207.5 | 191.2 |
| χ² about zero, after | 6.5 | 4.5 | 5.3 | 8.3 | 13.2 | 7.2 |
| weighted mean after | +0.31% ± 0.32% | +0.02% ± 0.19% | +0.11% ± 0.13% | −0.03% ± 0.09% | +0.05% ± 0.05% | −0.01% ± 0.03% |
| plain mean after (spread / √10) | +0.09% ± 0.52% | +0.73% ± 0.37% | +0.25% ± 0.13% | +0.29% ± 0.20% | +0.17% ± 0.09% | +0.03% ± 0.04% |

At 2^28: d1-08 +1.72% → +0.14%, d1-09 −1.18% → +0.11%. Largest remaining deviation of any algebraic instance at any bound: 2.1σ (d1-08 at 2^24).

Rational control after the term: χ² 8.6, 12.5, 19.1, 12.2, 6.4, 3.3; weighted mean within 0.02% of zero at every bound.

The exact term is 2.8 times the linear α-tail prediction at 2^14 and 1.5 times at 2^28: e^x − 1 against x, with x = ln p times the slope of ln Q in ln c, which is of order one for a prime near B.

α from the root counts agrees with `d1-alpha`'s α(2^24) − α(2000) to 4 × 10⁻¹³ on five algebraic and all ten rational sides, to 2.4 × 10⁻⁶ at most on the rest (primes dividing the discriminant). α at 2^28 is now known; no tail moves by more than 0.002 from 2^24.

## Questions answered

- Why the linear tail's regression slope exceeded 1: the linearization. The exact term has slope 1.01 to 1.19.
- Whether the mean after the tail was real: no. After the exact term the residuals are counting noise and the weighted mean is −0.01% ± 0.03% at 2^28.
- Is anything left for D1 on these instances? Nothing that this measurement can see, to 0.03–0.05% at 2^24 and 2^28.

## Questions raised

- Does the closure hold on a larger instance, where u is larger and the region reaches past the primes it sieves?
- The region has b ≤ 1500 and |a| ≤ 3000. Above those, the root at infinity and the root 0 are never or almost never met; both are dropped. Other roots of small height are not modelled; the χ² says they do not matter at this precision.
- The first χ² I computed was taken about the unweighted mean and gave the control a χ² up to five times too large, because one noisy instance moved that mean. The summary now uses the χ² about zero and the weighted mean.

## Hardware

`d1-tail` records no hardware trace: 22 s on 18 threads for the run to 2^28. The comparison script runs in about 10 s.
