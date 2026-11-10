# Baseline calibration: what integers without structure do

The D1 comparison asks whether the values of a selected polynomial are B-smooth more often than integers of the same size with no structure, after the root properties (Murphy α) are accounted for. Through pull request 24 the "integers of the same size" side of that comparison was Dickman's ρ(u) read off a table, in two forms: ρ(u) itself and the derivative form ρ(u) + ρ′(u)/ln B. Neither had been checked against counts in this harness, and the residual after α changed sign with u across the ten instances in a way a baseline error would. This document records the measurement that settles which form is right, the correction it exposes in the α step itself, and how both are applied.

## 1. Method: consecutive integers near X

`d1-baseline` (`src/bin/d1_baseline.rs`) takes a grid of sizes ln X (default 20 to 120 in steps of 1, so X from 5 × 10^8 to 10^52) and, for each X, sieves the consecutive integers in [X − L/2, X + L/2) with L = 2^24 by default. It is the same line sieve the instrument runs use (`docs/line_sieve.md`), applied to f(x) = x + X at b = 1: the root of f mod p is −X mod p, so the marked positions are exactly the multiples of p, and the resieve-and-verify pass decides smoothness exactly for every candidate. The one approximation is the sieve's own (a value whose repeated prime powers exceed e^slack = B² is counted as not smooth). The counts match trial division on every chunk the tests cover (`chunk_counts_agree_with_trial_division`).

Every bound B in the list (default the protocol's 2^14 through 2^28) is answered from the one pass at the top bound, through the largest prime factor. For each size and bound the result records u = ln X / ln B, the smooth count, the observed density, its Poisson standard error, and three predictions:

- **ρ(u)**, the Dickman function. It is the density of B-smooth integers averaged over [1, X], Ψ(X, B)/X to first order.
- **ρ(u) + ρ′(u)/ln B**, the derivative of X·ρ(ln X / ln B), what the harness called the local density. It is the density near X if Ψ(X, B) were exactly X·ρ(u).
- **ρ(u) + γ ρ′(u)/ln B**, with γ = 0.5772… the Euler–Mascheroni constant. This is the density near X to first order in 1/ln B that follows from de Bruijn's function Λ(X, B), which approximates Ψ(X, B) with a relative error far below 1/ln B (Saias 1989; Tenenbaum, *Introduction to Analytic and Probabilistic Number Theory*, chapter III.5). Expanding Λ gives Ψ(X, B) = X ρ(u) − (1 − γ) X ρ′(u)/ln B + O(X/ln² B); the integral that produces 1 − γ is ∫₁^∞ {t}/t² dt. Differentiating in X gives the density near X, ρ(u) + γ ρ′(u)/ln B + O(1/ln² B). For 1 < u < 2 both statements can be checked by hand: Ψ(X, B) = X − Σ_{B<p≤X} ⌊X/p⌋, and the fractional parts Σ {X/p} contribute (1 − γ) X/ln X, while the count of multiples of primes near X in a short interval falls short of Σ 1/p by γ/ln X.

The result is `d1-baseline/2` (`schemas/d1-baseline/2.json`; version 1 had the points table only), with the points table `data/results/baseline.consecutive.points.csv` and the cofactor table of section 7 beside it and the hardware trace of the run (`hardware-sample/1`). The default grid takes about 94 s on this machine: 101 sizes, 1.69 × 10^9 integers.

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

One measurement separates them, and it replaces the expansion entirely (sections 7 and 8 record it and its result): compute the exact small-prime part s(F) of every value (trial division by the primes up to the α bound, about 300 of them, which the harness can afford over the region), and compare each value's smoothness with the baseline's conditional density Q_B(X/s) of B-smooth integers among those with no prime factor below the α bound, which `d1-baseline` can tabulate from the same consecutive integers by binning on the size of their cofactor. Nothing in that comparison assumes independence, a point mass or a Taylor expansion. A gap that survives it is structure at primes between the α bound and B, and the first evidence for D1. A gap that vanishes closes the α chapter and moves the search to the other sub-claims.

## 6. The criterion's weighting

`docs/d1_falsifiable_prediction.md` rejects the claim "if the mean gap is smaller than two standard deviations of the measurement noise" without saying whether the gap is the absolute density difference or the relative one. `scripts/d1_calibrate.py` reports both. The absolute mean is dominated by the small instances, whose densities are 100 times those of the large ones, and that is why the α-only summary rejected at 2^20 (the small instances cross zero there) and nowhere else. The relative mean weights the instances alike, which is what a claim about "an optimized polynomial" on "hard balanced semiprimes of 40–80 digits" means. The proposal, left for the author of the prediction to adopt, is: **the gap is relative (observed/predicted − 1), the prediction is the de Bruijn baseline with α and α₂ applied, and the noise is the larger of the Poisson noise and the spread across instances.** Under that reading the claim is not rejected at any bound today, by about two standard deviations of the spread at 2^28, and the measurement in section 5 is what should decide it.

## 7. The conditional baseline: Q_B(c) by cofactor size

Section 5 needs the density of B-smooth integers among the integers of size c with no prime factor up to the α bound y = 2000 (`alpha::DEFAULT_BOUND`). `d1-baseline` tabulates it in the same pass, and the result is `d1-baseline/2`.

**Definition.** Every integer n of every interval is split as n = s·c, where s is the part of n made of the primes up to y with their full multiplicity and c is the cofactor. The run counts, per bin of ln c and per bound B, the integers whose cofactor falls in the bin and the B-smooth ones among them. Their quotient is Q_B(c). For a fixed s, the cofactors of the integers of an interval are exactly the y-rough integers of the interval divided by s, so pooling over s and over the 101 sizes introduces no bias, and nothing in the table rests on independence or on an expansion.

**How s is computed.** A sieve over the 303 primes up to y walks every prime power up to 2^62 and adds ln p at each multiple, for every integer of the interval and not only for the sieve's candidates. A position the last power still divides is finished by exact division (`small_part_logs`; the test places 2^70 inside a range).

**The bins.** The width is ln 2 / 8, an eighth of a bit (`cofactor::DEFAULT_BIN_WIDTH`, `--cofactor-bin`). The choice is not cosmetic. A cofactor below B is always B-smooth, and just above B the primes are not, so Q jumps at c = B: from 1 to 0.31 at 2^28, to 0.15 at 2^24, and to 0 at 2^20 and below (a rough composite needs two primes above 2000). With a width of ln 2 over an integer every bound that is a power of two is a bin edge and no bin straddles the jump. The first build used bins of 0.25 and the rational side of d1-01 showed the cost: −0.11% at 2^28, 3 standard errors, from the one bin that held the jump. Bin 0 holds c = 1, the integers that are y-smooth outright; a cofactor is otherwise larger than y. The bin rule and the split are in `src/cofactor.rs`, shared by `d1-baseline` and `d1-measure`.

**The run.** 1,694,498,816 integers in 1,285 filled bins, 94 s. Bin 0 has 6,118,855 integers; the bins from ln c = 20 to 105 have 1.0 to 2.6 million each. Two checks hold exactly: at every bound the smooth counts summed over the bins equal the unconditional smooth counts summed over the sizes (24,012,705 at 2^14 through 243,385,322 at 2^28), and the points table is byte-for-byte the one of the earlier runs. A unit test compares the per-bin counts of three chunks with division by every prime.

**What the table shows.**

- Rough integers are much less often smooth than integers of the same size. Against the unconditional de Bruijn form at the same u, Q is 0.63 of it at ln c = 25 and 0.31 at ln c = 70 for B = 2^28, and 0.33 falling to 0.11 over the same range for B = 2^20. This is the size of the effect the α expansion approximates by a shift of ln n.
- At the small bounds Q is not a smooth function of ln c. For B = 2^14 the cofactor's primes have ln p in (7.60, 9.70], so three of them reach e^29.11 at most and four start at e^30.41: the bins from ln c = 29.11 to 30.76 hold about 20 million integers and no smooth one. The comparison reads the table per bin and interpolates only between neighbouring bins.
- Poisson error of Q per bin, relative: at 2^28, 0.10% at ln c = 25, 0.31% at 50, 1.1% at 70; at 2^24, 0.11%, 0.55% and 2.9%; at 2^20, 0.18%, 1.7% and 12%. A sum over an instance's values averages this down: the table's share of the noise of an instance's prediction is about an eighth of the instance's own counting noise (d1-01 at 2^28: 115 against 946 smooth values).

## 8. The comparison: every value against Q_B of its cofactor

**Method.** `d1-measure` (result `d1-measurement/4`) divides every value |F(a, b)| and |G(a, b)| by its part made of the primes up to 2000, by trial division grouped into products that fit a machine word (`cofactor::SmallPart`), and counts the values and the smooth ones per bin of ln c in the baseline's bins. `scripts/d1_conditional.py` sums Q_B over each side's values. That sum is the prediction: it uses no α, no α₂ and no Dickman function. Q is interpolated linearly between the mean ln c of neighbouring baseline bins at the instance's own mean in the bin, never across the jump at c = B; reading Q per bin instead moves the mean gap by 0.04% at most. The noise of a gap is the binomial noise of the instance's count and the table's noise on the prediction, together. The smooth counts of all twenty runs are those of the earlier runs; the split costs no measurable time.

**The control.** The rational polynomial is linear, with one root at every prime, so nothing distinguishes its cofactors from rough integers and the gap should vanish. It does:

| rational side | 2^14 | 2^16 | 2^18 | 2^20 | 2^24 | 2^28 |
|---|---:|---:|---:|---:|---:|---:|
| mean gap over ten instances | +0.35% | +0.27% | +0.18% | +0.07% | +0.02% | −0.02% |
| spread / √10 | 0.52% | 0.29% | 0.20% | 0.07% | 0.02% | 0.01% |
| mean per-instance noise | 0.74% | 0.38% | 0.23% | 0.16% | 0.09% | 0.06% |

No instance is beyond 2.7 standard errors at any bound. The method measures zero where zero is the answer, to 0.02% at the protocol's upper bounds.

**The algebraic side.**

| algebraic side | 2^14 | 2^16 | 2^18 | 2^20 | 2^24 | 2^28 |
|---|---:|---:|---:|---:|---:|---:|
| mean gap over ten instances | −0.64% | +0.95% | +0.59% | +0.57% | +0.35% | +0.16% |
| spread / √10 | 1.56% | 1.09% | 0.73% | 0.64% | 0.42% | 0.24% |
| mean per-instance noise | 3.10% | 1.52% | 0.83% | 0.50% | 0.24% | 0.14% |
| instances above zero | 4 | 6 | 7 | 6 | 6 | 6 |

The mean is within one standard error of zero at every bound. The +1.2% at 2^24 and +0.5% at 2^28 that remained after α and α₂ were mostly the expansion's own truncation: with the small primes handled exactly, a third of it is left and it is not distinguishable from zero.

The individual instances are not at zero. At 2^28 d1-08 is +1.72% (8.9 standard errors) and d1-09 is −1.18% (6.6); six of ten are beyond 2.9. The deviations have both signs and they have a known cause.

**The α tail.** The split removes the primes up to 2000 exactly. At a prime p between 2000 and B the polynomial has n_p roots, a random integer has one, and n_p varies from prime to prime. The first-order effect is the size shift α(B) − α(2000), which is zero on average over polynomials and has a spread of 0.048 across the ten. `d1-alpha --bound` computes it (the root enumeration in `src/alpha.rs` now uses the roots mod p in place of a scan of every residue, with identical results at 2000 and 2^14; α at 2^28 was not computed, and the tail at 2^24 stands in for it). The gap that shift predicts is exp(slope × tail) − 1, with the slope d ln(prediction)/d ln c read from the table.

| algebraic side | 2^14 | 2^16 | 2^18 | 2^20 | 2^24 | 2^28 |
|---|---:|---:|---:|---:|---:|---:|
| correlation of the gap with the tail's prediction | +0.95 | +0.93 | +0.97 | +0.95 | +0.98 | +0.98 |
| regression slope | 3.37 | 3.07 | 2.52 | 2.42 | 2.08 | 1.53 |
| spread of the gap across instances | 4.92% | 3.45% | 2.32% | 2.03% | 1.32% | 0.76% |
| spread after the tail | 3.62% | 2.51% | 1.48% | 1.30% | 0.72% | 0.29% |
| mean gap after the tail | −0.35% | +0.90% | +0.48% | +0.47% | +0.27% | +0.10% |
| its standard error | 1.15% | 0.79% | 0.47% | 0.41% | 0.23% | 0.09% |

The sign and the order of every instance's deviation follow its tail (d1-08 has the most negative tail, −0.094, and the largest excess; d1-09 the most positive, +0.085, and the largest deficit). The tail's prediction is too small by the regression slope, 1.5 at 2^28 rising to 3.4 at 2^14. I have not computed why. The likely reason is that a size shift is the wrong first-order form for a prime of this size: removing one prime p near B changes the cofactor by ln p, which is of the order of ln B and not small against the scale on which Q varies, so the exact first-order term is Σ (n_p − 1)/p × (Q_B(c/p)/Q_B(c) − 1) and exceeds its linearization. That sum can be computed from the root counts and this table, and it is the next check.

**Reading.** After the exact small-prime measurement the algebraic values of these ten polynomials are smooth as often as rough integers of their cofactor's size, on average, to +0.16% ± 0.24% at 2^28 and +0.35% ± 0.42% at 2^24. What each polynomial shows beyond that is its own root counts between 2000 and B, the continuation of α, with both signs. This is not evidence for D1. The claim's rejection threshold (a mean gap below two standard deviations of the noise) is met at every bound under the relative weighting of section 6. One thing is still open: the mean after the tail is positive at five of six bounds, by about one standard error each, and those bounds share their values, so the five are not independent.

## Files

| File | Role |
|---|---|
| `src/bin/d1_baseline.rs` | The consecutive-integer baseline, unconditional and by cofactor size; result `d1-baseline/2`. |
| `src/linesieve.rs` | `sieve_range` and `resieve_range`: the row sieve over an arbitrary window, used for the interval. |
| `src/cofactor.rs` | The split n = s·c, the bin rule, and grouped trial division by the small primes. |
| `src/bin/d1_measure.rs` | Per value: the cofactor bin; table `cofactor` in `d1-measurement/4`. |
| `scripts/d1_conditional.py` | Sum of Q_B over each instance's cofactors, the gaps, and the α tail; writes `data/results/summary.conditional.{csv,md}`. |
| `data/results/alpha_moments.b{14,16,18,20,24}.{json,csv}` | α at the larger bounds, for the tail. |
| `src/alpha.rs` | `affine_moments`, `projective_moments`, `valuation_moments`, `random_valuation_variance`, `alpha2`. |
| `src/bin/d1_alpha.rs` | α and α₂ for every instrument polynomial; result `alpha-moments/1`. |
| `scripts/d1_calibrate.py` | Baseline check, calibrated predictions, second-order term, implied variance, the criterion under both weightings. |
| `data/results/baseline.consecutive.{json,points.csv,cofactor.csv}` | The baseline run: the result, the unconditional points, and the conditional table Q_B(c). |
| `data/results/alpha_moments.{json,csv}` | α and α₂ for the ten instruments; `data/results/fixtures/alpha_moments.*` for the structured fixtures. |
| `data/results/summary.calibrated.{csv,baseline.csv,md}` | The instances against every baseline. |
