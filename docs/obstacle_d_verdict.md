# Obstacle D: verdict on the measurements

Recorded on 2026-11-10, when the author closed the D measurements.

Obstacle D (`primness_roadmap.md` section 3) is the missing deeper mathematical structure: no published method improves on the exponent of the general number field sieve for hard integers. Its sub-claims D1 to D5 asked whether the values the NFS sieves hold more structure than the standard model gives them. This document records what was measured, what the closing checks found in the data, what is bounded without a measurement, and what would reopen D.

## 1. What was measured

Every claim was written down with its protocol and rejection criterion before any statistic was computed. The instances are the ten hard balanced semiprimes of 40 to 80 digits in `data/d1_test_instances.md`, each with a CADO-NFS polynomial, over the region a in [−3000, 3000], b in [1, 1500] (5,472,915 coprime values), at the smoothness bounds 2^14 to 2^28.

| Sub-claim | Question | Document | Verdict | At 2^28 |
|---|---|---|---|---|
| D1 | Are the values smoother than their sizes and root properties say? | `docs/d1_falsifiable_prediction.md` | rejected; adopted | −0.01% ± 0.03% after the root counts |
| D5 | Do smooth values cluster in regions that can be found in advance? | `docs/d5_falsifiable_prediction.md` | rejected | held-out lift −0.11% ± 0.15% on 50 × 50 tiles |
| D2, part 1 | Are values the NFS combines smooth together more often than apart? | `docs/d2_falsifiable_prediction.md` | rejected | −0.01% ± 0.03% (rational side with a field), −0.02% ± 0.06% (two fields) |

The model that survived all three gives every value the smooth probability π_B = Q_B(c) + s_B: the density of B-smooth integers among the integers without a prime factor up to 2000 whose cofactor has the value's size, measured on 1.69 × 10^9 consecutive integers, plus the exact first-order effect of the polynomial's own root counts at every prime between 2000 and B (`docs/baseline_calibration.md` sections 7 to 9). It holds on forty polynomials: the ten instruments and the thirty rotated fields of D2.

## 2. The closing checks

`scripts/d_checks_run_all.sh` asks of the data whether anything was left over that the three criteria could miss. The binary `d-checks` makes one pass over the per-value dumps, and `scripts/d_checks.py` computes the statistics into `data/results/d_checks.{md,csv}`. Each check covers a gap that the three criteria could have missed.

**2.1 A weak signal spread across many tests.** All 2,160 per-instance statistics of D1, D2 part 1 and D5, as z = deviation / error, per family and bound:

- D1's algebraic side has a mean z of +0.27 over its six bounds. The rational side has +0.31, and a linear polynomial has no structure to find, so the shift belongs to the method. In size it is about +0.1 to +0.3 percent at 2^16 to 2^20 and nothing at 2^28.
- The thirty rotated fields sit slightly below their prediction, mean z −0.40 at 2^28, about −0.05 percent.
- The related joint classes of D2 have mean z −0.06 and −0.04. The unrelated control pairs sit higher (+0.04 and +0.39).
- D5's held-out lifts are negative on both sides (mean z −0.18 and −0.14): regions picked for an excess in one half come up short in the other.
- The ten largest |z|, up to 38, are all D5 dispersions at 2^14 and 2^16 in d1-08 to d1-10. Each is carried by a 50 × 50 tile holding one smooth value where about 0.001 to 0.01 was expected, which is how a χ² behaves when the expected counts are tiny, not structure.

No family shows the positive shift or the positive tail that a hidden excess would leave.

**2.2 Assumptions the criteria did not test.** Per value, with the values smooth for certain left out:

| The model says this does not matter | Result |
|---|---|
| the size of the part made of the primes up to 2000, at fixed cofactor | flat: χ² p ≥ 0.20 in every family and bound, no slope beyond 1.5 errors |
| the other side's small part, and its cofactor, at the same (a, b) | flat: 3 of 36 χ² tests below p = 0.05 (about 2 expected), largest slope 2.4 errors |
| (a mod 2, b mod 2) and (a mod 3, b mod 3) | flat: every p ≥ 0.10 |
| whether the neighbour (a + 1, b) or (a, b + 1) is smooth | no excess: −0.13% ± 0.09% and +0.00% ± 0.08% at 2^28 over the forty polynomials; the one value beyond two errors is a deficit, −1.02% ± 0.39% for (a, b + 1) at 2^20 |

**2.3 Growth toward 200 digits.** Two checks:

- **In u = ln c / ln B.** A 200-digit run with CADO-NFS's parameters sieves values at u of about 4 to 6. The small bounds put this project's values in the same range. Pooled over the forty polynomials and all bounds, observed over predicted minus one is within ±0.06% for u from 1 to 3. It is +0.12% ± 0.09% for u from 3 to 3.5, +0.19% ± 0.57% from 4 to 4.5, and −2.9% ± 1.9% and −6.8% ± 5.1% in the two bins from 4.5 to 5.5. The model holds in the u range of the goal within the counting error. A larger instance would mostly retest the same u at a larger B.
- **In the digits of N.** With forty polynomials, the gap at 2^28 changes by −0.0000% ± 0.0021% per digit. The one-sided 95% upper bound on an excess is +0.47% at 200 digits and +0.64% at 250. At 2^24 the bounds are +1.11% and +1.47%. The loosest bound, 2^16, gives +7.5% and +9.9%.

**2.4 Algebraic structure at its most extreme.** The five irreducible toy quartics of the structure ladder were measured again with the final harness (`data/instruments/galois/`, `data/results/galois/`). Their tiny coefficients and Galois groups A4, S4, C4, D4 and V4 put the variance of the root count per prime at 1, 1, 2, 3 and 3. Before the root-count term their gaps at 2^14 run from −3.4% to +4.3%; after it, from +0.07% to +1.02%. The residual does not follow the variance: no slope beyond 1.6 errors at any bound, and the five scatter about their mean as their errors say (χ² 2.1 to 4.4 on 4). The Galois group acts through the root counts and through nothing else. The "A4 lever" of the November 3 note has nothing behind it beyond what α already measures. This answers open question 3 of the roadmap.

## 3. What is bounded without a measurement

- **D2, other forms.** A field of higher degree than the optimum changes the sizes of the values, which the model prices exactly. The records already use degree 5 or 6 where it pays: CADO-NFS's 200-digit parameters use degree 5, RSA-240 used degree 6. Sieving in dimension 3 makes the norms larger at these sizes. It pays for discrete logarithms in some fields F_{p^n} with n > 1, not for factoring.
- **D3, lattice embeddings.** Schnorr's lattice-based factoring claims were tested by Ducas (2021) and by independent studies in 2023. The lattices needed grow far beyond the claims, and the method does not scale.
- **D4, structure of the relation matrix.** Linear algebra is about 9 percent of the record computations (250 of 2,700 core-years for RSA-250). Making it free would speed the whole factorization up by about 1.1 times.
- **D5 in other region shapes**, the special-q lattices that real sieving uses, follows the same per-value model, which is local to each value. It was not measured.

## 4. Verdict

On hard balanced semiprimes of 40 to 80 digits, the values of the number field sieve are smooth exactly as often as their sizes and their root counts say, prime by prime. That holds independently across the two sides, across several fields, across positions in the region and across neighbouring values, with no dependence on the Galois group and no trend toward larger sizes. The L_N[1/3] analysis of the NFS rests on exactly these assumptions, and they hold here to about 0.1 percent. Obstacle D stands for the number field sieve family at this scale.

Three routes remain open, and none of them is a property of the values the NFS already uses:
1. smaller values: a polynomial construction for general N whose values beat the usual coefficient size N^{1/(d+1)};
2. relations from another source than polynomial values;
3. throughput, which is Obstacle H.

The harness can price any candidate for the first route from its value sizes and root counts within hours. That is the screening step for new mathematics.

## 5. What would reopen D

- A polynomial, or pair, whose measured yield exceeds this model by more than three errors at the protocol bounds and by more than one percent.
- A construction for general N whose values are smaller than the GNFS's at the same N.
- A measurement at 100 digits or more whose residual exceeds the 95 percent upper bounds of section 2.3.
- A source of relations other than polynomial values with an analysis at L[1/3] or better whose independence assumptions this harness can test.

## 6. Small open items

None of these items bears on the verdict.

- **The rational side scatters less than counting noise** (open question 2). Its sectors give χ² per sector of 0.79 to 0.87, its region-level D1 χ² is 3.3 on 10 at 2^28, and D5's lifts come out negative on both sides. Smooth values are slightly anti-clustered. A plausible cause: the multiples of each prime form a lattice, so a region holds an almost fixed number of them rather than a random one. If so, the binomial errors of D1 and D2 are upper bounds, clearly so on the rational side, and the nulls are stronger than stated. D5's held-out lift errors are about 16 percent too small (mean z² 1.37), which moves its largest criterion value from +1.5 to about +1.3.
- **The toy quartics share a small offset.** All five sit +0.10% ± 0.03% above the model at 2^24 and 2^28 and +0.3% at 2^16, whatever their group. That is three errors, and it is not seen in the forty NFS polynomials (−0.01% ± 0.03% at 2^28). Two of the quartics, x^4 + 1 and x^4 − 2, are even in a, so each value appears twice and their counting errors are √2 too small. The 5th cyclotomic polynomial repeats values where (b, a) is also in the region. The neglected second-order term and roots of small height are the candidates.
- **The rotated fields sit slightly low**, about −0.05 percent at 2^28 (mean z −0.40 over thirty).

## Files

| File | Role |
|---|---|
| `src/bin/d_checks.rs` | Binary `d-checks`: per column of a dump, sums of the smooth values, the prediction and its variance by u, small part, the other side, residues mod 2 and 3; neighbouring pairs with their exact variance (`d-checks-manifest/1`). |
| `scripts/d_checks.py` | The manifests, the pooled statistics of D1, D2 part 1 and D5, the per-value tests, the trend in digits, the quartics; writes `data/results/d_checks.{md,csv,sums.csv}`. |
| `scripts/d_checks_run_all.sh` | The quartics' root counts and measurements, then the checks, end to end. |
| `data/instruments/galois/`, `data/results/galois/` | The five quartics, their root counts to 2^28 and their measurements at 2^14 to 2^28. |
