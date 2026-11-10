# Primness Project: History of Current States

**Metadata**
- **Created**: 2026-11-03 12:10 EST
- **Last Modified**: 2026-11-10 04:10 EST
- **Version**: 1.16
- **Tags**: current-state, history, roadmap, obstacle-D
- **Related Files**: primness_roadmap.md

## Current State Entries

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
