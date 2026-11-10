# D5: smooth values region by region

10 instances, the D1 region. Each value's smooth probability is the conditional baseline Q_B at its cofactor plus the first-order root-count term (`scripts/d1_conditional.py`); the protocol and the criterion are `docs/d5_falsifiable_prediction.md`, the reading `docs/d5_smooth_clusters.md`. Rows per instance, side, bound and partition are in `data/results/d5_clusters.csv`.

## Algebraic side

**Held-out lift**: the tenth of the regions with the largest excess in one half, read on the other half against that half as a whole, mean of both directions and of the ten instances, ± the larger of the spread and the noise over √10.

| partition | 2^14 | 2^16 | 2^18 | 2^20 | 2^24 | 2^28 |
|---|---:|---:|---:|---:|---:|---:|
| 2x2 tiles | +49.20% ± 47.91% | +3.04% ± 2.43% | -0.11% ± 0.63% | -0.31% ± 0.41% | -0.17% ± 0.24% | -0.02% ± 0.12% |
| 5x5 tiles | -12.17% ± 11.13% | -8.99% ± 7.32% | -1.67% ± 1.05% | -0.44% ± 0.64% | +0.43% ± 0.29% | +0.29% ± 0.33% |
| 10x10 tiles | -1.35% ± 4.55% | -1.46% ± 2.93% | -0.58% ± 0.87% | -0.12% ± 0.54% | -0.15% ± 0.26% | -0.36% ± 0.29% |
| 25x25 tiles | -2.25% ± 3.41% | +0.30% ± 1.12% | -2.40% ± 1.12% | +0.64% ± 1.05% | -0.13% ± 0.36% | +0.01% ± 0.15% |
| 50x50 tiles | -6.04% ± 4.49% | +0.72% ± 2.66% | -0.47% ± 1.18% | +0.74% ± 1.22% | +0.02% ± 0.26% | -0.11% ± 0.15% |
| 64 sectors | -2.24% ± 4.04% | -6.88% ± 5.40% | -2.69% ± 1.54% | -1.40% ± 0.75% | +0.01% ± 0.25% | -0.05% ± 0.20% |

**Split-half correlation** of the regions' excess, mean over instances ± the larger of its error with no correlation and the spread over √10:

| partition | 2^14 | 2^16 | 2^18 | 2^20 | 2^24 | 2^28 |
|---|---:|---:|---:|---:|---:|---:|
| 2x2 tiles | +0.107 ± 0.183 | +0.035 ± 0.211 | -0.084 ± 0.183 | -0.022 ± 0.183 | -0.331 ± 0.183 | -0.239 ± 0.183 |
| 5x5 tiles | -0.031 ± 0.068 | -0.001 ± 0.065 | +0.019 ± 0.065 | -0.004 ± 0.075 | -0.001 ± 0.065 | +0.011 ± 0.066 |
| 10x10 tiles | -0.022 ± 0.042 | -0.013 ± 0.032 | -0.039 ± 0.032 | +0.043 ± 0.032 | -0.004 ± 0.032 | -0.023 ± 0.032 |
| 25x25 tiles | -0.003 ± 0.014 | -0.010 ± 0.013 | -0.012 ± 0.015 | -0.005 ± 0.014 | -0.006 ± 0.014 | -0.013 ± 0.013 |
| 50x50 tiles | -0.004 ± 0.008 | -0.002 ± 0.008 | -0.001 ± 0.008 | -0.004 ± 0.009 | -0.014 ± 0.006 | -0.009 ± 0.006 |
| 64 sectors | -0.012 ± 0.040 | -0.056 ± 0.040 | -0.023 ± 0.040 | -0.022 ± 0.044 | -0.045 ± 0.040 | -0.012 ± 0.040 |

**Dispersion**: χ² per region, mean over instances (1 for counting noise) ± the larger of its error under counting noise and the spread over √10:

| partition | 2^14 | 2^16 | 2^18 | 2^20 | 2^24 | 2^28 |
|---|---:|---:|---:|---:|---:|---:|
| 2x2 tiles | 1.12 ± 0.26 | 0.96 ± 0.22 | 0.82 ± 0.22 | 1.16 ± 0.26 | 0.90 ± 0.22 | 0.94 ± 0.22 |
| 5x5 tiles | 0.95 ± 0.09 | 0.98 ± 0.09 | 0.95 ± 0.09 | 0.94 ± 0.09 | 1.02 ± 0.09 | 1.05 ± 0.10 |
| 10x10 tiles | 1.01 ± 0.09 | 1.05 ± 0.09 | 0.92 ± 0.05 | 1.01 ± 0.05 | 1.03 ± 0.04 | 1.01 ± 0.04 |
| 25x25 tiles | 1.05 ± 0.08 | 1.02 ± 0.03 | 0.97 ± 0.02 | 0.99 ± 0.03 | 0.99 ± 0.02 | 0.98 ± 0.02 |
| 50x50 tiles | 1.07 ± 0.12 | 1.03 ± 0.03 | 0.99 ± 0.01 | 0.99 ± 0.01 | 0.99 ± 0.01 | 1.00 ± 0.01 |
| 64 sectors | 0.97 ± 0.08 | 0.95 ± 0.06 | 0.97 ± 0.08 | 0.95 ± 0.06 | 0.95 ± 0.06 | 1.04 ± 0.06 |

## Rational side

**Held-out lift**: the tenth of the regions with the largest excess in one half, read on the other half against that half as a whole, mean of both directions and of the ten instances, ± the larger of the spread and the noise over √10.

| partition | 2^14 | 2^16 | 2^18 | 2^20 | 2^24 | 2^28 |
|---|---:|---:|---:|---:|---:|---:|
| 2x2 tiles | +0.19% ± 0.54% | +0.06% ± 0.32% | +0.17% ± 0.20% | -0.03% ± 0.11% | -0.09% ± 0.06% | -0.04% ± 0.04% |
| 5x5 tiles | +1.20% ± 1.16% | -0.29% ± 0.42% | -0.27% ± 0.41% | +0.01% ± 0.18% | +0.02% ± 0.09% | +0.09% ± 0.07% |
| 10x10 tiles | +0.00% ± 1.01% | -0.16% ± 0.71% | +0.18% ± 0.52% | -0.18% ± 0.24% | -0.13% ± 0.13% | +0.00% ± 0.09% |
| 25x25 tiles | -0.16% ± 0.72% | -0.20% ± 0.45% | +0.31% ± 0.41% | -0.01% ± 0.16% | -0.01% ± 0.15% | +0.02% ± 0.06% |
| 50x50 tiles | -1.99% ± 1.52% | -1.15% ± 0.67% | -0.09% ± 0.22% | -0.14% ± 0.15% | +0.07% ± 0.10% | +0.04% ± 0.07% |
| 64 sectors | -0.92% ± 1.91% | -0.93% ± 0.68% | -0.52% ± 0.24% | -0.46% ± 0.18% | -0.04% ± 0.12% | -0.00% ± 0.08% |

**Split-half correlation** of the regions' excess, mean over instances ± the larger of its error with no correlation and the spread over √10:

| partition | 2^14 | 2^16 | 2^18 | 2^20 | 2^24 | 2^28 |
|---|---:|---:|---:|---:|---:|---:|
| 2x2 tiles | +0.178 ± 0.183 | +0.200 ± 0.203 | +0.020 ± 0.211 | +0.121 ± 0.238 | -0.216 ± 0.197 | -0.136 ± 0.208 |
| 5x5 tiles | -0.023 ± 0.065 | -0.009 ± 0.065 | -0.104 ± 0.065 | -0.003 ± 0.065 | +0.025 ± 0.070 | -0.009 ± 0.065 |
| 10x10 tiles | -0.012 ± 0.032 | +0.007 ± 0.032 | -0.033 ± 0.032 | -0.051 ± 0.032 | -0.016 ± 0.034 | -0.029 ± 0.051 |
| 25x25 tiles | -0.000 ± 0.013 | -0.002 ± 0.013 | +0.001 ± 0.013 | -0.012 ± 0.013 | -0.005 ± 0.013 | -0.005 ± 0.013 |
| 50x50 tiles | -0.009 ± 0.009 | -0.011 ± 0.008 | -0.003 ± 0.006 | -0.009 ± 0.006 | +0.006 ± 0.006 | +0.001 ± 0.006 |
| 64 sectors | -0.011 ± 0.040 | -0.121 ± 0.040 | -0.096 ± 0.040 | -0.080 ± 0.040 | -0.076 ± 0.053 | -0.081 ± 0.041 |

**Dispersion**: χ² per region, mean over instances (1 for counting noise) ± the larger of its error under counting noise and the spread over √10:

| partition | 2^14 | 2^16 | 2^18 | 2^20 | 2^24 | 2^28 |
|---|---:|---:|---:|---:|---:|---:|
| 2x2 tiles | 1.20 ± 0.26 | 1.34 ± 0.22 | 1.30 ± 0.28 | 1.28 ± 0.24 | 0.74 ± 0.22 | 0.61 ± 0.22 |
| 5x5 tiles | 1.02 ± 0.10 | 1.04 ± 0.09 | 1.05 ± 0.09 | 1.04 ± 0.09 | 1.02 ± 0.10 | 0.88 ± 0.09 |
| 10x10 tiles | 0.97 ± 0.04 | 1.02 ± 0.04 | 1.04 ± 0.04 | 0.96 ± 0.04 | 0.99 ± 0.04 | 0.94 ± 0.06 |
| 25x25 tiles | 1.02 ± 0.02 | 1.00 ± 0.02 | 1.01 ± 0.02 | 0.99 ± 0.02 | 0.96 ± 0.02 | 0.98 ± 0.02 |
| 50x50 tiles | 1.01 ± 0.01 | 0.99 ± 0.01 | 1.00 ± 0.01 | 1.00 ± 0.01 | 1.00 ± 0.01 | 1.00 ± 0.01 |
| 64 sectors | 0.98 ± 0.07 | 0.86 ± 0.06 | 0.87 ± 0.06 | 0.84 ± 0.06 | 0.79 ± 0.06 | 0.80 ± 0.06 |

## The criterion

At each protocol bound and partition: the algebraic mean lift over its error, and the algebraic minus the rational lift over their combined error. The claim is supported only where both exceed 3.

| partition | 2^20 algebraic | 2^20 minus control | 2^24 algebraic | 2^24 minus control | 2^28 algebraic | 2^28 minus control |
|---|---:|---:|---:|---:|---:|---:|
| 2x2 tiles | -0.8 | -0.7 | -0.7 | -0.3 | -0.2 | +0.1 |
| 5x5 tiles | -0.7 | -0.7 | +1.5 | +1.3 | +0.9 | +0.6 |
| 10x10 tiles | -0.2 | +0.1 | -0.6 | -0.1 | -1.2 | -1.2 |
| 25x25 tiles | +0.6 | +0.6 | -0.4 | -0.3 | +0.1 | -0.0 |
| 50x50 tiles | +0.6 | +0.7 | +0.1 | -0.2 | -0.7 | -0.9 |
| 64 sectors | -1.9 | -1.2 | +0.0 | +0.2 | -0.2 | -0.2 |

**Rejected**: at no protocol bound and partition does the algebraic lift exceed three standard errors and the control by three combined standard errors.
