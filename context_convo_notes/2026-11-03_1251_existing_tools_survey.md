# Primness Notes: Careful Survey of Existing Tools for Obstacle D / D1

**Metadata**
- **Created**: 2026-11-03 12:51 EST
- **Last Modified**: 2026-11-03 12:51 EST
- **Version**: 1.0
- **Tags**: obstacle-D, D1, existing-tools, CADO-NFS, msieve, YAFU, measurement-baseline
- **Related Files**: primness_roadmap.md
- **Context**: Instrument existing implementations without accepting their stated constraints as facts

## Purpose

We will use selected open-source and university implementations solely as measurement instruments and sources of candidate ideas while we formulate the first falsifiable prediction under D1. Every published bound, parameter heuristic, or “best polynomial” is treated as an experimental observation to be tested on our own 40–80-digit instances, not as a fact.

## Tools and exact interfaces we can instrument

### CADO-NFS
- Polynomial output: CADO polynomial file format (or convertible to msieve/GGNFS `.poly`).
- Key scores: Murphy E value after root optimization (Bai–Bouvier–Kruppa–Zimmermann).
- Relation files: standard format shared with msieve and GGNFS.
- Use: generate reference polynomials and relation yields; measure smoothness statistics of the polynomial values ourselves rather than accepting CADO’s internal estimates.

### msieve
- Polynomial selection: multi-stage Kleinjung-style pipeline; output `.poly` files.
- Relation format: identical to CADO/GGNFS.
- Use: independent polyselect baseline for the lower end of the 40–80-digit range; instrument stage-1/2/3 coefficient distributions.

### YAFU
- Orchestration layer that calls CADO-NFS and msieve.
- Use: rapid generation of comparison runs on laptop-plus-GPU hardware; extract the polynomials and relations it produces and re-evaluate smoothness independently.

### Rust educational GNFS ports (e.g. danielrcurtis/gnfs)
- Readable scaffolds for polynomial evaluation and sieving loops.
- Use: only as structural reference for our own Rust kernels; performance numbers are ignored.

### University GPU explorations (Paderborn CUDA-MPQS / early CUDA-GNFS)
- Concrete kernel timings only.
- Use: timing reference for sieving primitives; no acceptance of claimed speed-ups as changes to the mathematical obstacle.

## First measurement protocol (does not inherit tool limits)

1. Select 10 hard balanced semiprimes in the 40–80-digit range.
2. Obtain one “best” polynomial from CADO-NFS and one from msieve for each.
3. Independently evaluate the polynomial values over a fixed sieve region and compute the observed B-smooth density for several B.
4. Compare the observed density to the Dickman–de Bruijn prediction (adjusted only by the measured Murphy α / root properties, not by the tools’ internal scores).
5. Record the gap as a falsifiable claim under D1; rejection criterion = gap smaller than the measurement noise on these instances.

## Working rule

No stated constraint, crossover size, or asymptotic constant from these projects is adopted. Our job remains the invention of novel structure that improves upon or replaces the current sieve approach.

## Next action

Open this note as a PR for review. After merge, implement the measurement harness on `primness_dev01` in Rust (with optional Python sidecar only for post-processing statistics).
