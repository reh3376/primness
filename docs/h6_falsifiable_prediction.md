# Falsifiable Prediction under Obstacle H / H6: the linear algebra at 200 digits, measured

Written on 2026-11-11, before any block Wiedemann timing beyond H4's was taken on this machine.

**Scope**
H1 measured the relation collection at 200 digits: 3,896 laptop-hours. The rest of a 200-digit factorization has stood in the roadmap as the records' share, linear algebra and polynomial selection at 9 to 17 percent of the total, or about 400 to 800 laptop-hours. H5 found that a perfect relation collection would leave exactly that rest, so it sets the floor under any change to the sieve. H6 measures its largest part: block Wiedemann on the matrix CADO-NFS's own c200 parameters would build.

**Claim (H6)**
On this machine, the block Wiedemann step of a 200-digit factorization with CADO-NFS's c200 parameters takes within a factor of 2 of 46 laptop-hours. That is far below the records'-share stand-in of about 400.

**Derivation**
- **Matrix size.** In H4's four matrices the number of rows N divided by ΣL ranges from 0.0292 (RSA-130) to 0.0334. ΣL is the sum over the two sides of 2^lpb / ln 2^lpb, the large-prime ideals that CADO-NFS's relation target counts. At c200, with lpb 32 and 33, ΣL = 5.69 × 10^8. Merging to c200's target density of 170 rather than c130's 110 is taken to shorten the matrix by a fifth. That gives N_200 = 13.3 to 15.2 million rows and W = 170 N = 2.3 to 2.6 × 10^9 nonzeros. The product N × W that sets the cost hardly depends on that fifth: a fifth fewer rows at 1.55 times the density leaves N² × density nearly unchanged.
- **Iterations.** With m = n = 64, Krylov does N/m + N/n products and mksol N/n, so 3N/64 in all.
- **Each product.** 9.5 × 10^-11 s per nonzero with thr=3x6. That is the constant H4 measured from RSA-100 to RSA-130, 9.5 × 10^-11 to 1.0 × 10^-10 across a 9-fold range of weight.
- **Lingen.** It runs with thr=1x1, as CADO-NFS ran it. Scaled from RSA-130's 206 s by N log N (m + n + log N), it takes 0.8 to 0.9 hours.
- **Overhead.** RSA-130's linear algebra phase was 6.4 percent longer than its Krylov, lingen and mksol together.
- **Total.** 40 to 53 laptop-hours, about 46 in the middle.

The assumption the measurement tests most is the constant cost per nonzero. At 200 digits the vector, about 8 bytes per row or 110 to 120 MB, no longer fits in the caches it fit at RSA-130.

**Measurement protocol**
1. **Density.** CADO-NFS's `merge` reruns on H4's RSA-130 purged relations with `-target_density 170` in place of 110. κ is the rows at 170 divided by the rows at 110. N_200 = 0.0292 × ΣL_200 × κ, using RSA-130's ratio because it is the nearest size.
2. **Synthetic matrices.** A new binary `h6-matrix` writes CADO-NFS's binary matrix format and its row and column weight files.
   - Row weights follow a gamma distribution with mean d and standard deviation 0.785 d, RSA-130's measured ratio.
   - Column positions are drawn from P(i) ∝ (i + 32)^-0.94, the NFS-like profile in CADO-NFS's `random_matrix` source. The standalone tool does not apply it. That law predicts RSA-130's real profile closely: its heaviest 1 percent of columns hold 48 percent of the nonzeros, against 50 percent in the real matrix.
   - No row repeats a column.
3. **Calibration.**
   - bwc's `krylov` runs with m = n = 64, thr=3x6, interval 500, start 0, end 3000 and skip_online_checks=1, after bwc's `prep`.
   - It runs on RSA-130's real matrix and on a synthetic matrix with RSA-130's N and density 111.4.
   - The synthetic matrix stands in only if its time per iteration is within 25 percent of the real one's. Otherwise H6 stops here and reports why.
   - The ratio ρ, real over synthetic, corrects the 200-digit time.
4. **The 200-digit matrix.**
   - A synthetic matrix with N_200 rows and density 170 runs through the same `krylov` for at least 20 minutes, under `hw-sample`.
   - t is the median of the per-interval times after the first interval, multiplied by ρ.
5. **Lingen.** `lingen_b64` runs with m = n = 64, thr=1x1 and random-input-with-length = N_200/32, timed.
6. **Total.** T_LA = (3 N_200/64 × t + T_lingen) × 1.064.

**Rejection criterion**
H6 holds if T_LA lies in [23, 92] laptop-hours. Otherwise H6 is rejected, and the measured cost replaces the stand-in wherever the roadmap uses it.

**Reported beside the verdict, not part of it**
- κ, and N_200 over the range of H4's ratios.
- ρ, and the cost per nonzero at the 200-digit size beside RSA-130's.
- Synthetic matrices between the two sizes, to show how the cost per nonzero moves with N.
- Lingen with more threads than CADO-NFS gave it.
- The linear algebra's share of a complete 200-digit factorization, and what a perfect relation collection would leave.
- Peak memory, power and clocks. The 20-minute `krylov` run reaches sustained clocks.

**Hardware envelope**
This machine: Apple M5 Max, 18 CPU cores, 40 GPU cores, 128 GB unified memory. bwc uses only the CPU.
