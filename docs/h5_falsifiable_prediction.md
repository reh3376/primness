# Falsifiable Prediction under Obstacle H / H5: what a perfect sieve would buy at the one-hour reach

Written on 2026-11-11, before any categorized timing at 130 digits was measured.

**Scope**
H4 found the sieving phase is 60 to 77 percent of a complete factorization's wall time on this laptop, and its share rises with size (`docs/h4_one_hour_reach.md`). That phase does two jobs:
- the lattice sieve itself: norm initialization, the bucket sieve, and the search for survivors;
- the cofactorization of what survives, by ECM on both sides.

A faster sieve, or a replacement that finds the same relations without sieving, can save at most the sieve's own time. H5 measures that share at the laptop's one-hour reach and turns it into the ceiling on any speedup confined to the sieve.

**Claim (H5)**
At the laptop's one-hour reach, a sieve that cost nothing and found the same relations would make a complete factorization less than 3 times faster. It is measured on RSA-130 as H4 ran it. The sieve proper is what `las -T` counts as norms, sieving and survivor search, and the claim is that it is less than two thirds of the wall time.

**Derivation**
At 200 digits H1 measured, per `las -T`:
- sieving: 50 to 64 percent of the counted time;
- survivor search: 4 to 5 percent;
- norms: under 1 percent.

The sieve proper is therefore 55 to 70 percent. H4's sieving phase is 76.7 percent of RSA-130's wall time. If the split at 130 digits is H1's, the sieve proper is 42 to 54 percent of the wall time, and a free sieve makes the factorization 1.7 to 2.2 times faster.

**Measurement protocol**
1. Instance and sieve: RSA-130 as H4 ran it.
   - The polynomial CADO-NFS selected in that run and its side-1 factor base (`data/dumps/h4/rsa130/c130.poly` and `c130.roots1.gz`, with their hashes kept).
   - The parameters of the run's client command, exactly: -I 13, lim0 = 6e6, lim1 = 7.5e6, lpb 28 and 29, mfb 54 and 57, lambda 1.93 and 1.96, 11 and 13 curves, special-q on side 1.
2. Sampling: the special-q range the run sieved, [7.5e5, 4.33e6), is cut into 9 equal parts. Nine `las` processes run at once, one per part.
   - Each runs with -t 2, as the run's nine clients did.
   - Each runs `-random-sample 300 -T -seed 1`, under `hw-sample`.
3. Weighting:
   - Each part's categorized times per special-q are multiplied by the number of special-q's the run sieved in that part, counted from its workunit outputs, and the products are summed.
   - The sieve proper is norms (categories 4 and 260), sieving (2, 3 and 258) and survivor search (1).
   - Cofactorization is 5, 6 and 261. Bookkeeping is 0 and −1.
4. The statistic: s = (the sieve proper's share of the counted time) × (the sieving phase's share of the wall time in H4, 0.767). The ceiling is 1 / (1 − s).

**Rejection criterion**
H5 holds if s < 2/3, which is a ceiling below 3 times. Otherwise H5 is rejected. A faster sieve could then buy at least 3 times at the reach, and the roadmap's reading of the 77 percent changes with it.

**Reported beside the verdict, not part of it**
- The cofactorization's share, and the ceiling for a relation collection that cost nothing at all. At RSA-130 that is 4.3 times from H4's phases. At 200 digits it is 5.8 to 10.6 times, from the records' share of linear algebra and polynomial selection, until the linear algebra is measured.
- The same two ceilings at 200 digits, from H1's split.
- The survivor funnel per special-q at 130 digits, beside H1's at 200.
- How the growth of the relation-collection cost from RSA-130 to 200 digits divides between the number of sieve positions and the cost of each. These figures come from H1's and H4's data. They were worked out before this prediction was written, so they are not part of it.

**Hardware envelope**
This machine: Apple M5 Max, 18 CPU cores, 40 GPU cores, 128 GB unified memory. `las` uses only the CPU.

**Relation to the rest of Obstacle H**
H4 measured how far the laptop reaches in one hour, and H1 measured the cost at the goal's size. H5 asks how much of either is the sieve's own work. That is the most a better sieve could save.
