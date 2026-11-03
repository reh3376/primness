# Contributing to Primness

## Branch Policy

Development must not be done directly on the `main` / origin branch.

All work is performed on the `primness_dev01` branch (or feature branches branched from it). Changes are submitted via pull request for personal review, approval, and merge by the repository owner.

The `main` branch is protected: direct pushes are blocked, pull requests require at least one approving review, and force-pushes and deletions are prohibited.

## Process

1. Work on `primness_dev01` or a short-lived feature branch.
2. Update `## Current State` in `primness_roadmap.md`, record that snapshot in `context_convo_notes/current_state_history.md` without removing the previous heading, and update `README.md` in the same change. The roadmap metadata line named by the contract (version `current-state/1` uses `- **Schema**: current-state/1`) selects the file under `schemas/` that states what the check requires.
3. Push the branch. `.github/workflows/auto-pr.yml` opens or refreshes the draft pull request only after `scripts/check-current-state.sh` passes. Do not open the pull request by hand.
4. The owner (reh3376) reviews, approves, and merges.

This ensures a clean, auditable history for the living roadmap, experimental code, and documentation.