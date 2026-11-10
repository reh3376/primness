# The Conditional Baseline Q_B(c) Tabulated (2026-11-10 04:10 EST)

**Metadata**
- **Created**: 2026-11-10 04:10 EST
- **Last Modified**: 2026-11-10 04:10 EST
- **Version**: 1.0
- **Tags**: obstacle-D, D1, baseline, conditional-density, cofactor, small-prime-part
- **Related Files**: docs/baseline_calibration.md, src/bin/d1_baseline.rs, schemas/d1-baseline/2.json, scripts/d1_calibrate.py, data/results/baseline.consecutive.json, data/results/baseline.consecutive.cofactor.csv

## Why

After the measured baseline and α₂ the algebraic side keeps a gap of +1.2% at 2^24 and +0.5% at 2^28. The measurement that decides what carries it compares each value's smoothness with the density of smooth integers among rough integers of its cofactor's size. That density had to be tabulated first.

## What was built

- `d1-baseline` splits every integer n of every interval as n = s·c, s the part made of primes up to the rough bound (default 2000, `alpha::DEFAULT_BOUND`), and tallies per bin of ln c (default 0.25) the integers and, per bound, the B-smooth ones. Pooled over the sizes. Options `--rough-bound`, `--cofactor-bin`.
- `small_part_logs`: a sieve over the 303 small primes and their powers up to 2^62 for every position, with exact division where the last power still divides.
- Result contract `d1-baseline/2` (version 1 kept): adds the `cofactor` object and the table named `cofactor`. `scripts/d1_calibrate.py` reads either version.
- Tests: per-bin counts of three chunks against division by every prime; small-part logs against division on ranges holding 2^70 and 3^45.

## What was measured

1,694,498,816 integers, 450 filled bins, 95 s. Bin 0 (c = 1) holds 6,118,855. Bins from ln c = 20 to 105 hold 4.3 to 4.9 million each.

- Smooth counts summed over bins equal the unconditional counts at every bound, exactly. The points table is byte-identical to the previous run and `summary.calibrated` changes only in the run id and duration.
- Q over the unconditional de Bruijn form at the same u: 0.64 (ln c = 25) to 0.31 (70) at 2^28; 0.66 to 0.20 at 2^24; 0.32 to 0.10 at 2^20.
- Banded at small bounds: at 2^14 the bin [30, 30.25) has no smooth integer, because three primes in (2000, 2^14] reach e^29.1 and four start at e^30.4.
- Poisson error per bin, relative, at ln c = 25, 50, 70: 2^28: 0.07%, 0.22%, 0.75%. 2^24: 0.08%, 0.40%, 2.0%. 2^20: 0.14%, 1.2%, 8.7%.

## Questions raised

- Is the table precise enough at the upper cofactor sizes for a 0.5% to 1.2% gap? Not computed until the instrument sum exists; `--length 2^26` is the lever.
- The instrument side must use the same definition of s (all primes up to 2000 with full multiplicity, including primes dividing the leading coefficient on rows where they divide b).

## Hardware

Baseline run `d1b-14723-1794301633483`: 93 samples over 94 s; CPU activity mean 0.51, memory used 0.83 of 128 GB (other processes included), CPU temperature peak 90.5 °C.
