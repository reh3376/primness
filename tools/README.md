# External Tools as Measurement Instruments (Obstacle D / D1)

These tools are **not** part of the Primness runtime. They are external reference generators whose outputs we instrument. Their stated parameters, crossover sizes, and asymptotic claims are treated as hypotheses to test, not as facts.

## CADO-NFS
- Source: https://gitlab.inria.fr/cado-nfs/cado-nfs
- Build: `git clone ... && make` (requires GMP >= 5 with shared libs, C++11/20 compiler, CMake).
- Use: generate polynomials (Murphy-E after Bai et al. root optimization) and relations in the shared format.
- Output we consume: `.poly` files and relation files (`a,b:rat_factors:alg_factors`).

## msieve
- Source: https://github.com/upiter/msieve (public domain).
- Use: independent polynomial selection pipeline for 40–80 digit instances; relation format identical to CADO/GGNFS.
- Output: `.poly` / `.fb` files and relation files.

## YAFU
- Source: https://github.com/bbuhrow/yafu or MersenneForum forks.
- Use: rapid orchestration that already calls CADO-NFS and msieve; extract the polynomials and relations it produces and re-evaluate smoothness ourselves.

## Shared relation format (GGNFS / msieve / CADO)
```
a,b:rat1,rat2,...:alg1,alg2,...
```
Factors may omit primes < 1000; our harness must recompute or complete them for independent smoothness checks.

## Integration rule
No code from these projects is vendored. We call them (or parse their output files) only to obtain candidate polynomials and relation sets. All smoothness density measurements, Dickman–de Bruijn comparisons, and falsifiable claims under D1 are performed by the Primness Rust harness.
