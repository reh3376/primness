# D1 summary against the calibrated baseline

Baseline run `d1b-14723-1794301633483`: 101 sizes, ln X from 20 to 120 in steps of 1, 16,777,216 consecutive integers each, 95 s. Definitions in `docs/baseline_calibration.md`. Per instance, bound and side the rows are in `data/results/summary.calibrated.csv`; the baseline check is in `data/results/summary.calibrated.baseline.csv`.

## Which Dickman form do integers without structure follow?

Over the baseline points with at least 1,000 smooth values and u in [1.5, 6.0]: mean of observed/form − 1, the largest |observed/form − 1|, and the mean |observed − form| in Poisson standard errors.

| B | points | u range | ρ(u): mean dev | max | mean σ | ρ + ρ′/ln B: mean dev | max | mean σ | ρ + γρ′/ln B: mean dev | max | mean σ |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 2^14 | 34 | 2.06–5.46 | -12.5% | 16.6% | 77.5 | +15.5% | 24.5% | 67.9 | +1.7% | 6.0% | 6.4 |
| 2^16 | 42 | 1.80–5.50 | -11.4% | 15.8% | 80.9 | +11.7% | 18.4% | 63.9 | +0.6% | 3.4% | 2.8 |
| 2^18 | 50 | 1.60–5.53 | -10.1% | 15.2% | 79.3 | +9.7% | 15.1% | 61.2 | +0.3% | 2.6% | 2.0 |
| 2^20 | 57 | 1.51–5.55 | -9.1% | 15.5% | 74.3 | +8.4% | 14.7% | 56.6 | +0.2% | 4.5% | 1.5 |
| 2^24 | 68 | 1.50–5.53 | -7.6% | 14.8% | 62.6 | +6.7% | 11.4% | 47.1 | +0.1% | 5.4% | 1.0 |
| 2^28 | 79 | 1.55–5.56 | -6.5% | 11.3% | 51.9 | +5.6% | 10.5% | 38.9 | +0.1% | 3.0% | 0.8 |

## Rejection criterion, algebraic side, against each baseline

Each prediction is the α-adjusted Dickman sum of the run, per u bin, times the baseline's ratio to ρ at the α-shifted u. `α only` is the prediction the earlier summaries used. The gap is observed − predicted. The noise is the Poisson noise of the counts (√Σse² / n) and, beside it, the spread of the gaps across instances (std / √n). Absolute gaps weight the dense small instances most; relative gaps weight every instance alike.

### Baseline: α only

| B | n | mean abs gap | Poisson noise | gap / noise | spread | gap / spread | verdict (absolute) | mean rel gap | rel Poisson noise | rel spread | verdict (relative) |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 2^14 | 10 | +0.000117 | 0.000004 | +26.5 | 0.000039 | +3.0 | not rejected | +21.73% | 2.24% | 3.51% | not rejected |
| 2^16 | 10 | +0.000197 | 0.000008 | +24.5 | 0.000060 | +3.3 | not rejected | +12.93% | 0.87% | 2.77% | not rejected |
| 2^18 | 10 | +0.000210 | 0.000013 | +16.5 | 0.000079 | +2.7 | not rejected | +6.72% | 0.40% | 1.77% | not rejected |
| 2^20 | 10 | +0.000004 | 0.000018 | +0.2 | 0.000200 | +0.0 | rejected | +3.27% | 0.22% | 1.40% | not rejected |
| 2^24 | 10 | -0.000972 | 0.000030 | -32.2 | 0.000575 | -1.7 | not rejected | -0.31% | 0.09% | 0.76% | not rejected |
| 2^28 | 10 | -0.002709 | 0.000042 | -64.9 | 0.000926 | -2.9 | not rejected | -1.70% | 0.05% | 0.41% | not rejected |

### Baseline: de Bruijn local form

| B | n | mean abs gap | Poisson noise | gap / noise | spread | gap / spread | verdict (absolute) | mean rel gap | rel Poisson noise | rel spread | verdict (relative) |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 2^14 | 10 | +0.000258 | 0.000004 | +58.4 | 0.000091 | +2.8 | not rejected | +43.95% | 2.68% | 4.30% | not rejected |
| 2^16 | 10 | +0.000611 | 0.000008 | +76.0 | 0.000195 | +3.1 | not rejected | +29.77% | 1.01% | 3.41% | not rejected |
| 2^18 | 10 | +0.001126 | 0.000013 | +88.4 | 0.000318 | +3.5 | not rejected | +19.97% | 0.46% | 2.23% | not rejected |
| 2^20 | 10 | +0.001672 | 0.000018 | +91.8 | 0.000389 | +4.3 | not rejected | +14.11% | 0.25% | 1.80% | not rejected |
| 2^24 | 10 | +0.002769 | 0.000030 | +91.6 | 0.000443 | +6.2 | not rejected | +7.43% | 0.10% | 1.03% | not rejected |
| 2^28 | 10 | +0.003434 | 0.000042 | +82.3 | 0.000400 | +8.6 | not rejected | +4.16% | 0.06% | 0.61% | not rejected |

### Baseline: empirical curve

| B | n | mean abs gap | Poisson noise | gap / noise | spread | gap / spread | verdict (absolute) | mean rel gap | rel Poisson noise | rel spread | verdict (relative) |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 2^14 | 10 | +0.000242 | 0.000004 | +54.9 | 0.000084 | +2.9 | not rejected | +41.15% | 2.63% | 4.18% | not rejected |
| 2^16 | 10 | +0.000592 | 0.000008 | +73.6 | 0.000188 | +3.2 | not rejected | +28.86% | 1.00% | 3.36% | not rejected |
| 2^18 | 10 | +0.001093 | 0.000013 | +85.7 | 0.000306 | +3.6 | not rejected | +19.48% | 0.46% | 2.24% | not rejected |
| 2^20 | 10 | +0.001633 | 0.000018 | +89.7 | 0.000377 | +4.3 | not rejected | +13.86% | 0.25% | 1.80% | not rejected |
| 2^24 | 10 | +0.002718 | 0.000030 | +89.9 | 0.000434 | +6.3 | not rejected | +7.31% | 0.10% | 1.02% | not rejected |
| 2^28 | 10 | +0.003366 | 0.000042 | +80.6 | 0.000393 | +8.6 | not rejected | +4.08% | 0.06% | 0.60% | not rejected |

### Baseline: de Bruijn local form with the second-order term α₂

| B | n | mean abs gap | Poisson noise | gap / noise | spread | gap / spread | verdict (absolute) | mean rel gap | rel Poisson noise | rel spread | verdict (relative) |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 2^14 | 10 | +0.000076 | 0.000004 | +17.3 | 0.000029 | +2.6 | not rejected | +11.07% | 1.98% | 2.34% | not rejected |
| 2^16 | 10 | +0.000154 | 0.000008 | +19.2 | 0.000050 | +3.1 | not rejected | +7.34% | 0.81% | 1.55% | not rejected |
| 2^18 | 10 | +0.000250 | 0.000013 | +19.6 | 0.000072 | +3.5 | not rejected | +4.31% | 0.38% | 0.99% | not rejected |
| 2^20 | 10 | +0.000279 | 0.000018 | +15.3 | 0.000077 | +3.6 | not rejected | +2.80% | 0.22% | 0.82% | not rejected |
| 2^24 | 10 | +0.000356 | 0.000030 | +11.8 | 0.000127 | +2.8 | not rejected | +1.19% | 0.09% | 0.47% | not rejected |
| 2^28 | 10 | +0.000350 | 0.000042 | +8.4 | 0.000175 | +2.0 | not rejected | +0.53% | 0.05% | 0.26% | not rejected |

## Per instance, algebraic side: relative gap against the de Bruijn baseline

| instance | digits | 2^14 | 2^16 | 2^18 | 2^20 | 2^24 | 2^28 | α |
|---|---|---|---|---|---|---|---|---|
| d1-01 | 40 | +20.8% | +14.3% | +9.9% | +6.1% | +2.8% | +1.5% | -3.96 |
| d1-02 | 44 | +35.1% | +20.7% | +13.6% | +9.0% | +4.3% | +2.5% | -3.22 |
| d1-03 | 48 | +43.8% | +27.2% | +16.7% | +11.2% | +5.9% | +3.2% | -5.05 |
| d1-04 | 52 | +39.9% | +22.2% | +14.4% | +10.0% | +5.4% | +2.8% | -4.11 |
| d1-05 | 56 | +39.6% | +25.2% | +18.1% | +12.5% | +6.3% | +3.3% | -3.80 |
| d1-06 | 60 | +47.1% | +27.5% | +19.2% | +13.1% | +7.1% | +3.8% | -3.57 |
| d1-07 | 64 | +67.3% | +49.8% | +31.3% | +22.4% | +12.0% | +6.6% | -5.33 |
| d1-08 | 68 | +63.4% | +43.2% | +30.2% | +21.7% | +12.1% | +6.9% | -5.49 |
| d1-09 | 72 | +35.8% | +30.6% | +20.8% | +14.3% | +7.3% | +4.3% | -5.94 |
| d1-10 | 80 | +46.7% | +37.1% | +25.4% | +20.9% | +11.1% | +6.7% | -4.71 |

## Per instance, algebraic side: relative gap after the second-order term α₂

The prediction is the de Bruijn baseline times 1 + α₂ ρ″(u)/(2 ρ(u) ln² B) per u bin, with α₂ computed from the polynomial (`data/results/alpha_moments.csv`). At 2^14 the shift α/ln B is about 0.4 in u and the second-order expansion is coarse; from 2^20 up it is the regime the expansion is built for.

| instance | digits | 2^14 | 2^16 | 2^18 | 2^20 | 2^24 | 2^28 | α | α₂ |
|---|---|---|---|---|---|---|---|---|---|
| d1-01 | 40 | +1.7% | +1.7% | +1.4% | +0.2% | -0.2% | -0.2% | -3.96 | +6.28 |
| d1-02 | 44 | +11.2% | +5.5% | +3.4% | +1.9% | +0.8% | +0.5% | -3.22 | +6.93 |
| d1-03 | 48 | +13.0% | +7.4% | +3.4% | +1.8% | +0.9% | +0.5% | -5.05 | +8.50 |
| d1-04 | 52 | +11.8% | +3.9% | +1.6% | +0.8% | +0.3% | -0.0% | -4.11 | +7.72 |
| d1-05 | 56 | +9.3% | +4.8% | +3.7% | +2.1% | +0.6% | +0.1% | -3.80 | +8.34 |
| d1-06 | 60 | +15.1% | +6.8% | +4.8% | +2.8% | +1.5% | +0.7% | -3.57 | +8.02 |
| d1-07 | 64 | +17.5% | +15.0% | +7.5% | +5.1% | +2.4% | +0.9% | -5.33 | +11.97 |
| d1-08 | 68 | +24.4% | +16.4% | +11.3% | +7.9% | +4.3% | +2.3% | -5.49 | +8.86 |
| d1-09 | 72 | -1.3% | +3.0% | +1.0% | -0.2% | -0.9% | -0.7% | -5.94 | +10.21 |
| d1-10 | 80 | +8.1% | +8.8% | +5.0% | +5.4% | +2.3% | +1.2% | -4.71 | +8.72 |

## Implied extra variance against computed α₂, algebraic side

The implied value is the extra variance of the size shift (in ln² units) that would close each instance's gap against the de Bruijn baseline at that bound, (observed − predicted) / Σ predicted·ρ″/(2ρ ln² B). A residual that is a pure second-order effect gives the same number at every bound, and that number is α₂.

| instance | 2^14 | 2^16 | 2^18 | 2^20 | 2^24 | 2^28 | α₂ computed |
|---|---|---|---|---|---|---|---|
| d1-01 | 6.9 | 7.2 | 7.4 | 6.5 | 5.8 | 5.5 | +6.28 |
| d1-02 | 11.3 | 10.0 | 9.6 | 8.9 | 8.5 | 8.6 | +6.93 |
| d1-03 | 13.7 | 12.5 | 11.0 | 10.4 | 10.2 | 10.1 | +8.50 |
| d1-04 | 12.3 | 9.7 | 8.8 | 8.5 | 8.2 | 7.6 | +7.72 |
| d1-05 | 11.9 | 10.8 | 10.9 | 10.3 | 9.2 | 8.5 | +8.34 |
| d1-06 | 13.6 | 11.4 | 11.2 | 10.5 | 10.3 | 9.9 | +8.02 |
| d1-07 | 19.0 | 19.7 | 16.9 | 16.3 | 15.3 | 13.9 | +11.97 |
| d1-08 | 17.9 | 16.7 | 15.7 | 15.0 | 14.4 | 13.6 | +8.86 |
| d1-09 | 9.7 | 11.7 | 10.9 | 10.1 | 9.0 | 8.8 | +10.21 |
| d1-10 | 11.4 | 12.5 | 11.4 | 12.4 | 11.2 | 10.8 | +8.72 |

## Per instance, rational side: relative gap against the de Bruijn baseline

| instance | 2^14 | 2^16 | 2^18 | 2^20 | 2^24 | 2^28 | α |
|---|---|---|---|---|---|---|---|
| d1-01 | -0.7% | -0.7% | -0.5% | -0.1% | -0.1% | -0.1% | +0.57 |
| d1-02 | -0.8% | -0.7% | -0.6% | -0.5% | -0.1% | -0.1% | +0.57 |
| d1-03 | -1.4% | -1.0% | -0.7% | -0.6% | -0.2% | -0.1% | +0.57 |
| d1-04 | -1.2% | -1.3% | -0.9% | -0.6% | -0.3% | -0.1% | +0.57 |
| d1-05 | -1.4% | -1.4% | -1.0% | -0.6% | -0.4% | -0.2% | +0.57 |
| d1-06 | -2.2% | -1.1% | -0.6% | -0.4% | -0.3% | -0.1% | +0.57 |
| d1-07 | -1.0% | -0.8% | -0.7% | -0.5% | -0.3% | -0.3% | +0.57 |
| d1-08 | -2.0% | -2.3% | -1.8% | -1.3% | -0.4% | -0.3% | +0.57 |
| d1-09 | -0.6% | -1.9% | -1.2% | -0.7% | -0.4% | -0.3% | +0.57 |
| d1-10 | +2.0% | +0.5% | +0.2% | -0.8% | -0.7% | -0.4% | +0.57 |
