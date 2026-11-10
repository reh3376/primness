# D2 part 1: several number fields sharing the rational side

10 instances, the D1 region; per instance the instrument polynomial F_0 and three rotations f + v·g (`data/results/d2/fields.csv`). Every value's smooth probability is D5's (conditional baseline plus the polynomial's own first-order root-count term), each column scaled by its own κ = O/P. R = O/E − 1 for the values at which both members of a pair are smooth, summed over the pairs of a class. The protocol and the criterion are `docs/d2_falsifiable_prediction.md`, the reading `docs/d2_multiple_fields.md`; per-instance rows are in `data/results/d2_multiple_fields.csv`.

## Joint smoothness against independence

Mean of R over the instances weighted by their noise, ± its error (widened where the instances scatter beyond their noise); χ² about the mean on the instances used.

| class | 2^14 | 2^16 | 2^18 | 2^20 | 2^24 | 2^28 |
|---|---:|---:|---:|---:|---:|---:|
| cross | +0.56% ± 1.07% (χ² 8.4/10) | +0.64% ± 0.44% (χ² 8.6/10) | -0.00% ± 0.21% (χ² 7.5/10) | +0.14% ± 0.12% (χ² 7.2/10) | +0.01% ± 0.07% (χ² 12.6/10) | -0.01% ± 0.03% (χ² 6.0/10) |
| cross control | +1.24% ± 1.32% (χ² 5.2/10) | +0.99% ± 0.57% (χ² 10.8/10) | +0.15% ± 0.25% (χ² 9.1/10) | +0.19% ± 0.16% (χ² 10.8/10) | +0.05% ± 0.06% (χ² 4.4/10) | +0.01% ± 0.04% (χ² 2.6/10) |
| algebraic | -0.79% ± 2.71% (χ² 9.1/10) | +0.07% ± 1.10% (χ² 7.5/10) | +0.07% ± 0.51% (χ² 7.6/10) | -0.25% ± 0.27% (χ² 4.5/10) | +0.15% ± 0.11% (χ² 6.6/10) | -0.02% ± 0.06% (χ² 5.0/10) |
| algebraic control | -1.44% ± 5.20% (χ² 20.8/10) | +0.26% ± 1.68% (χ² 14.1/10) | +0.62% ± 0.60% (χ² 6.1/10) | -0.02% ± 0.31% (χ² 6.5/10) | +0.04% ± 0.12% (χ² 7.7/10) | +0.03% ± 0.06% (χ² 3.9/10) |

**The standard pair** (F_0, G), the two sides of the ordinary NFS: +1.19% ± 1.51% at 2^14, +1.44% ± 0.65% at 2^16, +0.04% ± 0.35% at 2^18, +0.02% ± 0.26% at 2^20, +0.04% ± 0.10% at 2^24, -0.03% ± 0.05% at 2^28.

**Per instance**, R of each class ± its error at the protocol bounds:

| instance | class | 2^20 | 2^24 | 2^28 |
|---|---|---:|---:|---:|
| d1-01 | cross | +0.02% ± 0.18% | +0.01% ± 0.09% | -0.04% ± 0.05% |
| d1-01 | algebraic | +0.11% ± 0.42% | +0.26% ± 0.18% | -0.03% ± 0.10% |
| d1-02 | cross | +0.30% ± 0.24% | -0.03% ± 0.11% | -0.05% ± 0.07% |
| d1-02 | algebraic | -0.31% ± 0.53% | -0.03% ± 0.22% | +0.06% ± 0.12% |
| d1-03 | cross | +0.22% ± 0.34% | +0.09% ± 0.14% | +0.01% ± 0.08% |
| d1-03 | algebraic | -1.09% ± 0.75% | +0.12% ± 0.28% | -0.11% ± 0.14% |
| d1-04 | cross | +0.07% ± 0.55% | -0.13% ± 0.21% | +0.04% ± 0.11% |
| d1-04 | algebraic | -1.18% ± 1.13% | +0.36% ± 0.40% | -0.11% ± 0.19% |
| d1-05 | cross | +0.50% ± 0.70% | +0.41% ± 0.25% | +0.16% ± 0.13% |
| d1-05 | algebraic | -0.41% ± 1.24% | +0.46% ± 0.45% | +0.02% ± 0.21% |
| d1-06 | cross | +0.24% ± 0.74% | +0.31% ± 0.26% | +0.08% ± 0.13% |
| d1-06 | algebraic | +1.17% ± 1.27% | +0.32% ± 0.45% | +0.12% ± 0.21% |
| d1-07 | cross | -0.19% ± 1.45% | -0.19% ± 0.44% | -0.12% ± 0.20% |
| d1-07 | algebraic | -1.08% ± 2.44% | -0.21% ± 0.78% | -0.00% ± 0.34% |
| d1-08 | cross | +1.95% ± 2.18% | -0.19% ± 0.61% | +0.03% ± 0.26% |
| d1-08 | algebraic | -1.55% ± 3.28% | -1.43% ± 1.03% | -0.76% ± 0.43% |
| d1-09 | cross | -4.92% ± 2.22% | -1.53% ± 0.61% | -0.15% ± 0.26% |
| d1-09 | algebraic | -1.37% ± 3.23% | -0.99% ± 0.96% | +0.13% ± 0.40% |
| d1-10 | cross | +2.41% ± 8.66% | -2.02% ± 1.76% | -0.87% ± 0.59% |
| d1-10 | algebraic | +4.28% ± 14.62% | +2.70% ± 3.41% | -0.67% ± 1.15% |

## The fields

κ − 1 (observed over predicted smooth values, over the values not smooth for certain), the D1 check on the rotated fields: weighted mean over the thirty ± its error, and the range.

| columns | 2^14 | 2^16 | 2^18 | 2^20 | 2^24 | 2^28 |
|---|---:|---:|---:|---:|---:|---:|
| instruments F_0 | +0.49% ± 0.50% [-3.0, +4.1] | +0.02% ± 0.24% [-0.5, +3.3] | +0.12% ± 0.14% [-0.4, +1.2] | -0.04% ± 0.10% [-0.3, +1.8] | +0.06% ± 0.06% [-0.2, +0.7] | -0.01% ± 0.04% [-0.2, +0.3] |
| rotated fields | -0.01% ± 0.45% [-37.7, +35.7] | +0.02% ± 0.22% [-6.2, +4.0] | -0.03% ± 0.12% [-5.0, +1.4] | +0.01% ± 0.07% [-2.6, +3.0] | +0.01% ± 0.04% [-1.4, +0.5] | -0.05% ± 0.02% [-0.7, +0.3] |
| rational sides | +0.03% ± 0.15% [-1.6, +5.4] | +0.07% ± 0.09% [-0.8, +2.8] | +0.04% ± 0.08% [-0.5, +1.9] | +0.03% ± 0.05% [-0.2, +0.4] | +0.02% ± 0.03% [-0.1, +0.2] | -0.00% ± 0.02% [-0.1, +0.1] |

**MNFS yield**: relations Σ_i O(F_i, G) from the four fields over O(F_0, G) from the instrument alone, and the pairs (a, b) with at least one relation over the same, summed over the instances:

| | 2^14 | 2^16 | 2^18 | 2^20 | 2^24 | 2^28 |
|---|---:|---:|---:|---:|---:|---:|
| relations, four fields over one | 2.11 | 2.30 | 2.52 | 2.71 | 3.00 | 3.21 |
| pairs with a relation, four fields over one | 2.08 | 2.25 | 2.42 | 2.54 | 2.59 | 2.50 |

## The criterion

At each protocol bound: the related class's weighted mean R over its error, and the related minus the control over their combined error. The claim is supported only where both exceed 3.

| class | 2^20 related | 2^20 minus control | 2^24 related | 2^24 minus control | 2^28 related | 2^28 minus control |
|---|---:|---:|---:|---:|---:|---:|
| cross | +1.1 | -0.3 | +0.2 | -0.4 | -0.4 | -0.5 |
| algebraic | -0.9 | -0.6 | +1.4 | +0.7 | -0.4 | -0.7 |

**Rejected**: at no protocol bound does a related class exceed zero by three errors and its control by three combined errors.
