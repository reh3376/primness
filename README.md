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
- Next concrete step: decide what follows D1's null result at this scale. The open check is the exact first-order term of the primes between 2000 and B from the root counts, which should account for the regression slope above 1; then the criterion weighting in the prediction document, and the A4 test or another sub-claim.

Full history of current-state snapshots lives in `context_convo_notes/current_state_history.md`. The authoritative living document is `primness_roadmap.md`.

## Documentation Maintenance

The README is generated from, and must stay consistent with, the living roadmap and the current-state history. When those documents change on `primness_dev01`, this file is updated in the same pull request so that the repository landing page always reflects the approximate current state of the project.

---

*This README is maintained as part of the Primness project workflow.*
