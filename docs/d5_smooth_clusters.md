# D5 on the D1 instances: smooth values region by region

`docs/d5_falsifiable_prediction.md` states the claim, the protocol and the rejection criterion, written before any statistic below was computed. This document records the method as built and the result.

## 1. Method

**Values.** `d1-measure --dump` writes one line per value of the D1 region: a in [−3000, 3000], b in [1, 1500], gcd(a, b) = 1, which is 5,472,915 values per instance. Each line has a and b, and for each side ln|F|, the largest prime factor when the value is 2^28-smooth (else 0), and, new with this change, the log of the part made of the primes up to 2000. The largest prime factor gives smoothness at every bound from 2^14 to 2^28 from the one run. The dump runs reproduce the committed cofactor tables byte for byte. `scripts/d5_run_all.sh` writes the dumps to `data/dumps/` (gitignored, 315 MB per instance, 39 s each) and then runs the analysis.

**Prediction per value.** π_B(v) = Q_B(c_v) + s_B(k_v). Q_B is the conditional baseline read at the value's own ln c, interpolated as `scripts/d1_conditional.py` reads it, on cells of an eighth of a cofactor bin, so that no cell straddles the jump at c = B. s_B(k) is the exact first-order root-count term of section 9 of `docs/baseline_calibration.md` for the cofactor bin k (`first_order_by_bin`). Summed over an instance, π reproduces D1's prediction to one part in 10^5 at every bound, and the smooth counts are D1's exactly (checked on d1-01).

**Regions and halves.** There are five tilings into n × n tiles for n = 2, 5, 10, 25 and 50, all unions of the 50 × 50 grid. Their tiles hold about 1.37 million, 219,000, 55,000, 8,800 and 2,200 values. There are also 64 sectors of equal angle, about 85,000 values each on average. A splitmix64 hash of (a, b) puts every value in one of two halves.

**Statistics** (`scripts/d5_clusters.py`), per instance, side, bound and partition, with n, O, P and V for each region and half:

- the dispersion χ²/R;
- the Pearson correlation between the halves of z = (O − P)/√V across regions;
- the held-out lift. The tenth of the regions with the largest z in one half is read on the other half as (O/P there)/(O/P over that half) − 1, and the two directions are averaged.

The analysis takes three minutes for the ten instances.

## 2. Result

**The criterion.** The table gives, for the algebraic side, the mean held-out lift over its error, then the algebraic minus the rational lift over their combined error. Support needs both above 3.

| partition | 2^20 | | 2^24 | | 2^28 | |
|---|---:|---:|---:|---:|---:|---:|
| | algebraic | minus control | algebraic | minus control | algebraic | minus control |
| 2 × 2 tiles | −0.8 | −0.7 | −0.7 | −0.3 | −0.2 | +0.1 |
| 5 × 5 tiles | −0.7 | −0.7 | +1.5 | +1.3 | +0.9 | +0.6 |
| 10 × 10 tiles | −0.2 | +0.1 | −0.6 | −0.1 | −1.2 | −1.2 |
| 25 × 25 tiles | +0.6 | +0.6 | −0.4 | −0.3 | +0.1 | −0.0 |
| 50 × 50 tiles | +0.6 | +0.7 | +0.1 | −0.2 | −0.7 | −0.9 |
| 64 sectors | −1.9 | −1.2 | +0.0 | +0.2 | −0.2 | −0.2 |

**The claim is rejected on these instances.** The largest value is +1.5, for 5 × 5 tiles at 2^24, and eleven of the eighteen algebraic values are negative. In size, the held-out lift of the algebraic side at 2^28 is +0.01% ± 0.15% on 25 × 25 tiles, −0.11% ± 0.15% on 50 × 50 tiles and −0.05% ± 0.20% on sectors. At 2^24 it is −0.13% ± 0.36%, +0.02% ± 0.26% and +0.01% ± 0.25%. A region picked for an excess of smooth values in one half shows none in the other.

**Dispersion and correlation, algebraic side.** The regions scatter as counting noise says:

- χ² per region is 0.94 to 1.05 at the protocol bounds for every partition except the 2 × 2 tiles, whose four regions give 0.90 to 1.16 ± 0.22–0.26. On the 25 × 25 and 50 × 50 tiles it is 0.98 to 1.00 ± 0.01–0.03.
- The split-half correlations are within two errors of zero in 35 of 36 cells. The exception is negative, 50 × 50 tiles at 2^24 (−0.014 ± 0.006), and the largest positive value is +0.043 ± 0.032. Clustering would make them positive.
- At the small bounds, where few values are smooth, a single tile can carry an instance's χ². d1-10 gives 2.09 on the 50 × 50 tiles at 2^14, and the mean of 1.07 is 0.6 of its spread-based error. The errors in the summary are the larger of the counting-noise error and the spread across instances over √10.

**The control.** The rational side's lift is within two errors of zero at the protocol bounds, except on sectors at 2^20 (−0.46% ± 0.18%), which is negative. Its tiles scatter as counting noise (25 × 25 and 50 × 50: 0.96 to 1.00). Its 64 sectors scatter less: χ² per sector 0.84, 0.79 and 0.80 ± 0.06 at 2^20, 2^24 and 2^28, and 0.86 and 0.87 at 2^16 and 2^18. Their split-half correlation is correspondingly negative (−0.08 to −0.12). In d1-02, the strongest case, the largest third of the sectors gives 0.35 to 0.38. Tiles of similar size (10 × 10, 55,000 values) and the algebraic sectors (0.95 to 1.04) do not show it.

Less scatter than counting noise is the opposite of clustering, and it makes the control's sector errors conservative, so it does not bear on the verdict. A region whose values are not independent draws can show it: the region fixes how many of its values each prime smaller than itself divides, where independent draws would let those numbers vary. Why the rational form's sectors show it and its tiles do not is open.

**Bearing on D1's noise.** D1 used the same counting noise for the whole region of an instance. On tiles of up to 8,800 values that noise is right to 1–2%. For the largest regions there is a hint that it is too large: the rational control's χ² over the ten instances at 2^28 is 3.3 on ten, and its quarter-region tiles give 0.74 and 0.61 ± 0.22 at 2^24 and 2^28. If so, D1's per-instance errors are upper bounds and its χ² is a lower bound. The D1 verdict draft uses the spread across instances, which does not depend on that noise, at 2^24 and 2^28.

## 3. Reading

Over the D1 region of ten optimized degree-4 polynomials, the smooth values sit where their sizes and root counts put them, at scales from 2,200 values to a quarter of the region and along rays from the origin. The B-smooth values form no excess that one half of the values can locate and the other confirm. A held-out excess larger than about 0.5% at 2^28, or 1% at 2^24, would have reached the criterion's threshold of three errors on the fine tiles, and none appeared. D5's claim is rejected at this scale.

D5 asked whether the root-property model is wrong locally in a way that can be found in advance. The answer here is the same as D1's: no. Structure that would move the GNFS exponent would have to live somewhere this measurement does not look. It could appear at sizes far beyond 80 digits, in regions shaped by the lattice of a special q rather than by the rectangle, or in relations between the two sides, which both D1 and D5 treat separately.

## Files

| File | Role |
|---|---|
| `docs/d5_falsifiable_prediction.md` | The claim, the protocol and the criterion, fixed before measuring. |
| `scripts/d5_clusters.py` | Per-value prediction from the dump, sums per region and half, the three statistics and the criterion; writes `data/results/d5_clusters.{csv,md}`. |
| `scripts/d5_run_all.sh` | The dumps (`d1-measure --dump`, checked against the committed cofactor tables) and the analysis. |
| `src/bin/d1_measure.rs` | The dump now carries ln of the part made of the primes up to the rough bound, per side. |
| `scripts/d1_conditional.py` | `first_order_by_bin`: the first-order term per cofactor bin, shared by both scripts. |
| `data/results/d5_clusters.{csv,md}` | 720 rows: instance, side, bound, partition; the summary and the criterion. |
