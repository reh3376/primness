# D1 against the conditional baseline

Baseline run `d1b-18973-1794303061436`: 1,694,498,816 consecutive integers split at the rough bound 2000, ln c in bins of 0.08664. 10 instances. The prediction for a side is the sum over its values of Q_B(c), the baseline density of B-smooth integers among rough integers of the cofactor's size; no α, no α₂, no Dickman function. Rows per instance, side and bound are in `data/results/summary.conditional.csv`. Definitions in `docs/baseline_calibration.md`.

## Algebraic side

Relative gap observed/predicted − 1 per instance, with Q interpolated at the instance's mean ln c in each bin. The last rows: the mean over instances, the same with Q read per bin, the standard error of the mean from the spread across instances, the mean of the per-instance noise (instance count and baseline table together), and the mean gap over the spread's standard error.

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
| instances above zero | 4 of 10 | 6 of 10 | 7 of 10 | 6 of 10 | 6 of 10 | 6 of 10 |
| values outside the table | 0 | 0 | 0 | 0 | 0 | 0 |

**The α tail.** α over the primes in (2000, B] (computed up to the largest bound on file, at most B), the gap it predicts as exp(slope × tail) − 1 with the slope d ln(prediction)/d ln c read from the table, and the gap that remains. Correlation and regression slope are of the observed gap on the predicted one across instances.

| | 2^14 | 2^16 | 2^18 | 2^20 | 2^24 | 2^28 |
|---|---:|---:|---:|---:|---:|---:|
| tail computed up to | 2^14 | 2^16 | 2^18 | 2^20 | 2^24 | 2^24 |
| spread of the tail across instances | 0.046 | 0.047 | 0.046 | 0.048 | 0.048 | 0.048 |
| correlation, gap with tail prediction | +0.95 | +0.93 | +0.97 | +0.95 | +0.98 | +0.98 |
| regression slope | 3.37 | 3.07 | 2.52 | 2.42 | 2.08 | 1.53 |
| spread of the gap | 4.92% | 3.45% | 2.32% | 2.03% | 1.32% | 0.76% |
| spread after the tail | 3.62% | 2.51% | 1.48% | 1.30% | 0.72% | 0.29% |
| **mean gap after the tail** | **-0.35%** | **+0.90%** | **+0.48%** | **+0.47%** | **+0.27%** | **+0.10%** |
| its standard error (spread / √n) | 1.15% | 0.79% | 0.47% | 0.41% | 0.23% | 0.09% |

## Rational side

Relative gap observed/predicted − 1 per instance, with Q interpolated at the instance's mean ln c in each bin. The last rows: the mean over instances, the same with Q read per bin, the standard error of the mean from the spread across instances, the mean of the per-instance noise (instance count and baseline table together), and the mean gap over the spread's standard error.

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
| instances above zero | 7 of 10 | 7 of 10 | 6 of 10 | 6 of 10 | 6 of 10 | 3 of 10 |
| values outside the table | 0 | 0 | 0 | 0 | 0 | 0 |

**The α tail.** Below 0.001 for every instance on this side (a linear polynomial has one root at every prime), so nothing is removed.

