# Baseline calibration: what integers without structure do

The D1 comparison asks whether the values of a selected polynomial are B-smooth more often than integers of the same size with no structure, after the root properties (Murphy α) are accounted for. Through pull request 24 the "integers of the same size" side of that comparison was Dickman's ρ(u) read off a table, in two forms: ρ(u) itself and the derivative form ρ(u) + ρ′(u)/ln B. Neither had been checked against counts in this harness, and the residual after α changed sign with u across the ten instances in a way a baseline error would. This document records the measurement that settles which form is right, the correction it exposes in the α step itself, and how both are applied.

## 1. Method: consecutive integers near X

`d1-baseline` (`src/bin/d1_baseline.rs`) takes a grid of sizes ln X (default 20 to 120 in steps of 1, so X from 5 × 10^8 to 10^52) and, for each X, sieves the consecutive integers in [X − L/2, X + L/2) with L = 2^24 by default. It is the same line sieve the instrument runs use (`docs/line_sieve.md`), applied to f(x) = x + X at b = 1: the root of f mod p is −X mod p, so the marked positions are exactly the multiples of p, and the resieve-and-verify pass decides smoothness exactly for every candidate. The one approximation is the sieve's own (a value whose repeated prime powers exceed e^slack = B² is counted as not smooth). The counts match trial division on every chunk the tests cover (`chunk_counts_agree_with_trial_division`).

Every bound B in the list (default the protocol's 2^14 through 2^28) is answered from the one pass at the top bound, through the largest prime factor. For each size and bound the result records u = ln X / ln B, the smooth count, the observed density, its Poisson standard error, and three predictions:

- **ρ(u)**, the Dickman function. It is the density of B-smooth integers averaged over [1, X], Ψ(X, B)/X to first order.
- **ρ(u) + ρ′(u)/ln B**, the derivative of X·ρ(ln X / ln B), what the harness called the local density. It is the density near X if Ψ(X, B) were exactly X·ρ(u).
- **ρ(u) + γ ρ′(u)/ln B**, with γ = 0.5772… the Euler–Mascheroni constant. This is the density near X to first order in 1/ln B that follows from de Bruijn's function Λ(X, B), which approximates Ψ(X, B) with a relative error far below 1/ln B (Saias 1989; Tenenbaum, *Introduction to Analytic and Probabilistic Number Theory*, chapter III.5). Expanding Λ gives Ψ(X, B) = X ρ(u) − (1 − γ) X ρ′(u)/ln B + O(X/ln² B); the integral that produces 1 − γ is ∫₁^∞ {t}/t² dt. Differentiating in X gives the density near X, ρ(u) + γ ρ′(u)/ln B + O(1/ln² B). For 1 < u < 2 both statements can be checked by hand: Ψ(X, B) = X − Σ_{B<p≤X} ⌊X/p⌋, and the fractional parts Σ {X/p} contribute (1 − γ) X/ln X, while the count of multiples of primes near X in a short interval falls short of Σ 1/p by γ/ln X.

The result is `d1-baseline/2` (`schemas/d1-baseline/2.json`; version 1 had the points table only), with the points table `data/results/baseline.consecutive.points.csv` and the cofactor table of section 7 beside it and the hardware trace of the run (`hardware-sample/1`). The default grid takes about 95 s on this machine: 101 sizes, 1.69 × 10^9 integers.

## 2. Result: the de Bruijn local form is the baseline

`scripts/d1_calibrate.py` compares the three forms over the points with at least 1,000 smooth values and 1.5 ≤ u ≤ 6 (`data/results/summary.calibrated.baseline.csv`):

| B | points | ρ(u): mean dev | ρ + ρ′/ln B: mean dev | ρ + γρ′/ln B: mean dev | largest |
|---|---|---|---|---|---|
| 2^14 | 34 | −12.5% | +15.5% | +1.7% | 6.0% |
| 2^16 | 42 | −11.4% | +11.7% | +0.6% | 3.4% |
| 2^18 | 50 | −10.1% | +9.7% | +0.3% | 2.6% |
| 2^20 | 57 | −9.1% | +8.4% | +0.2% | 4.5% |
| 2^24 | 68 | −7.6% | +6.7% | +0.1% | 5.4% |
| 2^28 | 79 | −6.5% | +5.6% | +0.1% | 3.0% |

"Mean dev" is the mean of observed/form − 1; "largest" is the largest |observed/form − 1| for the de Bruijn form, at the high-u end where the counts are hundreds. Integers without structure are smooth less often than ρ(u) says, by 6 to 13 percent at these bounds, and more often than the derivative form says. The de Bruijn local form is right to a fraction of a percent from 2^16 up, and to 1.7% at 2^14 where the next term, of order 1/ln² B, is 1% in size. The harness's `predicted_local` was the wrong local form; it stays in the result files as recorded and the de Bruijn form is applied in the calibration step.

## 3. Applying the baseline to the instances

For each instance, bound and side, `scripts/d1_calibrate.py` rebuilds the prediction from the run's u-bin table: in each bin of width 0.1 the α-adjusted Dickman sum Σ ρ(u_i + α/ln B) is multiplied by the baseline's ratio to ρ at the α-shifted bin center, either the de Bruijn closed form or the measured curve interpolated in u (the two agree to under 3% everywhere and to under 1% from 2^16 up). Against the de Bruijn baseline the algebraic side of all ten instances sits above the prediction at every bound: the mean relative gap is +44% at 2^14, +30% at 2^16, +20% at 2^18, +14% at 2^20, +7.4% at 2^24, +4.2% at 2^28, and it no longer changes sign. The rational side sits within 1% at every bound.

That remaining gap has a shape: it falls by a factor of ten from 2^14 to 2^28 while α/ln B falls by only a factor of two, and it is ordered by instance in roughly the order of |α|. Both are what a second-order term of the α heuristic predicts.

## 4. The second-order term: α fixes the mean of the size shift, not its spread

Murphy's α says a value F(a, b) behaves like a random integer of size |F| e^α, because the primes up to the α bound contribute on average α less (in ln units) to a random integer than to F. The exact statement is about the log of the small-prime part S = Σ_p v_p ln p: α is E[S_random] − E[S_F]. The smoothness probability is then ρ of the size minus S, and because ρ is convex, a spread in S raises the mean of ρ above ρ at the mean of S:

E[ρ(u − S/ln B)] ≈ ρ(u − E[S]/ln B) + ½ Var[S] ρ″(u)/ln² B.

The random integer's own spread is already inside the measured baseline. What is not is the difference, and the relative correction to the α-adjusted prediction is

1 + α₂ ρ″(u)/(2 ρ(u) ln² B), with α₂(F) = Σ_{p ≤ bound} (Var[v_p(F)] − Var[v_p(n)]) ln² p,

the primes taken as independent as in α, and Var[v_p(n)] = p/(p − 1)² for a random integer. `src/alpha.rs` computes the second moment of the valuation by the same root lifting that gives the expectation (`valuation_moments`, `alpha2`), and `d1-alpha` writes α and α₂ for every instrument polynomial to `data/results/alpha_moments.{json,csv}` (`alpha-moments/1`). For the ten CADO polynomials α₂ is between 6.3 and 12.0 on the algebraic side and −0.88 on the rational side; for x^8 − 1 it is 71.

ρ″ falls fast with B at fixed size, so the correction is about 40% at 2^14 and 4% at 2^28 for these instances. Two checks that it is the right term and not a fit:

- **The implied extra variance.** Dividing each instance's gap against the de Bruijn baseline by Σ predicted·ρ″/(2ρ ln² B) gives the variance that would close it. A residual that is a pure second-order effect gives the same number at every bound. It does, within the noise and a mild drift at the smallest bounds where the expansion is coarse: d1-01 gives 6.9, 7.2, 7.4, 6.5, 5.8, 5.5 across the six bounds against a computed α₂ of 6.3; d1-07 gives 19.0, 19.7, 16.9, 16.3, 15.3, 13.9 against 12.0; d1-04 gives 12.3, 9.7, 8.8, 8.5, 8.2, 7.6 against 7.7. The table for all ten is in `data/results/summary.calibrated.md`.
- **The rational side.** A linear polynomial is the case where "a random integer with a known twist at small primes" should be exact. After the baseline and α₂ its mean gap is +0.11% at 2^20, +0.04% at 2^24 and −0.01% at 2^28, with a spread across instances of 0.07%, 0.02% and 0.01%.

With the term applied to the algebraic side, the mean gap becomes +11% at 2^14, +7.3% at 2^16, +4.3% at 2^18, +2.8% at 2^20, +1.2% at 2^24 and +0.5% at 2^28: 75% of the gap closed at 2^14, 87% at 2^28. The implied variances exceed the computed α₂ by 10 to 50 percent for most instances, and the gap that remains is positive for eight of ten instances at 2^28 (−0.2% and −0.7% for d1-01 and d1-09, +2.3% for d1-08).

## 5. What remains, and what decides it

Three things can carry the last 0.5% at 2^28 and 1.2% at 2^24:

1. **Higher moments.** S is right-skewed and the expansion stops at the variance. The third-order term has the same sign.
2. **The independence and the cofactor.** α and α₂ treat the primes as independent and the cofactor after the small primes as a random integer of its size. Both are approximations of the same kind as α itself.
3. **Structure beyond the small primes**, which is what D1 claims.

One measurement separates them, and it replaces the expansion entirely: compute the exact small-prime part s(F) of every value (trial division by the primes up to the α bound, about 300 of them, which the harness can afford over the region), and compare each value's smoothness with the baseline's conditional density Q_B(X/s) of B-smooth integers among those with no prime factor below the α bound, which `d1-baseline` can tabulate from the same consecutive integers by binning on the size of their cofactor. Nothing in that comparison assumes independence, a point mass or a Taylor expansion. A gap that survives it is structure at primes between the α bound and B, and the first evidence for D1. A gap that vanishes closes the α chapter and moves the search to the other sub-claims.

## 6. The criterion's weighting

`docs/d1_falsifiable_prediction.md` rejects the claim "if the mean gap is smaller than two standard deviations of the measurement noise" without saying whether the gap is the absolute density difference or the relative one. `scripts/d1_calibrate.py` reports both. The absolute mean is dominated by the small instances, whose densities are 100 times those of the large ones, and that is why the α-only summary rejected at 2^20 (the small instances cross zero there) and nowhere else. The relative mean weights the instances alike, which is what a claim about "an optimized polynomial" on "hard balanced semiprimes of 40–80 digits" means. The proposal, left for the author of the prediction to adopt, is: **the gap is relative (observed/predicted − 1), the prediction is the de Bruijn baseline with α and α₂ applied, and the noise is the larger of the Poisson noise and the spread across instances.** Under that reading the claim is not rejected at any bound today, by about two standard deviations of the spread at 2^28, and the measurement in section 5 is what should decide it.

## 7. The conditional baseline: Q_B(c) by cofactor size

Section 5 needs the density of B-smooth integers among the integers of size c with no prime factor up to the α bound y = 2000 (`alpha::DEFAULT_BOUND`). `d1-baseline` tabulates it in the same pass, and the result is `d1-baseline/2`.

**Definition.** Every integer n of every interval is split as n = s·c, where s is the part of n made of the primes up to y with their full multiplicity and c is the cofactor. The run counts, per bin of ln c and per bound B, the integers whose cofactor falls in the bin and the B-smooth ones among them. Their quotient is Q_B(c). For a fixed s, the cofactors of the integers of an interval are exactly the y-rough integers of the interval divided by s, so pooling over s and over the 101 sizes introduces no bias, and nothing in the table rests on independence or on an expansion.

**How s is computed.** A sieve over the 303 primes up to y walks every prime power up to 2^62 and adds ln p at each multiple, for every integer of the interval and not only for the sieve's candidates. A position the last power still divides is finished by exact division (`small_part_logs`; the test places 2^70 inside a range). The bins are 0.25 wide in ln c (`--cofactor-bin`, `--rough-bound`). Bin 0 holds c = 1, the integers that are y-smooth outright; a cofactor is otherwise larger than y, so no bin between 0 and ln y is filled.

**The run.** 1,694,498,816 integers in 450 filled bins, 95 s. Bin 0 has 6,118,855 integers; the bins from ln c = 20 to 105 have 4.3 to 4.9 million each. Two checks hold exactly: at every bound the smooth counts summed over the bins equal the unconditional smooth counts summed over the sizes (24,012,705 at 2^14 through 243,385,322 at 2^28), and the points table is byte-for-byte the one of the previous run. A unit test compares the per-bin counts of three chunks with division by every prime.

**What the table shows.**

- Rough integers are much less often smooth than integers of the same size. Against the unconditional de Bruijn form at the same u, Q is 0.64 of it at ln c = 25 and 0.31 at ln c = 70 for B = 2^28, and 0.32 falling to 0.10 over the same range for B = 2^20. This is the size of the effect the α expansion approximates by a shift of ln n.
- At the small bounds Q is not a smooth function of ln c. For B = 2^14 the cofactor's primes have ln p in (7.60, 9.70], so three of them reach e^29.1 at most and four start at e^30.4: the bin [30, 30.25) has 4.3 million integers and no smooth one. The comparison has to read the table per bin, not through a fitted curve.
- Poisson error of Q per bin, relative: at 2^28, 0.07% at ln c = 25, 0.22% at 50, 0.75% at 70; at 2^24, 0.08%, 0.40% and 2.0%; at 2^20, 0.14%, 1.2% and 8.7%. The instances' algebraic values run from about e^37 to e^77 (row means on d1-01 and d1-10), so their cofactors sit in this range. The comparison sums over many bins, which averages the noise down, but the gap to resolve is 0.5% at 2^28 and 1.2% at 2^24, and the upper bins at the lower bounds are thin. A longer interval (`--length 2^26`) multiplies every count by four.

**Not done yet.** The instrument side: `d1-measure` has to compute s(F) for every value by the same definition, look up Q_B at ln(|F|/s) in this table, and sum the predictions. That sum against the observed count is the comparison of section 5.

## Files

| File | Role |
|---|---|
| `src/bin/d1_baseline.rs` | The consecutive-integer baseline, unconditional and by cofactor size; result `d1-baseline/2`. |
| `src/linesieve.rs` | `sieve_range` and `resieve_range`: the row sieve over an arbitrary window, used for the interval. |
| `src/alpha.rs` | `affine_moments`, `projective_moments`, `valuation_moments`, `random_valuation_variance`, `alpha2`. |
| `src/bin/d1_alpha.rs` | α and α₂ for every instrument polynomial; result `alpha-moments/1`. |
| `scripts/d1_calibrate.py` | Baseline check, calibrated predictions, second-order term, implied variance, the criterion under both weightings. |
| `data/results/baseline.consecutive.{json,points.csv,cofactor.csv}` | The baseline run: the result, the unconditional points, and the conditional table Q_B(c). |
| `data/results/alpha_moments.{json,csv}` | α and α₂ for the ten instruments; `data/results/fixtures/alpha_moments.*` for the structured fixtures. |
| `data/results/summary.calibrated.{csv,baseline.csv,md}` | The instances against every baseline. |
