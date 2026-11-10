# D1 against the conditional baseline

Baseline run `d1b-18973-1794303061436`: 1,694,498,816 consecutive integers split at the rough bound 2000, ln c in bins of 0.08664. 10 instances. The prediction for a side is the sum over its values of Q_B(c), the baseline density of B-smooth integers among rough integers of the cofactor's size; no α, no α₂, no Dickman function. Rows per instance, side and bound are in `data/results/summary.conditional.csv`. Definitions in `docs/baseline_calibration.md`.

## Algebraic side

Relative gap observed/predicted − 1 per instance, with Q interpolated at the instance's mean ln c in each bin. The last rows: the mean over instances, the same with Q read per bin, the standard error of the mean from the spread across instances, the mean of the per-instance noise (instance count and baseline table together), the mean gap over the spread's standard error, and the mean weighted by each instance's noise with the χ² of the gaps about zero (the noise differs fifty-fold between instances, so the χ² and not the spread says whether the gaps are noise).

| instance | 2^14 | 2^16 | 2^18 | 2^20 | 2^24 | 2^28 |
|---|---:|---:|---:|---:|---:|---:|
| d1-01 | -1.75% (-3.5σ) | -0.33% (-1.0σ) | +0.20% (+0.9σ) | -0.43% (-2.8σ) | -0.39% (-4.1σ) | -0.25% (-3.7σ) |
| d1-02 | +1.89% (+3.1σ) | +1.04% (+2.7σ) | +0.94% (+3.7σ) | +0.45% (+2.5σ) | +0.31% (+2.8σ) | +0.28% (+3.6σ) |
| d1-03 | +1.46% (+1.8σ) | +1.85% (+3.7σ) | +0.54% (+1.6σ) | +0.32% (+1.4σ) | +0.45% (+3.4σ) | +0.26% (+2.9σ) |
| d1-04 | -0.17% (-0.1σ) | -1.25% (-1.7σ) | -1.15% (-2.4σ) | -0.78% (-2.5σ) | -0.31% (-1.8σ) | -0.34% (-3.0σ) |
| d1-05 | -3.13% (-2.2σ) | -2.09% (-2.5σ) | -0.31% (-0.6σ) | -0.21% (-0.6σ) | -0.26% (-1.4σ) | -0.27% (-2.2σ) |
| d1-06 | +4.07% (+2.5σ) | +1.06% (+1.2σ) | +1.50% (+2.8σ) | +1.02% (+2.9σ) | +0.78% (+4.2σ) | +0.39% (+3.3σ) |
| d1-07 | -2.45% (-0.9σ) | +3.30% (+2.3σ) | +1.03% (+1.2σ) | +1.14% (+2.1σ) | +0.94% (+3.5σ) | +0.28% (+1.7σ) |
| d1-08 | +5.99% (+1.6σ) | +7.11% (+3.4σ) | +5.67% (+4.9σ) | +4.58% (+6.5σ) | +3.03% (+9.2σ) | +1.72% (+8.9σ) |
| d1-09 | -12.05% (-3.3σ) | -5.18% (-2.8σ) | -3.59% (-3.5σ) | -3.03% (-4.8σ) | -2.10% (-7.0σ) | -1.18% (-6.6σ) |
| d1-10 | -0.21% (-0.0σ) | +4.02% (+0.6σ) | +1.06% (+0.4σ) | +2.60% (+1.7σ) | +1.05% (+1.7σ) | +0.68% (+2.1σ) |
| **mean** | **-0.64%** | **+0.95%** | **+0.59%** | **+0.57%** | **+0.35%** | **+0.16%** |
| mean, Q per bin | -0.64% | +0.98% | +0.61% | +0.57% | +0.34% | +0.16% |
| spread / √n | 1.56% | 1.09% | 0.73% | 0.64% | 0.42% | 0.24% |
| mean per-instance noise | 3.10% | 1.52% | 0.83% | 0.50% | 0.24% | 0.14% |
| mean / (spread / √n) | -0.4 | +0.9 | +0.8 | +0.9 | +0.8 | +0.7 |
| weighted mean (by 1/noise²) ± its error | -0.08% ± 0.32% | +0.34% ± 0.19% | +0.41% ± 0.13% | +0.06% ± 0.09% | +0.09% ± 0.05% | +0.04% ± 0.03% |
| χ² about zero (dof) | 50.3 (10) | 57.6 (10) | 69.4 (10) | 103.4 (10) | 207.5 (10) | 191.2 (10) |
| instances above zero | 4 of 10 | 6 of 10 | 7 of 10 | 6 of 10 | 6 of 10 | 6 of 10 |
| values outside the table | 0 | 0 | 0 | 0 | 0 | 0 |

**The α tail.** α over the primes in (2000, B] (computed up to the largest bound on file, at most B), the gap it predicts as exp(slope × tail) − 1 with the slope d ln(prediction)/d ln c read from the table, and the gap that remains. Correlation and regression slope are of the observed gap on the predicted one across instances.

| | 2^14 | 2^16 | 2^18 | 2^20 | 2^24 | 2^28 |
|---|---:|---:|---:|---:|---:|---:|
| tail computed up to | 2^14 | 2^16 | 2^18 | 2^20 | 2^24 | 2^28 |
| spread of the tail across instances | 0.046 | 0.047 | 0.046 | 0.048 | 0.048 | 0.047 |
| correlation, gap with tail prediction | +0.95 | +0.93 | +0.97 | +0.95 | +0.98 | +0.98 |
| regression slope | 3.37 | 3.07 | 2.52 | 2.42 | 2.08 | 1.53 |
| spread of the gap | 4.92% | 3.45% | 2.32% | 2.03% | 1.32% | 0.76% |
| spread after the tail | 3.62% | 2.51% | 1.48% | 1.30% | 0.72% | 0.29% |
| **mean gap after the tail** | **-0.35%** | **+0.90%** | **+0.48%** | **+0.47%** | **+0.27%** | **+0.09%** |
| its standard error (spread / √n) | 1.15% | 0.79% | 0.47% | 0.41% | 0.23% | 0.09% |

**The exact first-order term.** The root counts r_p of `d1-tail` at every prime in (2000, B], summed over the values as η_p R(c/p)/R(c) (Q_B(c/p) − Q_B(c)), with η_p = (r_p p − p − 1)/(p² − 1), or (r_p^aff − 1)/(p − 1) over the affine roots alone at the primes above the region's largest b and less 1/(p − 1) where 0 is a root above its largest |a|, and R the density of rough integers (section 9 of the calibration document). Per instance: the term, and the gap that remains after it with its noise in σ.

| instance | 2^14 | 2^16 | 2^18 | 2^20 | 2^24 | 2^28 |
|---|---:|---:|---:|---:|---:|---:|
| d1-01 | -2.30% → +0.55% (+1.1σ) | -0.17% → -0.15% (-0.5σ) | -0.00% → +0.20% (+0.9σ) | -0.31% → -0.12% (-0.8σ) | -0.30% → -0.09% (-1.0σ) | -0.17% → -0.08% (-1.2σ) |
| d1-02 | +1.55% → +0.34% (+0.6σ) | +0.76% → +0.28% (+0.7σ) | +0.85% → +0.09% (+0.4σ) | +0.56% → -0.11% (-0.6σ) | +0.28% → +0.03% (+0.2σ) | +0.20% → +0.08% (+1.0σ) |
| d1-03 | +2.39% → -0.90% (-1.1σ) | +2.29% → -0.43% (-0.9σ) | +0.90% → -0.36% (-1.1σ) | +0.53% → -0.21% (-0.9σ) | +0.36% → +0.09% (+0.7σ) | +0.29% → -0.03% (-0.3σ) |
| d1-04 | -1.76% → +1.62% (+1.3σ) | -1.36% → +0.11% (+0.1σ) | -1.39% → +0.24% (+0.5σ) | -0.96% → +0.19% (+0.6σ) | -0.45% → +0.14% (+0.8σ) | -0.29% → -0.04% (-0.4σ) |
| d1-05 | -2.64% → -0.50% (-0.3σ) | -2.48% → +0.39% (+0.5σ) | -0.95% → +0.64% (+1.2σ) | -0.77% → +0.57% (+1.6σ) | -0.55% → +0.30% (+1.6σ) | -0.36% → +0.10% (+0.8σ) |
| d1-06 | +2.41% → +1.62% (+1.0σ) | +1.48% → -0.41% (-0.5σ) | +1.71% → -0.21% (-0.4σ) | +1.27% → -0.24% (-0.7σ) | +0.85% → -0.08% (-0.4σ) | +0.57% → -0.18% (-1.5σ) |
| d1-07 | -1.93% → -0.53% (-0.2σ) | +1.65% → +1.62% (+1.1σ) | +0.88% → +0.15% (+0.2σ) | +0.79% → +0.35% (+0.7σ) | +0.47% → +0.47% (+1.8σ) | +0.29% → -0.00% (-0.0σ) |
| d1-08 | +3.03% → +2.87% (+0.8σ) | +5.02% → +2.00% (+1.0σ) | +4.49% → +1.13% (+1.0σ) | +3.81% → +0.74% (+1.1σ) | +2.34% → +0.67% (+2.1σ) | +1.58% → +0.14% (+0.7σ) |
| d1-09 | -10.03% → -2.24% (-0.5σ) | -5.87% → +0.73% (+0.4σ) | -3.88% → +0.30% (+0.3σ) | -2.97% → -0.07% (-0.1σ) | -1.87% → -0.23% (-0.8σ) | -1.28% → +0.11% (+0.6σ) |
| d1-10 | +1.79% → -1.97% (-0.1σ) | +0.80% → +3.20% (+0.5σ) | +0.69% → +0.36% (+0.1σ) | +0.75% → +1.83% (+1.2σ) | +0.61% → +0.43% (+0.7σ) | +0.43% → +0.25% (+0.8σ) |

Across instances: correlation and regression slope of the observed gap on the term (a slope of 1 is a full explanation), the spread of the gap before and after, the χ² about zero before and after (consistent with noise when near the number of instances), the mean after, plain and weighted by the noise, with their errors, and the effect of the rough-integer density ratio R(c/p)/R(c).

| | 2^14 | 2^16 | 2^18 | 2^20 | 2^24 | 2^28 |
|---|---:|---:|---:|---:|---:|---:|
| correlation, gap with the term | +0.96 | +0.94 | +0.98 | +0.95 | +0.99 | +0.99 |
| regression slope | 1.19 | 1.10 | 1.05 | 1.11 | 1.19 | 1.01 |
| spread of the gap | 4.92% | 3.45% | 2.32% | 2.03% | 1.32% | 0.76% |
| spread after the term | 1.64% | 1.18% | 0.42% | 0.64% | 0.29% | 0.13% |
| mean per-instance noise | 3.10% | 1.52% | 0.83% | 0.50% | 0.24% | 0.14% |
| χ² about zero before the term (dof) | 50.3 (10) | 57.6 (10) | 69.4 (10) | 103.4 (10) | 207.5 (10) | 191.2 (10) |
| **χ² about zero after the term (dof)** | **6.5** (10) | **4.5** (10) | **5.3** (10) | **8.3** (10) | **13.2** (10) | **7.2** (10) |
| instances beyond 3σ after the term | 0 | 0 | 0 | 0 | 0 | 0 |
| **mean gap after the term** | **+0.09%** | **+0.73%** | **+0.25%** | **+0.29%** | **+0.17%** | **+0.03%** |
| its standard error (spread / √n) | 0.52% | 0.37% | 0.13% | 0.20% | 0.09% | 0.04% |
| **weighted mean after the term ± its error** | **+0.31%** ± 0.32% | **+0.02%** ± 0.19% | **+0.11%** ± 0.13% | **-0.03%** ± 0.09% | **+0.05%** ± 0.05% | **-0.01%** ± 0.03% |
| largest change from R(c/p)/R(c) | 1.113% | 0.285% | 0.116% | 0.053% | 0.010% | 0.004% |

## Rational side

Relative gap observed/predicted − 1 per instance, with Q interpolated at the instance's mean ln c in each bin. The last rows: the mean over instances, the same with Q read per bin, the standard error of the mean from the spread across instances, the mean of the per-instance noise (instance count and baseline table together), the mean gap over the spread's standard error, and the mean weighted by each instance's noise with the χ² of the gaps about zero (the noise differs fifty-fold between instances, so the χ² and not the spread says whether the gaps are noise).

| instance | 2^14 | 2^16 | 2^18 | 2^20 | 2^24 | 2^28 |
|---|---:|---:|---:|---:|---:|---:|
| d1-01 | +0.01% (+0.2σ) | -0.04% (-0.6σ) | +0.01% (+0.2σ) | +0.02% (+0.3σ) | +0.00% (+0.1σ) | +0.02% (+0.7σ) |
| d1-02 | +0.10% (+0.9σ) | +0.03% (+0.3σ) | -0.04% (-0.6σ) | -0.04% (-0.6σ) | -0.03% (-0.9σ) | -0.02% (-0.7σ) |
| d1-03 | -0.17% (-1.0σ) | +0.06% (+0.5σ) | -0.00% (-0.0σ) | -0.03% (-0.5σ) | +0.01% (+0.3σ) | +0.01% (+0.1σ) |
| d1-04 | +0.06% (+0.2σ) | +0.01% (+0.1σ) | -0.09% (-0.7σ) | -0.08% (-0.8σ) | -0.00% (-0.0σ) | -0.00% (-0.0σ) |
| d1-05 | +0.24% (+0.6σ) | +0.13% (+0.5σ) | +0.12% (+0.7σ) | +0.06% (+0.5σ) | -0.03% (-0.4σ) | -0.05% (-0.9σ) |
| d1-06 | -0.51% (-1.2σ) | +0.39% (+1.5σ) | +0.37% (+2.1σ) | +0.23% (+1.8σ) | +0.07% (+0.8σ) | -0.02% (-0.4σ) |
| d1-07 | +0.17% (+0.2σ) | +0.71% (+1.7σ) | +0.41% (+1.6σ) | +0.30% (+1.7σ) | +0.16% (+1.5σ) | +0.00% (+0.0σ) |
| d1-08 | -1.74% (-1.6σ) | -1.04% (-1.9σ) | -0.76% (-2.3σ) | -0.42% (-1.9σ) | +0.04% (+0.3σ) | -0.01% (-0.1σ) |
| d1-09 | +0.73% (+0.6σ) | -0.06% (-0.1σ) | +0.09% (+0.2σ) | +0.27% (+1.1σ) | +0.05% (+0.4σ) | -0.01% (-0.1σ) |
| d1-10 | +4.61% (+1.6σ) | +2.55% (+2.0σ) | +1.76% (+2.7σ) | +0.35% (+0.9σ) | -0.09% (-0.5σ) | -0.10% (-0.8σ) |
| **mean** | **+0.35%** | **+0.27%** | **+0.18%** | **+0.07%** | **+0.02%** | **-0.02%** |
| mean, Q per bin | +0.34% | +0.31% | +0.21% | +0.07% | +0.01% | -0.02% |
| spread / √n | 0.52% | 0.29% | 0.20% | 0.07% | 0.02% | 0.01% |
| mean per-instance noise | 0.74% | 0.38% | 0.23% | 0.16% | 0.09% | 0.06% |
| mean / (spread / √n) | +0.7 | +0.9 | +0.9 | +0.9 | +0.8 | -1.8 |
| weighted mean (by 1/noise²) ± its error | +0.01% ± 0.06% | +0.02% ± 0.04% | +0.01% ± 0.03% | +0.01% ± 0.03% | +0.00% ± 0.02% | -0.00% ± 0.01% |
| χ² about zero (dof) | 9.4 (10) | 13.9 (10) | 20.9 (10) | 13.0 (10) | 4.4 (10) | 2.7 (10) |
| instances above zero | 7 of 10 | 7 of 10 | 6 of 10 | 6 of 10 | 6 of 10 | 3 of 10 |
| values outside the table | 0 | 0 | 0 | 0 | 0 | 0 |

**The α tail.** Below 0.001 for every instance on this side (a linear polynomial has one root at every prime), so nothing is removed.

**The exact first-order term.** The root counts r_p of `d1-tail` at every prime in (2000, B], summed over the values as η_p R(c/p)/R(c) (Q_B(c/p) − Q_B(c)), with η_p = (r_p p − p − 1)/(p² − 1), or (r_p^aff − 1)/(p − 1) over the affine roots alone at the primes above the region's largest b and less 1/(p − 1) where 0 is a root above its largest |a|, and R the density of rough integers (section 9 of the calibration document). Per instance: the term, and the gap that remains after it with its noise in σ.

| instance | 2^14 | 2^16 | 2^18 | 2^20 | 2^24 | 2^28 |
|---|---:|---:|---:|---:|---:|---:|
| d1-01 | +0.00% → +0.01% (+0.2σ) | +0.00% → -0.04% (-0.6σ) | +0.00% → +0.01% (+0.2σ) | +0.00% → +0.02% (+0.3σ) | +0.00% → +0.00% (+0.1σ) | -0.00% → +0.02% (+0.7σ) |
| d1-02 | +0.00% → +0.10% (+0.9σ) | +0.00% → +0.03% (+0.3σ) | +0.00% → -0.04% (-0.6σ) | +0.00% → -0.04% (-0.6σ) | +0.00% → -0.03% (-0.9σ) | +0.00% → -0.02% (-0.7σ) |
| d1-03 | +0.00% → -0.17% (-1.0σ) | +0.00% → +0.06% (+0.5σ) | +0.00% → -0.00% (-0.0σ) | +0.00% → -0.03% (-0.5σ) | +0.00% → +0.01% (+0.3σ) | -0.00% → +0.01% (+0.1σ) |
| d1-04 | +0.00% → +0.06% (+0.2σ) | -0.01% → +0.02% (+0.1σ) | -0.01% → -0.08% (-0.6σ) | -0.01% → -0.07% (-0.7σ) | -0.00% → +0.00% (+0.0σ) | -0.00% → +0.00% (+0.0σ) |
| d1-05 | +0.00% → +0.24% (+0.6σ) | +0.00% → +0.13% (+0.5σ) | +0.00% → +0.12% (+0.7σ) | +0.00% → +0.06% (+0.5σ) | +0.00% → -0.03% (-0.4σ) | +0.00% → -0.05% (-0.9σ) |
| d1-06 | +0.00% → -0.51% (-1.2σ) | +0.00% → +0.39% (+1.5σ) | +0.00% → +0.37% (+2.1σ) | +0.00% → +0.23% (+1.8σ) | -0.00% → +0.07% (+0.8σ) | -0.00% → -0.02% (-0.4σ) |
| d1-07 | +0.00% → +0.17% (+0.2σ) | +0.00% → +0.71% (+1.7σ) | +0.00% → +0.41% (+1.6σ) | +0.00% → +0.30% (+1.7σ) | +0.00% → +0.16% (+1.5σ) | +0.00% → +0.00% (+0.0σ) |
| d1-08 | -0.52% → -1.23% (-1.1σ) | -0.35% → -0.69% (-1.2σ) | -0.25% → -0.51% (-1.5σ) | -0.19% → -0.24% (-1.0σ) | -0.11% → +0.15% (+1.2σ) | -0.08% → +0.07% (+0.8σ) |
| d1-09 | -0.36% → +1.09% (+0.9σ) | -0.24% → +0.18% (+0.3σ) | -0.17% → +0.25% (+0.7σ) | -0.12% → +0.40% (+1.6σ) | -0.07% → +0.13% (+0.9σ) | -0.05% → +0.04% (+0.5σ) |
| d1-10 | -0.10% → +4.72% (+1.6σ) | -0.13% → +2.69% (+2.1σ) | -0.09% → +1.85% (+2.8σ) | -0.06% → +0.41% (+1.0σ) | -0.04% → -0.05% (-0.3σ) | -0.02% → -0.08% (-0.7σ) |

Across instances: correlation and regression slope of the observed gap on the term (a slope of 1 is a full explanation), the spread of the gap before and after, the χ² about zero before and after (consistent with noise when near the number of instances), the mean after, plain and weighted by the noise, with their errors, and the effect of the rough-integer density ratio R(c/p)/R(c).

| | 2^14 | 2^16 | 2^18 | 2^20 | 2^24 | 2^28 |
|---|---:|---:|---:|---:|---:|---:|
| correlation, gap with the term | +0.19 | +0.23 | +0.23 | +0.35 | -0.05 | +0.06 |
| regression slope | 1.62 | 1.67 | 1.63 | 1.22 | -0.09 | 0.08 |
| spread of the gap | 1.63% | 0.92% | 0.64% | 0.23% | 0.07% | 0.03% |
| spread after the term | 1.61% | 0.90% | 0.62% | 0.22% | 0.08% | 0.04% |
| mean per-instance noise | 0.74% | 0.38% | 0.23% | 0.16% | 0.09% | 0.06% |
| χ² about zero before the term (dof) | 9.4 (10) | 13.9 (10) | 20.9 (10) | 13.0 (10) | 4.4 (10) | 2.7 (10) |
| **χ² about zero after the term (dof)** | **8.6** (10) | **12.5** (10) | **19.1** (10) | **12.2** (10) | **6.4** (10) | **3.3** (10) |
| instances beyond 3σ after the term | 0 | 0 | 0 | 0 | 0 | 0 |
| **mean gap after the term** | **+0.45%** | **+0.35%** | **+0.24%** | **+0.10%** | **+0.04%** | **-0.00%** |
| its standard error (spread / √n) | 0.51% | 0.28% | 0.20% | 0.07% | 0.03% | 0.01% |
| **weighted mean after the term ± its error** | **+0.01%** ± 0.06% | **+0.02%** ± 0.04% | **+0.02%** ± 0.03% | **+0.01%** ± 0.03% | **+0.01%** ± 0.02% | **-0.00%** ± 0.01% |
| largest change from R(c/p)/R(c) | 0.059% | 0.022% | 0.011% | 0.006% | 0.001% | 0.000% |

