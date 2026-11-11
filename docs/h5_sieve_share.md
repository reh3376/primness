# H5: what a perfect sieve would buy at the one-hour reach

`docs/h5_falsifiable_prediction.md` states the claim, the protocol and the criterion. It was committed (2f1ab98) before any categorized timing at 130 digits was measured. This document records the run and the result.

## 1. Method

**Instance and sieve.** RSA-130 exactly as H4 ran it:
- the polynomial CADO-NFS selected in that run, and its side-1 factor base (hashes in `data/results/h5/inputs.sha256`);
- the parameters of the run's client command: -I 13, lim0 = 6e6, lim1 = 7.5e6, lpb 28 and 29, mfb 54 and 57, lambda 1.93 and 1.96, 11 and 13 curves, special-q on side 1.

**Sampling.** `scripts/h5_sieve_share.sh` runs nine `las` processes at once, one per ninth of [7.5e5, 4.33e6). Each uses `-t 2 -random-sample 300 -T -seed 1`, as the run's nine 2-thread clients did. Together they sample 2,700 special-q's in 51 seconds under `hw-sample`. The CPU was 99 percent active at 58 W, with the tiers at 4.1 and 3.7 GHz. Those are near the burst clocks of H1's samples, and only shares enter the statistic.

**Weights.** Each part is weighted by the special-q's the H4 run sieved in it, counted from the run's 358 workunit outputs: 244,823 in all. The run's own log reports 243,487 in its sieving statistics. The 0.5 percent difference moves no share. `scripts/h5_summary.py` writes `data/results/h5_sieve_share.{md,csv}`.

## 2. Result

| group (`las -T`) | share of the run's counted time |
|---|---:|
| norms | 1.2% |
| sieving | 77.5% |
| survivor search | 6.2% |
| cofactorization | 10.8% |
| bookkeeping | 4.3% |
| **sieve proper** (norms, sieving, survivor search) | **84.9%** |

The split hardly moves across the range: the sieve proper is 84.0 percent in the lowest ninth and 85.7 percent in the highest.

The sieving phase is 76.7 percent of RSA-130's wall time (H4). That gives **s = 0.849 × 0.767 = 0.651**. A sieve that cost nothing and found the same relations would make the factorization at most **2.87 times** faster.

**H5 holds**: s is below 2/3, by a narrow margin. The derivation was off, though.
- It carried H1's 200-digit split down to 130 digits and predicted s = 0.42 to 0.54, a ceiling of 1.7 to 2.2 times.
- At 130 digits the cofactorization is 11 percent of `las`'s time, against 25 to 37 percent at 200 digits. The sieve proper is therefore 85 percent, not 55 to 70 percent.

## 3. Reading

- **What a better sieve can buy.** One hour at 200 digits needs 4,300 to 4,700 times the laptop.

  | | perfect sieve (same relations, no cost) | perfect relation collection (sieve and cofactorization) |
  |---|---:|---:|
  | RSA-130, the one-hour reach | 2.87× | 4.3× (H4's phases) |
  | 200 digits | 1.8 to 2.7× | 5.9 to 11× |

  The 200-digit figures use H1's split, with the records' share for everything outside relation collection until the linear algebra is measured.
- **Where the cost grows.** These figures come from H1's and H4's data and were worked out before the prediction was written.

  | from RSA-130 to 200 digits | ratio |
  |---|---:|
  | relations needed | 14.0× |
  | positions per relation | 144× |
  | sieve positions in all | 2,015× |
  | `las` core-nanoseconds per position (9.8 to 14.1) | 1.45× |
  | `las` core-seconds in all | 2,913× |

  The cost of handling one candidate barely changes. What grows is the number of candidates, because smooth values become rarer.
- **What "far more efficient" would have to change.** The sieve already spends about 10 to 14 core-nanoseconds, some 40 to 60 cycles, on each position. Anything that examines the same candidates can gain only constant factors, and H2's 8 times on the GPU is one of those. A replacement that is far more efficient would have to examine far fewer candidates per relation, or need far fewer relations. How rarely a candidate yields a relation is set by the size of the values, and Obstacle D measured that they are exactly as smooth as their sizes and root counts make them. That is a change of algorithm, not of sieve.
- **The share shifts with size.** Cofactorization grows from 11 percent of `las`'s time at 130 digits to 25 to 37 percent at 200. At the goal's size, ECM is a larger target, and that is where the GPU's 19.6 times applies (H2).

## Files

| File | Role |
|---|---|
| `docs/h5_falsifiable_prediction.md` | The claim, the protocol and the criterion, committed before the run. |
| `scripts/h5_sieve_share.sh` | The special-q counts of the H4 run, the nine `las -T` samples under `hw-sample`, then the summary. |
| `scripts/h5_summary.py` | The weighted shares, s, the ceilings, the criterion, H1's split at 200 digits and the growth decomposition; writes `data/results/h5_sieve_share.{md,csv}`. |
| `data/results/h5/` | `las`'s statistics per part, the run's special-q count per workunit, the input hashes, the CADO-NFS commit and the `hw-sample` summary. |
