# Primness Project: History of Current States

**Metadata**
- **Created**: 2026-11-03 12:10 EST
- **Last Modified**: 2026-11-11 14:40 EST
- **Version**: 1.23
- **Tags**: current-state, history, roadmap, obstacle-D, obstacle-H
- **Related Files**: primness_roadmap.md

## Current State Entries

### 2026-11-11 14:40 EST
- Pull request 29 is merged. This change measures the laptop's one-hour reach.
- H4 was pre-registered in 4660e12. H1's cost, carried down by L_N[1/3, 1.923], puts the reach at 132 digits, and H4 holds within ±5 digits.
- CADO-NFS as shipped (18 threads, default parameter files, work server on localhost) factored RSA-100, RSA-110, RSA-120 and RSA-130 in 4:31, 12:38, 41:00 and 2:08:15, each time with the published factors.
- The reach is 123.3 digits, so H4 is rejected. The time doubles every 5.9 to 6.7 digits, against L's 4.3 to 4.8.
- The effective constant from the reach to 200 digits is 1.67 to 1.68. The 240-to-260-digit figures are restated with it: 1.6–1.8 × 10^5, 3.7–4.2 × 10^5 and 8.4–9.7 × 10^5 laptop-hours, beside 2.9 × 10^5, 7.6 × 10^5 and 2.0 × 10^6 with 1.923.
- Sieving is 60–77% of the time and the linear algebra 13–17%. The runs ran at sustained clocks below H1's 20-second samples (37–44 W against 62–67 W). That makes H1's cost about 18% low by the clocks, and the verdict does not change.
- Next: the linear algebra at 200 digits, then the tuned one-hour reach. H3 is not started.

### 2026-11-10 19:30 EST
- Pull request 28 is merged. This change closes the D measurements and starts Obstacle H.
- The D1 verdict is adopted, and the claim is rejected. `docs/obstacle_d_verdict.md` records the verdict on D: D1, D5 and D2 part 1 were rejected under criteria written first; D2's other forms, D3 and D4 are bounded by the literature and by the share of linear algebra.
- Closing checks (`d-checks`, `scripts/d_checks.py`, `data/results/d_checks.md`) found nothing left over:
  - no shift or tail in the 2,160 pooled statistics;
  - no dependence on the small-prime part, the other side, or (a, b) mod 2 or 3;
  - no excess of smooth neighbours;
  - the model calibrated up to u ≈ 5;
  - +0.47% at most at 200 digits at 2^28;
  - the A4, S4, C4, D4 and V4 quartics follow the model whatever their group.
- H1 was pre-registered in c6dd75a. Sieving RSA-200 with CADO-NFS's own c200 polynomial and parameters on this laptop takes 3,896 laptop-hours (0.62 of the 6,300 scaled from the records), so H1 holds. Sieving is 50–64% of the time and cofactorization 25–37%.
- H2 was pre-registered in d183e59 and implemented twice, in Rust for the CPU and in Metal for the GPU, with the two sides cross-checked. The GPU is 8.2× the CPU on the bucket sieve and 19.6× on 96-bit Montgomery multiplication, against a limit of 10, so H2 is rejected. At least 189 laptop-hours remain for 200-digit relations with the GPU (350–755 from H1's time shares), so Obstacle H is forced on the primary envelope.
- Next: H3, the rates a machine built for the two kernels needs at one hour; then linear algebra at 200 digits and the laptop's tuned one-hour reach.

### 2026-11-10 16:45 EST
- Pull request 28 (D5, the D1 verdict draft) is still open; this change follows it on `primness_dev01`.
- D2 part 1, several number fields sharing the rational side (MNFS), is tested under a prediction written first (`docs/d2_falsifiable_prediction.md`). Per instance there are three rotations f + v·g (0 < |v| ≤ 16, best by mean ln|F| + α, certified irreducible; `d2-fields`). The test is joint smoothness of G with each field and of pairs of fields against the product of their probabilities (`d2-joint`, exact variance), with the next instance's fields as the control.
- Rejected: the largest criterion value is +1.4 errors. At 2^28, R is −0.01% ± 0.03% for the cross pairs and −0.02% ± 0.06% for the algebraic pairs; the standard pair (F_0, G) is −0.03% ± 0.05%. Four fields give 3.2× the relations at 2^28, as independence predicts.
- The thirty rotated fields are at their prediction (χ² 32.7/30 at 2^28), so D1's closure holds on forty polynomials. Next: the D1 verdict, then higher-degree D2, D3, D4 or a larger-scale check.

### 2026-11-10 13:45 EST
- Pull request 27 is merged. This is the first change on `primness_dev01` after that merge.
- D5 (smooth clusters) tested on the ten D1 instances under a prediction written first (`docs/d5_falsifiable_prediction.md`): per-value probability from the conditional baseline and the root-count term; tiles 2 × 2 to 50 × 50 and 64 sectors; split-half held-out lift.
- Rejected: largest algebraic lift +1.5 errors (5 × 5 at 2^24), 11 of 18 negative; −0.11% ± 0.15% on 50 × 50 tiles at 2^28. Tiles scatter as counting noise (χ² per tile 0.98–1.00 on fine tiles); split-half correlations at zero.
- Unexplained: rational sectors scatter less than counting noise (χ² 0.79–0.87); hints that whole-region counting noise is an upper bound.
- D1 verdict drafted in `docs/d1_falsifiable_prediction.md` for the author. Next: the verdict, then D2–D4 or a larger-scale check.

### 2026-11-10 13:05 EST
- Pull request 26 is merged. This is the first change on `primness_dev01` after that merge.
- `d1-tail` counts the projective roots of both sides at all 14.6 million primes in (2000, 2^28] in 22 s (`root-tail/1`); its α tail matches `d1-alpha` to 4 × 10⁻¹³ at unramified primes and gives α at 2^28.
- The exact first-order term η_p R(c/p)/R(c) (Q_B(c/p) − Q_B(c)) over those primes explains the per-instance deviations: regression slope 1.01–1.19 (linear tail: 1.5–3.4); χ² of the ten algebraic gaps about zero 50–208 before, 4.5–13.2 on ten after. Weighted mean after: +0.05% ± 0.05% at 2^24, −0.01% ± 0.03% at 2^28. Rational control at zero.
- Region cases: the root at infinity (b ≤ 1500 < p) and the root 0 (|a| ≤ 3000 < p) are dropped; the first moves the rational side of d1-08 by −0.52% at 2^14.
- No evidence for D1, nothing left over. Next: the verdict in the prediction document (author's), then another sub-claim or a larger-scale check.

### 2026-11-10 05:15 EST
- The exact small-prime measurement is done: `d1-measure` bins every value by its cofactor (`d1-measurement/4`), `scripts/d1_conditional.py` sums the tabulated Q_B over the values. Bins are now an eighth of a bit so power-of-two bounds are bin edges (Q jumps at c = B).
- Rational control at zero (+0.02% at 2^24, −0.02% at 2^28). Algebraic mean +0.57% ± 0.64% at 2^20, +0.35% ± 0.42% at 2^24, +0.16% ± 0.24% at 2^28: no excess.
- Single instances deviate (d1-08 +1.72%, d1-09 −1.18% at 2^28) and follow α's tail over primes in (2000, B]: correlation 0.93–0.98; after the tail's first-order prediction the mean is +0.10% ± 0.09% at 2^28. The prediction is too small by 1.5–3.4, not yet explained.
- Read as no evidence for D1 at this scale. Next: the exact first-order term from root counts, or another sub-claim; the verdict in the prediction document is the author's.

### 2026-11-10 04:10 EST
- Pull request 25 is merged. This is the first change on `primness_dev01` after that merge.
- `d1-baseline` tabulates the conditional baseline (`d1-baseline/2`, `data/results/baseline.consecutive.cofactor.csv`): each of 1.69 × 10^9 consecutive integers is split into its part made of primes up to 2000 and a cofactor c, and the integers and B-smooth ones are counted per 0.25 bin of ln c, giving Q_B(c). Bins sum exactly to the unconditional counts; the unconditional table is unchanged.
- Q is 0.64 to 0.31 of the de Bruijn value at 2^28 for ln c from 25 to 70; it is banded at the small bounds; per-bin Poisson error 0.07–0.75% at 2^28, 2% at ln c = 70 for 2^24.
- Next: the instrument side in `d1-measure` (exact small-prime part per value, sum of Q_B over cofactors, compare on the ten instances).

### 2026-11-09 10:05 EST
- Pull request 24 is merged. This is the first change on `primness_dev01` after that merge.
- The Dickman baseline is measured in the harness (`d1-baseline`, `d1-baseline/1`): consecutive integers at 101 sizes follow the de Bruijn local form ρ + γρ′/ln B to a fraction of a percent; ρ(u) overstates their smoothness by 6–13% at these bounds, the derivative form understates it.
- Against that baseline the ten instances sit above the α-adjusted prediction at every bound (+44% at 2^14 to +4.2% at 2^28). The second-order term α₂ (excess variance of the log small-prime part, `src/alpha.rs`, `d1-alpha`, `alpha-moments/1`) closes 75–87% of it; the rational side is exact to 0.1%. Criterion not rejected at any bound; not read as evidence for D1.
- Criterion weighting proposed (`docs/baseline_calibration.md` §6). Next: the exact small-prime part per value against the baseline's conditional density.

### 2026-11-03 19:28 EST
- Pull request 23 is merged. This is the first change on `primness_dev01` after that merge.
- All ten instances measured at 2^14–2^20 and 2^20–2^28 with their own CADO polynomial and harness-computed α. Residual after α averages +21.7% at 2^14, +12.9% at 2^16, +6.7% at 2^18, +3.3% at 2^20, -0.3% at 2^24, -1.7% at 2^28; its sign is set by u. Criterion verdict split; read as baseline error, not as evidence either way.
- Structure ladder: x^n − 1 shows the excess that survives α (+83% for n = 6); irreducible toy quartics fall below, ordered roughly by Galois group. The harness distinguishes the three shapes.
- Next: calibrate the baseline with random integers in this harness; state the criterion's weighting; test A4 polynomials.

### 2026-11-03 18:29 EST
- Pull request 22 is merged. This is the first change on `primness_dev01` after that merge.
- Murphy α is recorded (`docs/murphy_alpha.md`) and computed in the harness: -3.9640 for the d1-01 polynomial. Result contract `d1-measurement/2` carries raw, α-adjusted and local-density predictions.
- The line sieve replaces trial division (exact agreement on d1-01, 35 times faster); d1-01 measured at 2^20, 2^24, 2^28 in 40 s.
- After α the algebraic side shows no excess over Dickman at any bound; the residual is a deficit of a few percent at the larger bounds, flat in u. The baseline's own error now exceeds the Poisson noise.
- d1-02 through d1-10 are running; `scripts/d1_summary.py` applies the criterion across instances.

### 2026-11-03 18:05 EST
- Every measurement writes CSV tables beside the result (bounds, u bins, rows, largest-prime-factor histogram, hardware trace) and `scripts/d1_report.py` writes a report. `data/results/d1-01.scaled-down.report.md` records the observations, the questions answered and the questions raised.
- The implied size shift on d1-01 is −4.08 to −3.76 across bounds; the excess rises with u; the two sides are close to independent. No Murphy α yet.

### 2026-11-03 17:50 EST
- Pull request 21 is merged. This is the first change on `primness_dev01` after that merge.
- CADO-NFS is built outside the tree (commit 692ecb7). `scripts/cado-polyselect.sh` selected a degree-4 polynomial for d1-01 (111 candidates, best MurphyE 4.749e-06) into `data/instruments/d1-01/cado.poly`.
- The harness evaluates instrument polynomials in BigInt over a coprime rectangle, counts smooth values by trial division over sieved primes, and compares with Dickman ρ (Simpson grid; tests at ρ(1), ρ(2) = 1 − ln 2, and literature values). No Murphy α yet.
- First scaled-down run on d1-01: `data/results/d1-01.scaled-down.json`. Algebraic side above the raw prediction at every bound (0.05888 vs 0.03340 at 2^20), rational side below. Not evidence for D1 until α is applied.
- Hardware telemetry `hardware-sample/1` records every instrument run (macmon, ioreg, sysctl), with the summary in the result file and trend charts on the bench.
- The required status check on the main ruleset is still to be applied by the owner.

### 2026-11-03 17:15 EST
- Pull request 20 is merged. This is the first change on `primness_dev01` after that merge.
- CI runs on every push with `--workspace`; the `bench` crate is now in the check and test run. The `pull_request` trigger is removed because Auto-PR's bot-opened drafts produced jobless failed runs on pull requests 15 through 20. The main ruleset requires the `Rust check and test` check.
- The bench binds `PRIMNESS_DASHBOARD_HOST` (default 127.0.0.1); the container sets 0.0.0.0 behind a loopback port mapping.
- The ten semiprime certificates were rechecked independently. The D1 prediction is still unmeasured.

### 2026-11-03 17:01 EST
- Pull request 19 is merged. This branch matches that merge.
- The `open_questions` section is empty. The D1 prediction is still unmeasured. The next step is one NFS polynomial per instance from CADO-NFS, outside this tree, then the smoothness count and the Dickman comparison.

### 2026-11-03 16:58 EST
- The roadmap has an `open_questions` section. No questions are recorded in it yet.
- The roadmap still has no Decision Log. Obstacles E–H remain a placeholder line.
- The harness, the ten proved semiprimes, and the msieve 264-bit gate are unchanged. The D1 prediction is still unmeasured.

### 2026-11-03 16:52 EST
- `docs/AGENT_ONBOARDING.md` was rewritten from the tree. It is the zero-context handoff.
- The roadmap contains obstacles A–D and a placeholder line for E–H. It has no `open_questions` section and no Decision Log. Sub-claims D1–D5 are in `context_convo_notes/2026-11-03_1205_obstacle_D_subclaims.md`.
- `context_convo_notes/INDEX.md` lists notes through 15:58 EST and omits the note files from 13:06 through 14:44.
- The harness, the ten proved semiprimes, and the msieve 264-bit gate are unchanged. The D1 prediction is still unmeasured.

### 2026-11-03 16:48 EST
- Pull request 18 is merged. This branch matches that merge.
- msieve 1.54, built outside this tree, runs NFS polynomial selection only when the input has more than 264 bits. The degree-4 parameter table starts at log10(N) = 80. All ten proved semiprimes are at or below 264 bits, so that binary does not select an NFS polynomial for them.
- CADO-NFS remains the external source for those polynomials. It is not installed. Its LGPL-2.1 source stays outside this repository.
- The D1 prediction is still unmeasured.

### 2026-11-03 16:40 EST
- The harness parses CADO and msieve polynomial files into `instrument-polynomial/1`. `format` and `source` are closed lists. Instrument scores are not fields. A parsed file stays `source` `unspecified`.
- CADO-NFS stays outside the tree because it is LGPL-2.1. msieve is public domain. The stock Apple clang build stops because `gettimeofday` is undeclared in `common/util.c`. A checkout outside this tree builds after that file includes `<sys/time.h>`. The binary is not in this repository. Polynomial selection has not been run. YAFU is not copied in; its repository does not state a license in the README or a root LICENSE file.
- The D1 prediction is still unmeasured. The ten proved semiprimes are unchanged. The next step is one polynomial per instance from CADO-NFS and one from msieve, then the smoothness count and the Dickman comparison.

### 2026-11-03 16:36 EST
- Pull requests 16 and 17 are merged.
- The harness parses CADO and msieve polynomial files into `instrument-polynomial/1`. `format` and `source` are closed lists. Instrument scores are not fields.
- CADO-NFS stays outside the tree because it is LGPL-2.1. msieve is public domain and is being built outside the tree. YAFU is not copied in; its repository does not state a license in the README or a root LICENSE file.
- The D1 prediction is still unmeasured.

### 2026-11-03 16:24 EST
- The current-state gate reads `schemas/current-state/1.json`, named by the roadmap metadata `Schema` line (`current-state/1`).
- Paths, the section heading, the stamp shape, and which metadata labels must change are const values in that version. Enums list the closed options.
- Pull request 15 is merged. Draft pull request 16 remains open. The D1 prediction is still unmeasured.

### 2026-11-03 16:06 EST
- Pull request 15 is merged.
- Auto-PR opens or refreshes a draft only after `primness_roadmap.md` Current State changes and this file keeps the previous snapshot heading plus the new one.
- The D1 prediction is still unmeasured. The ten proved semiprimes remain in `data/d1_test_instances.md`. The next measurement step is the external polynomials and the Dickman comparison.

### 2026-11-03 15:58 EST
- Current State in `primness_roadmap.md` advanced from the 12:42 EST snapshot.
- The D1 prediction is written and unmeasured. The harness and bench remain toy-scale for smoothness.
- `data/d1_test_instances.md` now holds ten proved semiprimes from 40 through 80 digits. Factors of at most 24 digits use deterministic Miller-Rabin below ψ_13. Larger factors use one Pocklington step.
- Next measurement step is the external polynomials and the Dickman comparison.

### 2026-11-03 13:20 EST
- Living roadmap (primness_roadmap.md v1.6) added to the repository.
- Notes index and current-state history updated.

### 2026-11-03 13:05 EST
- Auto-PR workflow added to primness_dev01.

### 2026-11-03 12:51 EST
- Survey note of existing tools committed.

### 2026-11-03 12:43 EST
- Ruleset protecting primness_dev01 against deletion and force-pushes.

### 2026-11-03 12:42 EST
- PR #1 merged to main.
