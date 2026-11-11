# Falsifiable Prediction under Obstacle D / D1

**Claim (D1)**  
On hard balanced semiprimes of 40–80 decimal digits, the observed density of B-smooth values of an optimized degree-4 or degree-5 polynomial (Murphy-E maximized) exceeds the Dickman–de Bruijn prediction by a measurable margin after accounting only for the measured root properties (Murphy α). The excess is large enough to be distinguished from measurement noise on a fixed sieve region.

**Measurement protocol**
1. Select 10 hard balanced semiprimes N = p·q in the 40–80 digit range (no special algebraic form).
2. For each N obtain one polynomial from CADO-NFS and one from msieve (external instruments only).
3. Independently evaluate the homogeneous polynomial values over a fixed rectangular sieve region (a,b) of size sufficient for the digit length.
4. For several smoothness bounds B (e.g. 2^20, 2^24, 2^28), count the observed fraction of B-smooth values.
5. Compare the observed fraction to the Dickman–de Bruijn ρ(u) prediction, adjusting only by the measured Murphy α of the polynomial. Do not use the tools’ internal scores.
6. Record the gap (observed − predicted) and its variance across the 10 instances.

**Rejection criterion**  
The claim is rejected if, on these instances, the mean gap is smaller than two standard deviations of the measurement noise (determined by re-sampling the sieve region or by bootstrap).

**Hardware envelope**  
All measurements must complete on the primary laptop+GPU constraints stated in the roadmap. No distributed cluster is assumed.

**Relation to roadmap**  
This is the first concrete, testable claim under Obstacle D. A positive result weakens D; a negative result (rejection) forces a pivot or refinement of the sub-claim before larger speculation.

**Verdict (adopted 2026-11-10)**

*Drafted from the measurements for the author, and adopted on 2026-11-10 when the author closed the D measurements (`docs/obstacle_d_verdict.md`). The claim is rejected.*

- **Weighting.** The gap is relative, observed/predicted − 1, and the ten instances are weighted alike (`docs/baseline_calibration.md` section 6). The noise is the larger of the binomial noise and the spread across instances.
- **Prediction.** "Dickman–de Bruijn adjusted only by the measured Murphy α" is implemented exactly rather than through α's expansion. Every value is split into its part made of the primes up to 2000 and a cofactor c, and its smoothness is predicted by the density Q_B(c) of B-smooth integers among consecutive integers with no prime factor up to 2000 and a cofactor of the same size (sections 7 and 8). The polynomial's root counts at the primes between 2000 and B, α's continuation, are then applied to first order (section 9).
- **Result at the protocol's bounds**, mean relative gap over the ten instances ± the spread over √10:

  | | 2^20 | 2^24 | 2^28 |
  |---|---:|---:|---:|
  | after the exact small-prime split | +0.57% ± 0.64% | +0.35% ± 0.42% | +0.16% ± 0.24% |
  | after the root counts to B as well | +0.29% ± 0.20% | +0.17% ± 0.09% | +0.03% ± 0.04% |

  The mean gap is below two standard errors at every bound under both readings, so **the claim is rejected** at 2^20, 2^24 and 2^28. The margin is smallest at 2^24 after the root counts, where the gap is +0.17% against a threshold of 0.18%. Weighted by each instance's noise instead, the gaps after the root counts are −0.03% ± 0.09%, +0.05% ± 0.05% and −0.01% ± 0.03%. The ten residuals are consistent with counting noise at every bound (χ² 4.5 to 13.2 on ten).
- **Departures from the protocol.**
  - There is one polynomial per instance, from CADO-NFS. msieve enters NFS polynomial selection only above 264 bits.
  - The region is fixed at a in [−3000, 3000], b in [1, 1500], labelled scaled-down.
  - The noise is the binomial model checked by the χ² above, not a resampling of the region.
  - The binomial noise holds for regions of up to about 9,000 values: in D5 (`docs/d5_smooth_clusters.md`) the χ² per tile is 0.96 to 1.02 on 25 × 25 and 50 × 50 tiles, on both sides. For whole regions it may be too large, since the rational control's χ² at 2^28 is 3.3 on ten. If it is, the per-instance errors are upper bounds. The table above uses the spread across instances wherever that is the larger error, which is everywhere except 2^28 after the root counts; with the spread there, the gap is +0.03% ± 0.04%, still below two errors.
