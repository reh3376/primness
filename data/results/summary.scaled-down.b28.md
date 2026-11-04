# D1 summary across instances (scaled-down.b28)

10 instances: d1-01, d1-02, d1-03, d1-04, d1-05, d1-06, d1-07, d1-08, d1-09, d1-10. Per instance, bound and side the rows are in `data/results/summary.scaled-down.b28.csv`. Every prediction is the α-adjusted Dickman mean with α computed in the harness (`docs/murphy_alpha.md`).

## Rejection criterion, algebraic side

The claim is rejected at a bound when the mean gap (observed − α-adjusted prediction) over the instances is smaller than two standard deviations of the measurement noise. Two noise estimates are shown: the Poisson noise of the counts, and the spread of the gaps across instances.

| B | instances | mean gap | Poisson noise | gap / Poisson noise | spread across instances (std/√n) | gap / spread | mean relative gap | verdict |
|---|---|---|---|---|---|---|---|---|
| 2^20 | 10 | +0.000004 | 0.000018 | +0.2 | 0.000200 | +0.0 | +3.27% | rejected (within 2σ of Poisson noise) |
| 2^24 | 10 | -0.000972 | 0.000030 | -32.2 | 0.000575 | -1.7 | -0.31% | not rejected by Poisson noise; within 2σ of the instance spread |
| 2^28 | 10 | -0.002709 | 0.000042 | -64.9 | 0.000926 | -2.9 | -1.70% | not rejected by Poisson noise; outside 2σ of the instance spread |

## Per instance, algebraic side at the largest bound

| instance | digits | values | observed | α-adjusted | gap / se | α | implied α | method | seconds |
|---|---|---|---|---|---|---|---|---|---|
| d1-01 | 40 | 5,472,915 | 0.26010 | 0.26916 | -48.3 | -3.964 | -3.55 | line-sieve | 39.7 |
| d1-02 | 44 | 5,472,915 | 0.21721 | 0.22297 | -32.7 | -3.224 | -2.90 | line-sieve | 39.9 |
| d1-03 | 48 | 5,472,915 | 0.16612 | 0.16967 | -22.3 | -5.051 | -4.83 | line-sieve | 45.1 |
| d1-04 | 52 | 5,472,915 | 0.11403 | 0.11731 | -24.1 | -4.109 | -3.77 | line-sieve | 39.7 |
| d1-05 | 56 | 5,472,915 | 0.10370 | 0.10628 | -19.8 | -3.797 | -3.55 | line-sieve | 37.7 |
| d1-06 | 60 | 5,472,915 | 0.10515 | 0.10723 | -15.8 | -3.570 | -3.35 | line-sieve | 37.3 |
| d1-07 | 64 | 5,472,915 | 0.06041 | 0.06026 | +1.4 | -5.328 | -5.34 | line-sieve | 36.7 |
| d1-08 | 68 | 5,472,915 | 0.04486 | 0.04470 | +1.8 | -5.488 | -5.51 | line-sieve | 39.2 |
| d1-09 | 72 | 5,472,915 | 0.05087 | 0.05190 | -11.0 | -5.942 | -5.75 | line-sieve | 43.6 |
| d1-10 | 80 | 5,472,915 | 0.01722 | 0.01730 | -1.4 | -4.710 | -4.66 | line-sieve | 37.5 |

## Rational side at the largest bound

| instance | observed | α-adjusted | gap / se | local density | local gap / se |
|---|---|---|---|---|---|
| d1-01 | 0.68372 | 0.70633 | -114 | 0.66790 | +80 |
| d1-02 | 0.60056 | 0.62157 | -100 | 0.58626 | +68 |
| d1-03 | 0.50952 | 0.52854 | -89 | 0.49636 | +62 |
| d1-04 | 0.40892 | 0.42606 | -82 | 0.39702 | +57 |
| d1-05 | 0.34598 | 0.36228 | -80 | 0.33504 | +54 |
| d1-06 | 0.33265 | 0.34858 | -79 | 0.32171 | +54 |
| d1-07 | 0.24061 | 0.25375 | -72 | 0.23211 | +47 |
| d1-08 | 0.19079 | 0.20153 | -64 | 0.18381 | +42 |
| d1-09 | 0.17442 | 0.18439 | -61 | 0.16797 | +40 |
| d1-10 | 0.10361 | 0.11020 | -51 | 0.09955 | +31 |
