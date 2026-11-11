# Obstacle D Closed; H1 Measures the 200-Digit Sieve on This Laptop (2026-11-10 19:30 EST)

**Metadata**
- **Created**: 2026-11-10 19:30 EST
- **Last Modified**: 2026-11-10 19:30 EST
- **Version**: 1.0
- **Tags**: obstacle-D, verdict, closing-checks, galois, obstacle-H, H1, cado-nfs, rsa-200
- **Related Files**: docs/obstacle_d_verdict.md, docs/d1_falsifiable_prediction.md, src/bin/d_checks.rs, scripts/d_checks.py, scripts/d_checks_run_all.sh, data/results/d_checks.md, data/instruments/galois/, data/results/galois/, docs/h1_falsifiable_prediction.md, docs/h1_sieving_cost.md, scripts/h1_test_sieve.sh, scripts/h1_cost.py, data/results/h1_cost.md, data/results/h1/

## Why

D1, D5 and D2 part 1 were all rejected, and the author closed the D measurements. Before closing, the data were searched once more for anything the three criteria could miss. That covered a weak signal spread across many tests, assumptions the criteria never tested, growth toward the goal's sizes, and algebraic structure at its most extreme. With D closed, the distance between this laptop and the goal is a question of throughput, which is Obstacle H. Its first step replaces the gap scaled from the records with a measured one.

## What was built

- `d-checks` (Rust). One pass per dump column sums O, P = Σπ and V = Σπ(1 − π) over the values that are not smooth for certain. The sums are grouped by u, by the small-prime part, by the other side's small part and cofactor, and by (a, b) mod 2 and mod 3. The pass also counts neighbouring pairs with their exact variance, including the covariance of chained pairs. The variance is tested against enumeration.
- `scripts/d_checks.py` (standard library). It builds the manifests from the D5 prediction, pools the 2,160 statistics of D1, D2 part 1 and D5 (mean, mean square, Kolmogorov–Smirnov), and runs the per-value tests, the trend in digits and the quartic comparison. `scripts/d_checks_run_all.sh` runs everything end to end in about four minutes.
- The five irreducible toy quartics of the structure ladder were measured again with the final harness. They are in `data/instruments/galois/` and `data/results/galois/`, with root counts to 2^28.
- `docs/obstacle_d_verdict.md`: what was measured, the closing checks, what is bounded without a measurement, the verdict, and what would reopen D.
- `docs/h1_falsifiable_prediction.md`, committed alone in c6dd75a before the run. `scripts/h1_test_sieve.sh` builds the factor base and runs `las -random-sample 40 -t 18 -T` on six special-q ranges under `hw-sample`. `scripts/h1_cost.py` integrates the result to the relation target.

## What was measured

Closing checks:
- The D1 rational side shares the algebraic side's small positive mean z (+0.31 against +0.27), so that shift comes from the method.
- The rotated fields sit slightly low (mean z −0.40 at 2^28).
- D5's lifts are negative, which means the values are slightly anti-clustered.
- The largest |z| values, up to 38, are χ² values on tiles that hold one smooth value against an expected 0.001.
- None of the per-value groupings matters; all p ≥ 0.01 over 90 χ² tests.
- Neighbours at 2^28: −0.13% ± 0.09% and +0.00% ± 0.08%.
- Calibration in u is flat to ±0.06% for u from 1 to 3, +0.19% ± 0.57% at u 4 to 4.5, and −2.9% ± 1.9% at 4.5 to 5.
- Trend in digits at 2^28: −0.0000% ± 0.0021% per digit, 95% upper bound +0.47% at 200 digits and +0.64% at 250.
- Quartics after the root-count term: +0.07% to +1.02% at 2^14 and +0.02% to +0.16% at 2^28. No slope in Var r_p beyond 1.6 errors.

H1 sampled six ranges from q = 5e7 to 1.6e9, 40 special-q's each:
- Relations per special-q fall from 25.9 to 8.2, and wall time per special-q from 0.53 s to 0.40 s at 1,590 to 1,620 percent CPU.
- The integral reaches 461,027,928 relations at Q_max = 6.32e8 after 29.9 million special-q's, which is 3,896 laptop-hours. H1 holds.
- Sieving takes 50 to 64 percent of the time and cofactorization 25 to 37 percent.
- Power is 62 to 66 W, the CPU peaks at 83 °C, and peak memory is 6.4 GB.

## Questions answered

- Does the Galois group matter beyond root counts? No (roadmap open question 3).
- Is the record-based gap right? Within a factor of 2. Measured, the laptop does the work of about 30 Xeon Gold 6130 cores, not 18. The gap at 200 digits is about 3,900 times for the relations.
- Is anything left in the D data? No hidden excess, no untested assumption that fails, and no growth toward 200 digits within the bounds stated.

## Questions raised

- What does the GPU add to the bucket sieve and to ECM cofactorization? That is H2.
- What is the five quartics' common +0.10%? It is small, and the candidates are a second-order term or roots of small height.
- The linear algebra at 200 digits and the laptop's own tuned one-hour reach. The 100-digit CADO-NFS run spent a third of its wall time in serial stages. Its relation target was low (3.6 million of the 4.29 million needed), so 11 filtering passes overlapped the last 71 s of sieving.

## Hardware

- The closing checks took about 4 minutes: five quartic measurements of 33 to 54 s each, and `d-checks` over 70 columns in seconds.
- The H1 sampling took 6 × 17 to 21 s, plus makefb.
- The CADO-NFS calibration runs from c60 to c100 are in the session scratchpad, not in the repository.
