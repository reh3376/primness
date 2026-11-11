# Falsifiable Prediction under Obstacle H / H1: the laptop's distance from 200 digits, measured

Written on 2026-11-10, before any 200-digit sieving statistic was computed on this machine.

**Scope**
Obstacle H is the case the README names: the laptop-plus-GPU constraints force a pivot to specialized hardware. With Obstacle D closed (`docs/obstacle_d_verdict.md`), relation collection by the number field sieve costs what the standard smoothness model says, so the cost is a property of the algorithm and the machine. Every requirement a hardware specification states is a multiple of what a known machine does on a known task. So far that multiple was scaled from records on other machines: the RSA-240 and RSA-250 factorizations on Intel Xeon Gold 6130 cores, carried to 200 digits by L_N[1/3, 1.923] and to this laptop by counting it as 18 of those cores. That gives 6,800 to 7,800 laptop-hours for a 200-digit factorization, and about 6,300 (5,800 to 6,800) for its relation collection, which is 83 to 91 percent of the records' total. H1 measures the relation collection on this laptop instead.

**Claim (H1)**
On this machine, with CADO-NFS (commit 692ecb7) and its own 200-digit parameters, collecting the relations for a 200-digit hard semiprime costs within a factor of 2 of 6,300 laptop-hours.

**The instance**
RSA-200, 200 digits, with the degree-5 polynomial CADO-NFS ships as `parameters/polynomials/c200.poly` (found by CADO-NFS with default parameters; Murphy E 2.90e-12). RSA-200 is a public challenge number whose factorization was published in 2005. This measurement factors nothing and keeps no relations: it samples the cost of sieving.

**Measurement protocol**
1. Parameters: `params.c200` exactly. lim0 = 1.3e8, lim1 = 1e8, lpb0 = 32, lpb1 = 33, mfb0 = 85, mfb1 = 96, 13 curves per side, I = 15, special-q on side 1 from qmin = 5e7. `makefb` writes the side-1 factor base to lim1 with maxbits 15, as `cadofactor` does; `las` builds side 0's.
2. Sampling: `las -random-sample 40 -t 18 -T` on the ranges [q, 1.05 q) for q = 5e7, 1e8, 2e8, 4e8, 8e8 and 1.6e9, with a further doubling if the integral below has not reached the target by then. Each range runs once under `hw-sample`. Per range: the relations per special-q y(q), the elapsed seconds per special-q t(q) at 18 threads, `las`'s breakdown of its time, and its survivor counts.
3. Target: CADO-NFS's own rule when the parameters do not set `rels_wanted`, 0.81 × Σ over the sides of 2^lpb / ln 2^lpb = 4.61e8 relations. These are raw relations, counted as `las` reports them and as `cadofactor` counts them.
4. Cost: a special-q is a prime q with a root of f mod q, and an irreducible f has one root mod q on average. Up to Q there are therefore about ∫ dq / ln q special-q's, the relations are R(Q) = ∫ y(q) dq / ln q and the time is T(Q) = ∫ t(q) dq / ln q, from qmin. ln y and ln t are interpolated linearly in ln q between the sampled ranges, and beyond the last one they are extrapolated on the last segment's slope. Q_max is where R reaches the target, and T(Q_max) in hours is the measured cost.
5. Duplicates are not removed. The target counts raw relations as `cadofactor` does, and its filtering loop usually asks for some more. The measured cost is therefore slightly below a full run's, which errs toward a smaller gap.

**Rejection criterion**
H1 holds if T(Q_max) lies in [3,150, 12,600] laptop-hours. Otherwise H1 is rejected, the measured cost replaces the scaled one in every gap the roadmap states, and the figures for 240 to 260 digits are rescaled by the same ratio.

**Reported beside the verdict, not part of it**
- `las`'s breakdown of the time (norms, sieving, cofactorization) and the survivor counts per special-q. These are the operation counts a hardware specification is written in.
- Relations and special-q's per laptop-hour, and the multiple of each that one hour at 200 digits needs.
- Peak memory, power, temperature and the activity of the two CPU tiers from the hardware samples.
- The linear algebra is not measured here. The records' share, about 9 percent, stands in for it until it is measured.

**Hardware envelope**
This machine: Apple M5 Max, 18 CPU cores, 40 GPU cores, 128 GB unified memory. The run is the measurement of the envelope, so nothing beyond it is used.

**Relation to Obstacle D**
D found the values of the number field sieve exactly as smooth as their sizes and root counts make them, with no excess to exploit, at 40 to 80 digits and with no trend toward larger sizes. What remains between this laptop and the goal is a multiple of throughput. H1 measures that multiple at the goal's own size.
