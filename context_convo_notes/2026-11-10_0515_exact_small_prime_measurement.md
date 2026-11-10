# The Exact Small-Prime Measurement: No Excess, and the α Tail (2026-11-10 05:15 EST)

**Metadata**
- **Created**: 2026-11-10 05:15 EST
- **Last Modified**: 2026-11-10 05:15 EST
- **Version**: 1.0
- **Tags**: obstacle-D, D1, conditional-density, cofactor, alpha-tail, rejection-criterion, null-result
- **Related Files**: docs/baseline_calibration.md, src/cofactor.rs, src/bin/d1_measure.rs, src/bin/d1_baseline.rs, src/alpha.rs, scripts/d1_conditional.py, scripts/d1_run_all.sh, schemas/d1-measurement/4.json, data/results/summary.conditional.md, data/results/summary.conditional.csv, data/results/alpha_moments.b24.json

## Why

The table Q_B(c) existed (note of 04:10). The instrument side was missing: split every polynomial value the same way and compare its smoothness with Q_B of its cofactor.

## What was built

- `src/cofactor.rs`: the bin rule and `SmallPart`, trial division by the 303 primes up to 2000 grouped into word-sized products. Shared by both binaries.
- `d1-measure`: per value and side, the cofactor bin; table `cofactor`; result `d1-measurement/4`. Smooth counts of all twenty runs unchanged; run time unchanged.
- `scripts/d1_conditional.py`: prediction = sum of Q_B over the values, interpolated between neighbouring bins and never across c = B; gaps, noise, and the α tail.
- Bin width changed from 0.25 to ln 2 / 8. Q jumps at c = B (to 0.31 at 2^28); with 0.25 the bin holding the jump put −0.11% (3 standard errors) on the rational side of d1-01 at 2^28. With ln 2 over an integer every power-of-two bound is an edge. The baseline was rerun (1,285 bins, 94 s). This supersedes the bin width and the per-bin figures in the 04:10 note.
- `src/alpha.rs`: root enumeration through `roots_mod_p` in place of a scan of all residues. Output identical at 2000 and 2^14. α at 2^24 takes four minutes for the ten instances; 2^28 was started and stopped (about an hour).
- `scripts/d1_run_all.sh --remeasure`: repeat the measurements on the chosen polynomials without polynomial selection.

## What was measured

Mean relative gap over ten instances, ± spread / √10.

| | 2^14 | 2^16 | 2^18 | 2^20 | 2^24 | 2^28 |
|---|---:|---:|---:|---:|---:|---:|
| rational | +0.35 ± 0.52% | +0.27 ± 0.29% | +0.18 ± 0.20% | +0.07 ± 0.07% | +0.02 ± 0.02% | −0.02 ± 0.01% |
| algebraic | −0.64 ± 1.56% | +0.95 ± 1.09% | +0.59 ± 0.73% | +0.57 ± 0.64% | +0.35 ± 0.42% | +0.16 ± 0.24% |
| algebraic after the α tail | −0.35 ± 1.15% | +0.90 ± 0.79% | +0.48 ± 0.47% | +0.47 ± 0.41% | +0.27 ± 0.23% | +0.10 ± 0.09% |

Per instance at 2^28, algebraic: d1-01 −0.25%, d1-02 +0.28%, d1-03 +0.26%, d1-04 −0.34%, d1-05 −0.27%, d1-06 +0.39%, d1-07 +0.28%, d1-08 +1.72%, d1-09 −1.18%, d1-10 +0.68%. Per-instance noise 0.07% to 0.3%.

α tail (α at B minus α at 2000) per instance at 2^24: from −0.094 (d1-08) to +0.085 (d1-09), spread 0.048. Correlation of the gap with the tail's first-order prediction: 0.95, 0.93, 0.97, 0.95, 0.98, 0.98. Regression slope: 3.37, 3.07, 2.52, 2.42, 2.08, 1.53.

## Questions answered

- What carried the +1.2% at 2^24 and +0.5% at 2^28 left after α and α₂? Mostly the expansion. The exact measurement leaves +0.35% and +0.16%, within one standard error of zero.
- Is the table precise enough? Yes: its share of an instance's noise is about an eighth of the instance's own counting noise.
- Is there structure between the α bound and B? Per polynomial, yes, with both signs, and it is α continued. On average over polynomials, none detected.

## Questions raised

- Why the regression slope exceeds 1. Candidate: the linearization of a prime-sized step in ln c.
- Whether the mean after the tail (+0.10 ± 0.09% at 2^28, positive at five of six bounds) is real.
- What D1's null result at this scale means for the order of the other sub-claims.

## Hardware

Baseline run `d1b-18973-1794303061436`: 91 samples over 93 s; CPU activity mean 0.50, memory used 0.83 of 128 GB (other processes included), CPU temperature peak 80.7 °C. The twenty instrument runs carry their own traces in their result files.
