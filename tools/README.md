# External Tools as Measurement Instruments (Obstacle D / D1)

These tools are **not** part of the Primness runtime. They are external reference generators whose outputs we instrument. Their stated parameters, crossover sizes, and asymptotic claims are treated as hypotheses to test, not as facts.

## CADO-NFS
- Source: https://gitlab.inria.fr/cado-nfs/cado-nfs
- Build: `git clone ... && make` (requires GMP >= 5 with shared libs, C++11/20 compiler, CMake).
- Use: generate polynomials (Murphy-E after Bai et al. root optimization) and relations in the shared format.
- Output we consume: `.poly` files and relation files (`a,b:rat_factors:alg_factors`).

## msieve
- Source: https://github.com/upiter/msieve (public domain).
- Use: NFS polynomial selection when the input is large enough for that binary. The v1.54 help text limits `-n` to 80+ digits. In the checkout outside this tree, `MIN_NFS_BITS` is 264, and `common/driver.c` enters NFS polynomial selection only when `mp_bits(n)` is greater than 264. The degree-4 parameter table in `gnfs/poly/poly_param.c` starts at log10(N) = 80. The ten proved semiprimes are 131 through 264 bits, so this binary takes the quadratic-sieve path for them. `write_poly` still emits `N`, `SKEW`, `R*`, and `A*` when NFS selection does run.
- Output: `.poly` / `.fb` files and relation files.

## YAFU
- Source: https://github.com/bbuhrow/yafu or MersenneForum forks.
- Use: rapid orchestration that already calls CADO-NFS and msieve; extract the polynomials and relations it produces and re-evaluate smoothness ourselves.

## Shared relation format (GGNFS / msieve / CADO)
```
a,b:rat1,rat2,...:alg1,alg2,...
```
Factors may omit primes < 1000; our harness must recompute or complete them for independent smoothness checks.

## What may exist in this repo

A third-party tree is copied in only when its license can sit beside `MIT OR Apache-2.0` and the copy does not become the D1 measurement. Instrument scores stay out of the harness.

- CADO-NFS is LGPL-2.1 (repository `cado-nfs/cado-nfs`). Its source is not copied into this tree. The harness reads polynomial files it emits.
- msieve's own Readme places that code in the public domain (Jason Papadopoulos; the checkout named above is `upiter/msieve`). The license would allow a copy. The numerical claim still has to be computed here, so the build stays outside this tree and CI does not compile it. The harness reads its factor-base polynomial (`N`, `SKEW`, `R*`, `A*`). On this Mac, the stock Apple clang build stops because `gettimeofday` is undeclared in `common/util.c`. A checkout outside this tree builds after that file includes `<sys/time.h>`. That binary is not in this repository, and polynomial selection has not been run.
- YAFU's repository root has no `LICENSE` file, and `README.md` on `master` does not state one. As of YAFU 3.0 that tree also bundles msieve and GGNFS sieve binaries. It is not copied in. Polynomials it writes can be parsed when they use one of the two formats above; the record's `source` stays `unspecified` unless the caller names the tool.

The record is `schemas/instrument-polynomial/1.json` (`$id` `instrument-polynomial/1`). `format` is `cado` or `msieve-fb`. `source` is `cado-nfs`, `msieve`, `yafu`, or `unspecified`. Degree in this version is 1 through 6. The D1 claim still requires an optimized degree-4 or degree-5 polynomial; this schema does not decide that.

## Running CADO-NFS polynomial selection here

CADO-NFS is built outside this tree at `~/external/cado-nfs` (commit recorded in each log) with `GMP=/opt/homebrew` and its own Python venv (`scripts/setup-venv.sh` there). Its `cado-nfs.py` driver has no parameter file within 3 digits of 40, so the ten instances use the `polyselect` and `polyselect_ropt` binaries directly:

```sh
NQ=1000 KEEP=40 ROPTEFFORT=5 sh scripts/cado-polyselect.sh d1-01 <N> 4 420 20000
python3 scripts/cado-best-poly.py data/instruments/d1-01/ropt.polys data/instruments/d1-01/cado.poly
```

`scripts/cado-polyselect.sh` runs both stages under `hw-sample` and writes `data/instruments/<id>/`: `stage1.polys`, `ropt.polys`, `polyselect.log` (exact commands, CADO commit, instrument stderr) and `polyselect.hardware.json` (the `hardware-sample/1` finish record). `scripts/cado-best-poly.py` copies the candidate with the largest MurphyE to `cado.poly`, keeping the score only as a comment. The harness reads `cado.poly` with `--source cado-nfs` because the caller knows that program wrote it. Parameters follow `parameters/factor/params.c60` scaled down (degree 4, P = 420, incr 60); the MurphyE bounds and area (`Bf = Bg = 2^17`, area 1e12) only rank candidates.

## Integration rule
Call these programs, or parse their polynomial files, to obtain candidate polynomials and relation sets. All smoothness density measurements, Dickman–de Bruijn comparisons, and falsifiable claims under D1 are performed by the Primness Rust harness.
