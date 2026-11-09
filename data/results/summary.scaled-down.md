# D1 summary across instances (scaled-down)

10 instances: d1-01, d1-02, d1-03, d1-04, d1-05, d1-06, d1-07, d1-08, d1-09, d1-10. Per instance, bound and side the rows are in `data/results/summary.scaled-down.csv`. Every prediction is the α-adjusted Dickman mean with α computed in the harness (`docs/murphy_alpha.md`).

## Rejection criterion, algebraic side

The claim is rejected at a bound when the mean gap (observed − α-adjusted prediction) over the instances is smaller than two standard deviations of the measurement noise. Two noise estimates are shown: the Poisson noise of the counts, and the spread of the gaps across instances.

| B | instances | mean gap | Poisson noise | gap / Poisson noise | spread across instances (std/√n) | gap / spread | mean relative gap | verdict |
|---|---|---|---|---|---|---|---|---|
| 2^14 | 10 | +0.000117 | 0.000004 | +26.5 | 0.000039 | +3.0 | +21.73% | not rejected by Poisson noise; outside 2σ of the instance spread |
| 2^16 | 10 | +0.000197 | 0.000008 | +24.5 | 0.000060 | +3.3 | +12.93% | not rejected by Poisson noise; outside 2σ of the instance spread |
| 2^18 | 10 | +0.000210 | 0.000013 | +16.5 | 0.000079 | +2.7 | +6.72% | not rejected by Poisson noise; outside 2σ of the instance spread |
| 2^20 | 10 | +0.000004 | 0.000018 | +0.2 | 0.000200 | +0.0 | +3.27% | rejected (within 2σ of Poisson noise) |

## Per instance, algebraic side at the largest bound

| instance | digits | values | observed | α-adjusted | gap / se | α | implied α | method | seconds |
|---|---|---|---|---|---|---|---|---|---|
| d1-01 | 40 | 5,472,915 | 0.05888 | 0.06057 | -16.8 | -3.964 | -3.76 |  | 17.6 |
| d1-02 | 44 | 5,472,915 | 0.04413 | 0.04433 | -2.3 | -3.224 | -3.19 | line-sieve | 1.4 |
| d1-03 | 48 | 5,472,915 | 0.02877 | 0.02841 | +5.1 | -5.051 | -5.12 | line-sieve | 1.6 |
| d1-04 | 52 | 5,472,915 | 0.01557 | 0.01559 | -0.5 | -4.109 | -4.08 | line-sieve | 1.4 |
| d1-05 | 56 | 5,472,915 | 0.01347 | 0.01321 | +5.3 | -3.797 | -3.91 | line-sieve | 1.7 |
| d1-06 | 60 | 5,472,915 | 0.01331 | 0.01300 | +6.4 | -3.570 | -3.70 | line-sieve | 1.3 |
| d1-07 | 64 | 5,472,915 | 0.00577 | 0.00523 | +16.6 | -5.328 | -5.88 | line-sieve | 1.4 |
| d1-08 | 68 | 5,472,915 | 0.00354 | 0.00323 | +12.0 | -5.488 | -5.99 | line-sieve | 1.3 |
| d1-09 | 72 | 5,472,915 | 0.00417 | 0.00406 | +4.2 | -5.942 | -6.10 | line-sieve | 1.3 |
| d1-10 | 80 | 5,472,915 | 0.00074 | 0.00069 | +4.6 | -4.710 | -5.11 | line-sieve | 1.4 |

## Rational side at the largest bound

| instance | observed | α-adjusted | gap / se | local density | local gap / se |
|---|---|---|---|---|---|
| d1-01 | 0.34730 | 0.36988 | -111 | 0.33143 | +78 |
| d1-02 | 0.26601 | 0.28642 | -108 | 0.25309 | +68 |
| d1-03 | 0.19006 | 0.20576 | -94 | 0.18053 | +57 |
| d1-04 | 0.12366 | 0.13461 | -78 | 0.11700 | +47 |
| d1-05 | 0.09085 | 0.09929 | -69 | 0.08562 | +43 |
| d1-06 | 0.08479 | 0.09258 | -65 | 0.07969 | +43 |
| d1-07 | 0.04808 | 0.05293 | -53 | 0.04498 | +34 |
| d1-08 | 0.03168 | 0.03525 | -48 | 0.02978 | +25 |
| d1-09 | 0.02721 | 0.03013 | -42 | 0.02540 | +26 |
| d1-10 | 0.01091 | 0.01218 | -29 | 0.01012 | +18 |
