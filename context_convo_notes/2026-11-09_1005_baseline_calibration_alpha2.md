# The Baseline Measured, and the Second-Order Term of α (2026-11-09 10:05 EST)

**Metadata**
- **Created**: 2026-11-09 10:05 EST
- **Last Modified**: 2026-11-09 10:05 EST
- **Version**: 1.0
- **Tags**: obstacle-D, D1, baseline, dickman, de-bruijn, murphy-alpha, alpha2, rejection-criterion
- **Related Files**: docs/baseline_calibration.md, src/bin/d1_baseline.rs, src/bin/d1_alpha.rs, src/alpha.rs, src/linesieve.rs, scripts/d1_calibrate.py, schemas/d1-baseline/1.json, schemas/alpha-moments/1.json, data/results/baseline.consecutive.json, data/results/alpha_moments.json, data/results/summary.calibrated.md

## Why

After pull request 24 the residual after α across the ten instances changed sign with u: +22% at 2^14, −1.7% at 2^28. A residual that depends on u alone is what a baseline error looks like, and the Dickman table had never been checked against counts in this harness. The roadmap's next step was to calibrate it.

## What was built

- `linesieve::sieve_range` and `resieve_range`: the row sieve over an arbitrary window [a_lo, a_lo + len). `sieve_row` now calls the same walker. A test checks the window against the full row and checks that f(x) = x + X at b = 1 marks exactly the primes dividing each consecutive integer.
- `d1-baseline`: for each size on a grid of ln X (default 20 to 120 step 1), sieve 2^24 consecutive integers near X, decide smoothness exactly for every candidate, and record per bound the observed density, its Poisson error, ρ(u), the derivative form ρ + ρ′/ln B and the de Bruijn local form ρ + γρ′/ln B. Result `d1-baseline/1` with the points CSV and the hardware trace. 101 sizes, 1.69 × 10^9 integers, 100 s. A chunk-offset bug in the first build counted a quarter of the smooth values; the fix is covered by a test over three chunks against trial division.
- `alpha::affine_moments`, `projective_moments`, `valuation_moments`, `random_valuation_variance`, `alpha2`: the second moment of the p-adic valuation by the same lifting as α, and α₂ = Σ (Var[v_p(F)] − Var[v_p(n)]) ln² p. Tests pin the square identity E[v_p(x²)²] = 4(p+1)/(p−1)² and that one simple root per prime has the random integer's variance.
- `d1-alpha`: α and α₂ for every `data/instruments/*/cado.poly` (or named files), result `alpha-moments/1` plus CSV.
- `scripts/d1_calibrate.py`: which form the baseline follows; the instances' predictions rebuilt per u bin on the de Bruijn form and on the measured curve; the α₂ term; the implied extra variance per instance and bound; the criterion under absolute and relative weightings. Writes `data/results/summary.calibrated.{csv,baseline.csv,md}`.
- `docs/baseline_calibration.md`: the derivation of the de Bruijn local form (Λ(X, B) expansion; the 1 − γ from ∫{t}/t²), the α₂ derivation, the results, the proposed criterion weighting, and the decisive next measurement.
- Dashboard section "The baseline, measured" with the three-form chart at 2^20, the de Bruijn ratio at every bound, the gap-by-bound chart against four baselines with the instance-spread band, the implied-variance scatter against α₂, and the per-bound and per-instance tables. Four new datasets (`baseline`, `baseline_check`, `calibrated`, `alpha_moments`).

## What the baseline shows

Over the points with at least 1,000 smooth values and 1.5 ≤ u ≤ 6: ρ(u) is high by 12.5% (2^14) to 6.5% (2^28); the derivative form is low by 15.5% to 5.6%; the de Bruijn local form is within +1.7% (2^14), +0.6% (2^16), +0.3%, +0.2%, +0.1%, +0.1% (2^28), with mean |deviation| of 6.4σ at 2^14 falling to 0.8σ at 2^28. The next term of order 1/ln² B is visible only at 2^14.

## What the instances show against it

Algebraic side, mean relative gap over ten instances:

| baseline | 2^14 | 2^16 | 2^18 | 2^20 | 2^24 | 2^28 |
|---|---|---|---|---|---|---|
| α only (as before) | +21.7% | +12.9% | +6.7% | +3.3% | −0.3% | −1.7% |
| de Bruijn baseline, α | +44.0% | +29.8% | +20.0% | +14.1% | +7.4% | +4.2% |
| measured curve, α | +41.2% | +28.9% | +19.5% | +13.9% | +7.3% | +4.1% |
| de Bruijn baseline, α and α₂ | +11.1% | +7.3% | +4.3% | +2.8% | +1.2% | +0.5% |
| spread across instances (α₂ row) | 2.3% | 1.6% | 1.0% | 0.8% | 0.5% | 0.3% |

Rational side after α and α₂: +0.11% (2^20), +0.04% (2^24), −0.01% (2^28), spreads 0.07%, 0.02%, 0.01%.

The implied extra variance per instance is nearly constant across bounds (d1-01: 6.9, 7.2, 7.4, 6.5, 5.8, 5.5; d1-08: 17.9, 16.7, 15.7, 15.0, 14.4, 13.6) and tracks the computed α₂ (6.3 and 8.9), typically 10 to 50 percent above it. Computed α₂ for the ten runs from 6.28 (d1-01) to 11.97 (d1-07); x^8 − 1 has 71.

Criterion: not rejected at any bound under either weighting, against every baseline. Not read as evidence for D1: the residual after α was mostly the heuristic's own second-order term.

## Questions answered

- Which Dickman form is the baseline? The de Bruijn local form ρ(u) + γρ′(u)/ln B. Neither form the harness carried.
- Was the sign change of the residual with u a baseline error? Yes in part: against the measured baseline the residual is positive everywhere. The remaining u dependence is the second-order term of α, as its constant implied variance shows.
- Does the criterion's weighting matter? It decided the α-only verdict at 2^20. Proposal recorded in `docs/baseline_calibration.md` §6; the prediction document is unchanged.

## Questions raised

- What carries the last +0.5% at 2^28 and +1.2% at 2^24? Third moment, the independence and random-cofactor approximations, or structure between the α bound and B. The exact small-prime part per value against the baseline's conditional density Q_B(X/s) separates them without any expansion.
- Does the Galois ordering of the toy quartics survive the measured baseline and α₂?

## Hardware

Baseline run `d1b-*`: 98 samples over 100 s; CPU activity mean 0.59, memory used 0.85 of 128 GB (other processes included), CPU temperature mean 79.6 °C and peak 91.0 °C, package power mean 44 W. The trace is under `data/hardware/` (gitignored), the summary in the result file.
