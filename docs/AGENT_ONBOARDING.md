# Primness Agent Onboarding

**Purpose**  
Bring a new agent (Grok Build or equivalent) fully up to speed on the Primness project so work under Obstacle D can continue without loss of context or violation of established rules.

**Last updated**: 2026-11-03

## Project summary
Primness seeks a practical integer-factorization improvement beyond the GNFS by identifying deeper mathematical structure (Obstacle D). Preferred starting point is sub-claim D1: independent verification of whether optimized polynomial values exhibit a measurable excess of B-smooth density over the Dickman–de Bruijn model on hard 40–80 digit balanced semiprimes.

External tools (CADO-NFS, msieve, YAFU) are measurement instruments only. Their heuristics and stated constraints are never adopted as facts.

## Authoritative documents (read these first)
- `primness_roadmap.md` — problem statement, constraints, obstacles A–H, importance, and living Current State section.
- `context_convo_notes/INDEX.md` — index of timestamped notes.
- `context_convo_notes/current_state_history.md` — history of every Current State snapshot.
- `docs/d1_falsifiable_prediction.md` — first precise claim, measurement protocol, and rejection criterion.
- `docs/hardware.md` — Mac M5 Max primary envelope; Docker and dashboards permitted when useful.
- `tools/README.md` — external instruments and relation format.

## Current state (as of handoff)
- Development occurs exclusively on protected branch `primness_dev01`.
- Auto-PR workflow opens draft PRs authored by github-actions[bot]; owner reviews, approves, and merges.
- PR descriptions are detailed and reference living documents.
- CI (`.github/workflows/ci.yml`) runs `cargo check` and `cargo test` on PRs; checks remain dynamic and non-brittle.
- Rust core + Python sidecar (as needed for ML). Instance generator and measurement harness (`d1_generate_instances`, `d1_measure`) are at toy/scaling stage.
- First falsifiable prediction under D1 is written. The ten semiprimes are proved in `data/d1_test_instances.md`. The smoothness runs on those instances have not been executed.
- Primary hardware: Mac M5 Max. All measurements must respect this machine or be explicitly marked scaled-down.

## Workflow rules (must be followed)
1. Never develop on `main`. Use `primness_dev01`.
2. Changes accumulate until a coherent segmentation point; the auto-PR workflow opens or updates the draft PR.
3. PR descriptions must be detailed and reference relevant living docs.
4. Owner reviews, approves, and merges. Do not self-approve.
5. Every pull request updates `## Current State` in `primness_roadmap.md` and adds that snapshot to `context_convo_notes/current_state_history.md`, leaving the previous `###` heading in place. Auto-PR will not open or refresh the draft until `scripts/check-current-state.sh` passes. That check applies the JSON Schema named by the roadmap metadata `Schema` line (currently `current-state/1`). README.md is updated in the same pull request.
6. Progress = removing or substantially weakening an obstacle, or demonstrating a forced pivot to specialized hardware (Obstacle H).
7. Preserve first-person voice when editing the roadmap.
8. Prefer small-to-medium instances first so results can be compared against existing tools.

## Key directories and files
- `src/bin/` — Rust binaries (`d1_measure.rs`, `d1_generate_instances.rs`).
- `data/d1_test_instances.md` — ten proved 40–80 digit semiprimes and their prime certificates.
- `docs/` — prediction, hardware, this onboarding doc.
- `context_convo_notes/` — all historical notes and INDEX.
- `.github/workflows/` — auto-pr.yml and ci.yml.
- `scripts/check-current-state.sh` — refuses an Auto-PR draft unless the head commit satisfies the current-state contract it names. Version `current-state/1` is `schemas/current-state/1.json`.

## Immediate next work under D1
1. Obtain one polynomial per proved instance from CADO-NFS and one from msieve. The harness already parses those files (`instrument-polynomial/1`). Then run the independent B-smooth density count and Dickman comparison inside the harness.
2. Apply the rejection criterion; record the result in a timestamped note and update Current State.

## Integrations already in place
- GitHub Actions auto-PR and CI.
- Cargo workspace with `num-bigint`.
- Living documentation conventions (Active Constraints, Decision Log, Falsifiable Prediction template, open_questions).

## Handoff checklist for the new agent
- [ ] Read `primness_roadmap.md` Current State and open_questions.
- [ ] Read `docs/d1_falsifiable_prediction.md` and `docs/hardware.md`.
- [ ] Confirm local branch is `primness_dev01` and is up to date with remote.
- [ ] Verify CI is green on the latest PR before adding substantial new code.
- [ ] Continue from the “Immediate next work” list above; open or update the draft PR at the next coherent segmentation point.

This document plus the living files listed above constitute the complete transferable context.
