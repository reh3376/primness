# Ten Instances After α, and a Ladder of Structured Polynomials (2026-11-03 19:28 EST)

**Metadata**
- **Created**: 2026-11-03 19:28 EST
- **Last Modified**: 2026-11-03 19:28 EST
- **Version**: 1.0
- **Tags**: obstacle-D, D1, D2, murphy-alpha, rejection-criterion, galois, cyclotomic, baseline
- **Related Files**: data/results/summary.scaled-down.md, data/results/summary.scaled-down.b28.md, data/results/summary.all-bounds.csv, data/results/patterns.scaled-down.csv, data/results/fixtures/, scripts/d1_run_all.sh, scripts/d1_summary.py

## All ten instances

`scripts/d1_run_all.sh` selected a degree-4 polynomial for each instance with CADO-NFS (P and admax by digit count, from params.c60 through params.c80) and measured each at 2^14 through 2^20 and at the protocol's 2^20, 2^24, 2^28 on the same region (a in [−3000, 3000], b in [1, 1500], 5,472,915 values), with the line sieve and the hardware trace on. Polynomial selection took 40 s to 12 minutes per instance; each measurement took 1.3 to 2 s at 2^20 and under a minute at 2^28. Per-run files are `data/results/<id>.scaled-down.*` and `<id>.scaled-down.b28.*`.

Relative gap after α (observed minus the α-adjusted Dickman prediction, as a share of it), algebraic side:

| instance | digits | 2^14 | 2^16 | 2^18 | 2^20 | 2^24 | 2^28 | α |
|---|---|---|---|---|---|---|---|---|
| d1-01 | 40 | +3.2% | +0.7% | -1.0% | -2.8% | -3.5% | -3.4% | -3.96 |
| d1-02 | 44 | +15.1% | +6.0% | +2.1% | -0.4% | -2.3% | -2.6% | -3.22 |
| d1-03 | 48 | +22.3% | +11.4% | +4.5% | +1.3% | -1.2% | -2.1% | -5.05 |
| d1-04 | 52 | +18.8% | +6.7% | +2.1% | -0.2% | -2.0% | -2.8% | -4.11 |
| d1-05 | 56 | +18.4% | +9.2% | +5.3% | +2.0% | -1.3% | -2.4% | -3.80 |
| d1-06 | 60 | +24.3% | +10.9% | +6.1% | +2.4% | -0.6% | -1.9% | -3.57 |
| d1-07 | 64 | +41.1% | +29.9% | +16.3% | +10.3% | +3.6% | +0.2% | -5.33 |
| d1-08 | 68 | +37.9% | +24.0% | +15.1% | +9.4% | +3.4% | +0.4% | -5.49 |
| d1-09 | 72 | +14.1% | +13.0% | +6.8% | +2.9% | -0.9% | -2.0% | -5.94 |
| d1-10 | 80 | +22.2% | +17.5% | +9.8% | +7.8% | +1.8% | -0.4% | -4.71 |

Mean over instances: +21.7% at 2^14, +12.9% at 2^16, +6.7% at 2^18, +3.3% at 2^20, -0.3% at 2^24, -1.7% at 2^28.

## Reading

- The sign of the residual after α is set by u, not by the instance. At a fixed bound the small instances (small values, u near 3) sit below the α-adjusted prediction and the large instances (u of 4 to 5) sit above it; at a fixed instance the residual falls as B grows and u shrinks. The implied α drifts the same way on every instance. A residual that is a function of u alone is a property of the baseline, not of any polynomial.
- `scripts/d1_summary.py` applies the written criterion (mean absolute gap against twice the noise) per bound: rejected at 2^20, where the positive and negative instances cancel; not rejected at 2^14 to 2^18 (mean +0.0001 to +0.0002, 16 to 27 Poisson σ, about 3 σ of the instance spread) and at 2^24 to 2^28 (mean −0.001 to −0.003). The absolute gap weights the dense small instances most, which is why the 2^20 mean is near zero while the relative gaps are not; the criterion as written in `docs/d1_falsifiable_prediction.md` does not say which weighting it means.
- Neither verdict is evidence for D1. The sign flip with u is the shape of the Dickman approximation's own finite-B error: the next term of the expansion in 1/ln B is positive and grows with u, and at ln B between 10 and 20 it is of the same order as the residuals seen here, while the local-density form errs the other way. The baseline is not known to the percent level at these bounds, and the measurement is.

## The structure ladder

The same region and bounds, eleven polynomials, in `data/results/patterns.scaled-down.csv` and `data/results/fixtures/`:

- Reducible x^n − 1: residual after α +1.2% (n = 3), +5.8% (n = 4), −5.2% (n = 5), +83% (n = 6), −14% (n = 7), +143% (n = 8), +347% (n = 10), +1472% (n = 12), all at 2^20. Observed over raw Dickman reaches 28× (n = 8), 68× (n = 10) and 1100× (n = 12, mean u 6.3). The gain is not proportional to the number of pieces: ρ falls faster than exponentially in u, so k pieces of length u/k are smooth about ρ(u/k)^k of the time against ρ(u) for one piece, a ratio that itself grows like u^u. n = 5 and n = 7 are negative because their one large cyclotomic factor (Φ_5, Φ_7) has thin prime support. x^6 − 1 and d1-01 have the same mean u at 2^20 (3.1 against 3.2) and smooth densities of 0.675 against 0.059. The excess tracks the number of cyclotomic factors; it is the special number field sieve's advantage, and it is unavailable for a hard N because a reducible f factors N.
- Irreducible quartics with tiny coefficients: all below the α-adjusted prediction, −8.7% (A4), −9.4% (S4), −9.7% (C4), −11.4% (D4), −12.7% (V4) at 2^20, roughly ordered by the density of primes that can divide the values. C4 breaks the order. The thin-support penalty is the Mersenne effect in reverse: fewer admissible primes means fewer smooth values.
- d1-01's selected polynomial sits at −2.8%, closer to the baseline than any toy fixture: large coefficients and root optimization matter.

## What this changes

The harness now distinguishes three shapes: positive and growing with B (splitting), near zero (a real selected polynomial), and negative (thin support). D1 needs an irreducible polynomial with the first shape. None of the ten shows it; what they show is a baseline residual that is a function of u.

## Open

- Calibrate the baseline in this harness: sample random integers of the same sizes as the values and count their smoothness at the same bounds, so the comparison is against a measured density rather than an asymptotic one. Then re-apply the criterion with relative gaps and state the weighting in the prediction document.
- Test the A4 lever: search for polynomials for one instance with square discriminant and measure the residual.
- Fix the protocol's region per digit length so a run can be labeled `full`.
