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
- Documentation conventions are in place (Active Constraints, Decision Log, Falsifiable Prediction templates, open_questions subsection, timestamped notes under `context_convo_notes/`).
- Public repository established with branch protection on `main`.
- Next concrete step: formulate and measure the first falsifiable prediction under D1 on 40–80 digit instances.

Full history of current-state snapshots lives in `context_convo_notes/current_state_history.md`. The authoritative living document is `primness_roadmap.md` (to be added in a subsequent commit).

## Documentation Maintenance

The README is generated from, and must stay consistent with, the living roadmap and the current-state history. When those documents change on `primness_dev01`, this file is updated in the same pull request so that the repository landing page always reflects the approximate current state of the project.

---

*This README is maintained as part of the Primness project workflow.*
