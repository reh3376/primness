# D5 on the D1 Instances: No Locatable Clusters; D1 Verdict Drafted (2026-11-10 13:45 EST)

**Metadata**
- **Created**: 2026-11-10 13:45 EST
- **Last Modified**: 2026-11-10 13:45 EST
- **Version**: 1.0
- **Tags**: obstacle-D, D5, D1, smooth-clusters, split-half, null-result, verdict
- **Related Files**: docs/d5_falsifiable_prediction.md, docs/d5_smooth_clusters.md, docs/d1_falsifiable_prediction.md, scripts/d5_clusters.py, scripts/d5_run_all.sh, scripts/d1_conditional.py, src/bin/d1_measure.rs, data/results/d5_clusters.md, data/results/d5_clusters.csv

## Why

D1 closed with nothing left on these instances: the values are as smooth as their sizes and root counts make them, on average over the region. The owner chose D5 next, the sub-claim that smooth values cluster in regions that can be located. It asks the same question region by region, with the same per-value model, so the existing harness can test it. The owner also asked for a draft of the D1 verdict for their review.

## What was built

- `docs/d5_falsifiable_prediction.md`: claim, protocol, control and criterion, written before any D5 statistic. One refinement was made before the first run, again before any statistic: the held-out lift is measured against the half's own ratio, so that an offset of the whole instance (D1's question) stays out of D5.
- `d1-measure --dump` lines now end with ln of the small-prime part of F and of G. The dump runs reproduce the committed cofactor tables byte for byte.
- `scripts/d5_clusters.py`: each value's smooth probability at six bounds from a grid of ln c cells (Q_B plus the first-order term per bin); sums per 50 × 50 tile, per sector and per half; the coarser tilings as unions; dispersion, split-half correlation, held-out lift; the criterion. It runs in 3 minutes for the ten instances, and its totals match D1 to 10⁻⁵ with identical smooth counts.
- `scripts/d5_run_all.sh`: the ten dumps (39 s each, 315 MB, gitignored) and the analysis.
- `first_order_by_bin` in `scripts/d1_conditional.py`, shared by both scripts. D1's summary is unchanged byte for byte.
- `docs/d1_falsifiable_prediction.md`: a draft verdict section marked for the author's approval.

## What was measured

Criterion (algebraic lift over its error / algebraic minus control over the combined error), at the protocol bounds: largest +1.5 / +1.3 (5 × 5 tiles at 2^24); eleven of eighteen algebraic values negative. D5 rejected.

Held-out lift, algebraic, at 2^28: 25 × 25 tiles +0.01% ± 0.15%, 50 × 50 −0.11% ± 0.15%, sectors −0.05% ± 0.20%.

Dispersion (χ² per region), algebraic, protocol bounds: 0.94 to 1.05 for all partitions but 2 × 2; 0.98 to 1.00 on the fine tiles. Split-half correlations within two errors of zero in 35 of 36 cells; the exception is negative.

Rational sectors: χ² per sector 0.84, 0.79, 0.80 ± 0.06 at 2^20, 2^24, 2^28 (0.86, 0.87 at 2^16, 2^18), negative split-half correlation −0.08 to −0.12; d1-02's largest third of sectors 0.35 to 0.38. Not in the rational tiles, not in the algebraic sectors.

## Questions answered

- Do the smooth values cluster in locatable regions beyond their size and root counts? No, at any scale from 2,200 values to a quarter of the region, nor along rays.
- Is counting noise the right noise for regions? Yes for tiles up to 8,800 values (χ² per tile 0.96 to 1.02 on both sides).

## Questions raised

- Why the rational sectors scatter less than counting noise. The likely mechanism is that a region fixes how many of its values each smaller prime divides, which independent draws would not. That does not explain why the sectors show it and tiles of similar size do not.
- The same mechanism would make D1's whole-region counting noise an upper bound: the rational control's χ² at 2^28 is 3.3 on ten. The D1 verdict draft relies on the spread across instances where that is the larger error, and on the noise only at 2^28 after the root counts, where the spread also gives a rejection.
- The remaining sub-claims (D2, D3, D4) each need new machinery. The owner has to choose the next step.

## Hardware

The dump runs: ten d1-measure runs of 39 s each on 18 threads, without hardware sampling (their results went to a temporary directory; the committed results and their traces are unchanged). The analysis is single-threaded Python, 3 minutes.
