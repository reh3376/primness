# H6: block Wiedemann at 200 digits on this laptop

`docs/h6_falsifiable_prediction.md` states the claim, the protocol and the criterion. It was committed (2fc8e2e) before any block Wiedemann timing beyond H4's was taken. This document records the runs and the result.

## 1. Method

**The matrix size.** CADO-NFS's `merge` reran on H4's RSA-130 purged relations at c200's target density of 170. It gave 1,004,812 rows and weight 170,835,074, against 1,183,823 rows at c130's 110, so κ = 0.849. In H4's four matrices the rows divided by ΣL, the large-prime ideal count, are 0.0309, 0.0334, 0.0315 and 0.0292. With RSA-130's ratio and κ, a 200-digit matrix at c200's lpb of 32 and 33 has **N_200 = 14,107,489 rows** and 2.40 × 10^9 nonzeros. Over H4's range of ratios that becomes 14.1 to 16.1 million rows. The size is estimated from H4's matrices. A real 200-digit matrix would need H1's 3,896 hours of relations.

**The synthetic matrices.** `h6-matrix`, a new Rust binary with three tests, writes CADO-NFS's binary matrix format.
- Row weights are gamma-distributed with RSA-130's ratio of standard deviation to mean, 0.785.
- Column positions follow P(i) ∝ (i + 32)^-0.94, the profile in CADO-NFS's `random_matrix` source. The standalone tool draws columns uniformly instead.
- At the 200-digit size the heaviest 1 percent of columns hold 54.9 percent of the nonzeros; the law says 55.4.
- At RSA-130's shape the synthetic matrix's heaviest 1 percent hold 47.7 percent, against the real matrix's 50.0.

**The runs.** `scripts/h6_linalg.sh` runs bwc's `prep`, then `krylov` with m = n = 64, thr=3x6, interval 500 and skip_online_checks=1, under `hw-sample`.
- It runs on RSA-130's real matrix, on its synthetic twin, on synthetic matrices of N_200/4 and N_200/2 rows, and on the 200-digit matrix.
- Each interval's time comes from the checkpoint files' timestamps.
- `lingen_b64` runs with thr=1x1, as CADO-NFS ran it in H4, on random input of the 200-digit sequence length 440,860.
- `scripts/h6_summary.py` writes `data/results/h6_linalg.{md,csv}` and, per run, `data/results/h6/<tag>.intervals.csv`.

## 2. Result

| matrix | rows | nonzeros | s per iteration | ns per nonzero |
|---|---:|---:|---:|---:|
| RSA-130, real | 1,183,823 | 1.32 × 10^8 | 0.0108 | 0.082 |
| synthetic, RSA-130's shape | 1,183,823 | 1.32 × 10^8 | 0.0120 | 0.091 |
| synthetic, N_200/4 | 3,526,872 | 6.0 × 10^8 | 0.0385 | 0.064 |
| synthetic, N_200/2 | 7,053,744 | 1.20 × 10^9 | 0.0975 | 0.081 |
| synthetic, N_200 | 14,107,489 | 2.40 × 10^9 | 0.2676 | 0.112 |

- **Calibration.** The synthetic twin is 11.5 percent slower than the real matrix, inside the 25 percent gate, so ρ = 0.897.
- **The 200-digit iteration.** 0.2676 s × ρ = 0.240 s, 0.100 ns per nonzero against 0.082 at RSA-130. The cost per nonzero rises about 1.2 to 1.4 times once the vector, 113 MB, no longer fits in the caches.
- **Iterations.** 3 N_200 / 64 = 661,289 products, 44.1 hours.
- **Lingen.** 3,416 s, 0.95 hours.
- **Overhead.** Multiplied by 1.064, RSA-130's measured overhead.

**T_LA = 47.9 laptop-hours. H6 holds**: the prediction was 46 within a factor of 2, [23, 92].

**Other work on the machine.** The machine was not reserved. The 200-digit run's slower intervals all coincide with activity on the GPU, which bwc does not use: 10 to 26 percent active. The five intervals at 248 to 254 ms had the GPU at 2 to 3 percent.
- The synthetic twin's last three calibration intervals ran with the GPU 63 to 94 percent active. That inflates its median and lowers ρ.
- Counting only clean intervals (GPU under 5 percent): ρ = 1.122, t_200 = 0.280 s and **T_LA = 55.8 laptop-hours**, also inside the band.
- With the matrix at the top of H4's range of ratios, the products scale as N² and the total is 62.5.

The 200-digit linear algebra is therefore **48 to 63 laptop-hours**.

## 3. Reading

- **The stand-in was 8 times too high.** The roadmap carried the linear algebra and polynomial selection as the records' 9 to 17 percent of the total, about 400 to 800 laptop-hours. With CADO-NFS's c200 parameters, the linear algebra alone is 48 to 63 hours, 1.2 to 1.6 percent of relation collection and linear algebra together. A complete 200-digit factorization on this laptop is about 3,950 laptop-hours plus polynomial selection.
- **What a perfect relation collection would leave.** With relation collection at zero cost, the linear algebra's 48 to 63 hours remain, so the factorization would be at most 63 to 82 times faster. Polynomial selection is not counted. It was 5 to 6 percent of the wall time in H4's runs. If it kept that share at 200 digits, about 200 to 250 laptop-hours, the ceiling would fall to about 15 times. H5's 200-digit ceiling for a free relation collection, 5.9 to 11 times from the records' share, moves up accordingly. It stays far below the 4,000 times one hour needs.
- **Where the time goes.** The products are memory-bound. During clean intervals of the 200-digit run, the CPU drew 32 to 33 W with the tiers at 3.3 and 2.6 GHz. The products are 92 percent of the linear algebra, and lingen, single-threaded as CADO-NFS runs it, is 2 percent. At this size block Wiedemann is a bandwidth problem, and the GPU, measured at 1.5 times the CPU's copy bandwidth in H2, is not tried here.
- **What remains estimated.** The matrix is synthetic, calibrated against one real matrix at 130 digits. Its size comes from H4's matrices scaled by the large-prime ideal count. The long runs ran at sustained clocks, with the CPU at 89 °C at most.

## Files

| File | Role |
|---|---|
| `docs/h6_falsifiable_prediction.md` | The claim, the protocol and the criterion, committed before the runs. |
| `src/bin/h6_matrix.rs` | Binary `h6-matrix`: synthetic NFS-shaped matrices in CADO-NFS's binary format, with tests of the format, the weights and thread independence. |
| `scripts/h6_linalg.sh` | The merge at density 170, the five `krylov` runs and `lingen` under `hw-sample`, then the summary. |
| `scripts/h6_summary.py` | κ and N_200 (`--n200`), the per-interval times and their activity, ρ and the gate, the total, the criterion and the clean-interval version; writes `data/results/h6_linalg.{md,csv}`. |
| `data/results/h6/` | The two merge lines, and per run the matrix line, `krylov`'s progress lines, the checkpoint times, the per-interval table and the `hw-sample` summary. Also lingen's time and the CADO-NFS commit. |
