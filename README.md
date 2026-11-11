# Primness

**Structure Beyond the Sieve**

## Acknowledgement

I want to begin by acknowledging Dr. Michael R. Anderson, PhD, Department of Mathematics at West Virginia State University. In 1994, as a junior at WVSU, I received a NASA research grant to investigate integer factorization with the specific aim of assessing whether RSA could be cracked. Dr. Anderson was my mentor on that project. I remember him as consistently gracious and accommodating, and I am glad to be able to recognize him at the start of this return to the problem after many years.

Looking back, I believe my fondness for algorithms, machine learning, and artificial intelligence started with Dr. Anderson. The rigorous yet patient way he guided me through the early stages of that factorization work planted the seeds for the technical path I have followed since. Therefore, I attribute my current successes in these fields to his kind guidance and his tolerance of a young fool with “lofty aspirations.”

## Project Summary

Primness seeks a deterministic or randomized algorithm that factors integers of at least 200 decimal digits (roughly 664 bits) in under one hour of wall-clock time on standard laptop hardware that includes a GPU. The target instances are hard balanced semiprimes with no special algebraic form that would admit SNFS or other tailored methods.

The preferred path is to discover deeper mathematical structure (Obstacle D) that improves upon or replaces the General Number Field Sieve. Progress is defined as removing an obstacle, substantially weakening it, or demonstrating that the laptop-plus-GPU constraints force a pivot to specialized hardware (Obstacle H).

## Intent

- Treat `primness_roadmap.md` as the living, authoritative statement of the problem, constraints, obstacles, and roadmap.
- Preserve first-person voice when expanding the roadmap.
- Convert every new idea into a precise, testable claim before implementation.
- Prefer small-to-medium experiments (comparable against YAFU, msieve, CADO-NFS) before any large-scale claim.
- Implement core numerical kernels in Rust; use a Python sidecar only where mature ML/AI libraries are required.
- All development occurs on `primness_dev01` (or feature branches from it). Direct commits to `main` are prohibited; changes arrive via pull request for personal review, approval, and merge.

## Current State

As of the latest update on the `primness_dev01` branch:

- Obstacle D has been decomposed into five precise, testable sub-claims (D1–D5).
- D1 (polynomial value smoothness relative to the Dickman–de Bruijn model) is the selected starting point.
- The first falsifiable prediction is written in `docs/d1_falsifiable_prediction.md`. It has not been decided.
- The ten 40–80 digit semiprimes, with prime proofs, are in `data/d1_test_instances.md`. The certificates were rechecked by an independent script.
- CADO-NFS is built outside this tree. Its polynomial selection ran on d1-01 (`scripts/cado-polyselect.sh`, `scripts/cado-best-poly.py`); the chosen degree-4 polynomial is `data/instruments/d1-01/cado.poly`.
- The harness evaluates instrument polynomials in BigInt over a coprime rectangle, counts B-smooth values by trial division over sieved primes, and compares the observed density with Dickman ρ (`src/dickman.rs`, tested at ρ(1) = 1, ρ(2) = 1 − ln 2 and literature values). No Murphy α is applied yet.
- Murphy α is recorded in `docs/murphy_alpha.md` and computed in the harness (-3.964 for d1-01); results are `d1-measurement/2` with raw, α-adjusted and local-density Dickman predictions. A line sieve (`docs/line_sieve.md`) replaced trial division, with identical counts and the protocol's 2^28 reachable in 40 s.
- The Dickman baseline is measured, not assumed (`docs/baseline_calibration.md`, `d1-baseline`): consecutive integers without structure follow the de Bruijn local form ρ(u) + γρ′(u)/ln B to a fraction of a percent, while ρ(u) overstates their smoothness by 6–13% at these bounds. Against that baseline the ten instances sit above the α-adjusted prediction at every bound (+44% at 2^14, +4.2% at 2^28); the second-order term α₂ of the α heuristic, computed from each polynomial (`d1-alpha`, `alpha-moments/1`), closes 75–87% of that gap and makes the rational side exact to 0.1% (`data/results/summary.calibrated.md`). The criterion rejects at no bound; the remaining +0.5% at 2^28 is not read as evidence for D1 yet.
- The exact small-prime measurement is done (`docs/baseline_calibration.md` sections 7 and 8, `data/results/summary.conditional.md`). `d1-baseline` tabulates Q_B(c), the density of B-smooth integers among integers with no prime up to 2000, by the size of c in bins of an eighth of a bit (`d1-baseline/2`); `d1-measure` splits every value the same way (`d1-measurement/4`); `scripts/d1_conditional.py` sums Q_B over the values, with no α and no expansion. The rational side, the control, is at zero (+0.02% at 2^24, −0.02% at 2^28). The algebraic mean over the ten instances is +0.35% ± 0.42% at 2^24 and +0.16% ± 0.24% at 2^28. Single instances deviate by up to +1.7% and −1.2% at 2^28, and those deviations follow α's tail over the primes between 2000 and B (correlation 0.93 to 0.98). No evidence for D1.
- All ten instances are measured with their own CADO polynomial and harness-computed α at 2^14 through 2^28 (`data/results/summary.all-bounds.csv`). The residual after α averages +21.7% at 2^14, +12.9% at 2^16, +6.7% at 2^18, +3.3% at 2^20, -0.3% at 2^24, -1.7% at 2^28; its sign is set by u, which is the Dickman baseline's finite-B error, so the criterion's split verdict is not evidence either way. A ladder of structured polynomials (`data/results/patterns.scaled-down.csv`) shows the excess that survives α only for reducible x^n − 1, up to +83% for x^6 − 1.
- First scaled-down run on d1-01: `data/results/d1-01.scaled-down.json` (5,472,915 values). The algebraic side is above the raw Dickman prediction at every bound and the rational side below it. That is not evidence for D1 until α is applied; the run is `scaled-down`, not the protocol.
- Every measurement writes CSV tables beside its result and `scripts/d1_report.py` turns them into a written report with the observations and open questions (`data/results/d1-01.scaled-down.report.md`).
- Every instrument run records a `hardware-sample/1` trace (`docs/hardware_telemetry.md`): CPU, GPU, unified memory and the GPU's share of it, temperature, fans, power; summarized in the result file and plotted on the bench.
- CI runs on every push with `--workspace`. The `pull_request` trigger is gone because Auto-PR's bot-opened drafts produced jobless failed runs. The main ruleset is to require the `Rust check and test` check on the head commit; the owner applies that rule in the repository settings.
- The harness parses CADO and msieve polynomial files (`instrument-polynomial/1`). Those tools are not vendored: CADO-NFS is LGPL-2.1, and YAFU does not state a license. A local msieve 1.54 build enters NFS polynomial selection only above 264 bits, so it cannot supply polynomials for these instances.
- Auto-PR refuses to open or refresh a draft unless this branch satisfies the current-state contract `schemas/current-state/1.json`.
- `docs/AGENT_ONBOARDING.md` is the zero-context handoff. The roadmap has one `open_questions` entry and no Decision Log. `context_convo_notes/INDEX.md` omits the note files from 13:06 through 14:44.
- The exact first-order term of the primes between 2000 and B is computed from the root counts (`d1-tail`, `root-tail/1`, `docs/baseline_calibration.md` section 9): every prime in (2000, 2^28], 14.6 million of them, in 22 s. It explains the deviations of single instances (regression slope 1.01–1.19, against 1.5–3.4 for the linear α tail), and after it the ten algebraic gaps are counting noise at every bound (χ² 4.5–13.2 on ten, from 50–208). The noise-weighted mean after it is −0.01% ± 0.03% at 2^28 and +0.05% ± 0.05% at 2^24; the rational control stays at zero. No evidence for D1, and nothing left over.
- D5, the sub-claim that smooth values cluster in regions that can be located, is tested on the same ten instances under a prediction written before measuring (`docs/d5_falsifiable_prediction.md`, `docs/d5_smooth_clusters.md`, `scripts/d5_clusters.py`). Each value's smooth probability is the one D1 ended with. Regions picked for an excess on one half of the values show none on the other: the largest held-out lift is +1.5 errors, and on 50 × 50 tiles at 2^28 it is −0.11% ± 0.15%. Tiles scatter as counting noise says. D5 is rejected at this scale. The rational side's sectors scatter less than counting noise, which is open.
- `docs/d1_falsifiable_prediction.md` ends with the D1 verdict, adopted on 2026-11-10: rejected at 2^20, 2^24 and 2^28 under the relative weighting.
- D2 part 1, several number fields that share the rational side (the multiple number field sieve), is tested under a prediction written before measuring (`docs/d2_falsifiable_prediction.md`, `docs/d2_multiple_fields.md`, `scripts/d2_run_all.sh`). Each instance gets three rotations f + v·g of its polynomial (`d2-fields`), and `d2-joint` counts the values that are smooth together against the product of their probabilities. Values the NFS combines are smooth independently: at 2^28 the joint excess is −0.01% ± 0.03% for the rational side with a field and −0.02% ± 0.06% for two fields, and the largest criterion value is +1.4 errors. D2 part 1 is rejected. The extra relations from more fields are the known MNFS gain, and the thirty rotated fields extend D1's closure to forty polynomials.
- The D measurements are closed (`docs/obstacle_d_verdict.md`). A closing pass over all the data looked for what the three criteria could miss (`scripts/d_checks_run_all.sh`, binary `d-checks`, `data/results/d_checks.md`), and found nothing left over:
  - the 2,160 per-instance statistics show no hidden shift;
  - smoothness at fixed cofactor ignores the small-prime part, the other side, and (a, b) mod 2 and 3;
  - neighbouring values are independent;
  - the model holds up to u ≈ 5, the range a 200-digit run sieves in;
  - the trend in digits allows at most +0.47% at 200 digits at 2^28;
  - five toy quartics with Galois groups A4 to V4 follow the model whatever their group.

  The values of the number field sieve are as smooth as their sizes and root counts make them, and nothing more.
- Obstacle H has started. H1 was pre-registered before the run (`docs/h1_falsifiable_prediction.md`). Sieving RSA-200 with CADO-NFS's own 200-digit polynomial and parameters on this laptop, sampled over six special-q ranges and integrated to CADO-NFS's relation target, takes 3,896 laptop-hours (`docs/h1_sieving_cost.md`). That is 0.62 of the estimate scaled from the records, so H1 holds. Sieving takes 50–64% of the time and cofactorization 25–37%.
- H2 measured the laptop's GPU on the same two kernels, with both implementations written here and cross-checked (`docs/h2_gpu_kernels.md`, `src/bin/h2_cpu.rs`, `tools/h2_gpu/main.swift`). Against the 18 CPU cores, the GPU is 8.2 times as fast on the bucket sieve and 19.6 times on 96-bit Montgomery multiplication. H2's limit was 10, so H2 is rejected. Even so, the bound it fixes leaves at least 189 laptop-hours for 200-digit relations with the GPU, and the estimates from H1's time shares are 350 to 755. Obstacle H is forced on the primary hardware envelope.
- H4 measured the laptop's one-hour reach end to end, under a prediction committed before the runs (`docs/h4_falsifiable_prediction.md`, `docs/h4_one_hour_reach.md`, `scripts/h4_reach.sh`). CADO-NFS as shipped factored RSA-100, RSA-110, RSA-120 and RSA-130 on 18 threads in 4:31, 12:38, 41:00 and 2:08:15, and each run printed the published factors.
  - The reach is 123.3 digits. H1's cost carried down by L_N[1/3, 1.923] predicted 132, so H4 is rejected: below 200 digits the time doubles every 6 to 7 digits, not every 4 to 5.
  - The constant that joins the reach to H1's 200-digit cost is 1.67 to 1.68.
  - The gap to the goal is 77 digits, or 4,300 to 4,700 times the laptop's hour.
- H5 measured what a better sieve could save, under a prediction committed first (`docs/h5_falsifiable_prediction.md`, `docs/h5_sieve_share.md`, `scripts/h5_sieve_share.sh`).
  - Nine `las -T` samples of RSA-130's sieve, as H4 ran it, put the sieve proper (norms, sieving and survivor search) at 84.9% of `las`'s time and 65.1% of the complete factorization.
  - A sieve that cost nothing and found the same relations would make the one-hour-reach factorization at most 2.87× faster, so H5 holds (limit 3×). At 200 digits the ceiling is 1.8–2.7×, and a free relation collection buys 5.9–11×, against the 4,300–4,700× one hour needs.
  - From RSA-130 to 200 digits the cost per sieve position grows only 1.45×. The number of positions grows 2,015×, because smooth values become rarer. A far more efficient replacement would have to examine far fewer candidates, which is a change of algorithm rather than of sieve.
- Next concrete step: the linear algebra at 200 digits, the floor that remains even with a perfect sieve. Then the one-hour reach with tuned settings. H3, the rates a machine built for the two kernels would need, is not started.

Full history of current-state snapshots lives in `context_convo_notes/current_state_history.md`. The authoritative living document is `primness_roadmap.md`.

## Documentation Maintenance

The README is generated from, and must stay consistent with, the living roadmap and the current-state history. When those documents change on `primness_dev01`, this file is updated in the same pull request so that the repository landing page always reflects the approximate current state of the project.

---

*This README is maintained as part of the Primness project workflow.*
