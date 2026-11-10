# D2 Part 1 on the D1 Instances: Several Number Fields Are Smooth Independently (2026-11-10 16:45 EST)

**Metadata**
- **Created**: 2026-11-10 16:45 EST
- **Last Modified**: 2026-11-10 16:45 EST
- **Version**: 1.0
- **Tags**: obstacle-D, D2, MNFS, joint-smoothness, independence, null-result
- **Related Files**: docs/d2_falsifiable_prediction.md, docs/d2_multiple_fields.md, src/bin/d2_fields.rs, src/bin/d2_joint.rs, src/fp.rs, scripts/d2_joint.py, scripts/d2_run_all.sh, data/results/d2_multiple_fields.md, data/results/d2/, data/instruments/d2/

## Why

The owner chose D2 after D5. D2 covers several number fields, fields of higher degree, and sieving on higher-dimensional varieties. Every one of those L[1/3] analyses multiplies the smooth probabilities of the values that a relation combines, as independent events. D1 and D5 tested each side by itself, and neither tested that independence. This change tests it where it carries the most weight: four algebraic fields sharing the rational side, the multiple number field sieve.

## What was built

- `docs/d2_falsifiable_prediction.md`: the claim, the selection rule for the extra fields, the classes, the control and the criterion, written before any D2 statistic. One refinement came before the first run, again before any statistic: κ is fitted over the values with π < 1, since a value whose cofactor is below B is smooth for certain, and p = min(1, κπ).
- `fp::factor_degrees` (distinct-degree factorization mod p) and `fp::certify_irreducible` (irreducibility over Q from factor degrees at small primes), with tests: x^4 + 1 is never certified, and three reducible quartics are never certified.
- `d2-fields`: every rotation f + v·g with 0 < |v| ≤ 16, scored by mean ln|F_v| over the region plus α to 2000. All 320 candidates are certified by one or two primes below 60. The three best per instance go to `data/instruments/d2/`. The binary checks that every rotation keeps the resultant with g (Res/N is 1, 3, 6 or 10).
- `d2-joint`: reads up to ten columns from the dumps, checks that the (a, b) agree, fits κ per column, and sums the joint counts, their expectation and their exact variance per pair and class (the class second moment is tested against enumeration). It runs in 3.5 s per instance, and a rerun is byte-identical.
- `scripts/d2_joint.py`: the prediction tables per column, reusing D5's `prediction_grid`, the manifests (`d2-joint-manifest/1`), the weighted combination with the Birge widening, and the criterion. The instruments' totals match D5's to 4.5 × 10⁻⁶.
- `scripts/d2_run_all.sh`: end to end. The root counts of the thirty fields took 72 s and the measurements 41–64 s each.

## What was measured

Criterion values (related R over its error / related minus control over the combined error) at 2^20, 2^24, 2^28:
- cross (F_i, G): +1.1/−0.3, +0.2/−0.4, −0.4/−0.5;
- algebraic (F_i, F_j): −0.9/−0.6, +1.4/+0.7, −0.4/−0.7.

Rejected.

R at 2^28: cross −0.01% ± 0.03% (control +0.01% ± 0.04%), algebraic −0.02% ± 0.06% (control +0.03% ± 0.06%). Standard pair (F_0, G): +0.02% ± 0.26%, +0.04% ± 0.10%, −0.03% ± 0.05% at the protocol bounds. χ² of the instances 2.6–12.6 on ten at the protocol bounds.

Rotated fields' κ − 1: χ² about zero 30.1 and 32.7 on thirty at 2^24 and 2^28. The weighted mean is +0.01% ± 0.04% and −0.05% ± 0.02%; the largest deviation is d1-08.r-15, −3.1 errors at 2^28.

MNFS yield, four fields over one: relations 2.71, 3.00, 3.21; pairs with a relation 2.54, 2.59, 2.50 at the protocol bounds.

## Questions answered

- Are values that the NFS combines smooth more often together than apart? No. To 0.03% (G with a field) and 0.06% (two fields) at 2^28, they are independent given their own sizes and root counts. The deficit that the shared-prime exclusion predicts is below 0.1–0.2%.
- Does D1's closure need root-optimized polynomials? No: thirty rotations that lost their root optimization are at their prediction.

## Questions raised

- The higher-degree and dimension-3 forms of D2 are untested. A degree-5 or degree-6 field for the same N needs CADO-NFS selection at that degree and a check that the conditional baseline covers the larger cofactors.
- The owner has to choose the next step and decide on the D1 verdict draft.

## Hardware

Thirty d1-measure runs of 41–64 s on 18 threads with hardware sampling (traces under `data/hardware/`, gitignored), d1-tail for sixty sides in 72 s, d2-fields in 3 s, and d2-joint in 3.5 s per instance (under ten with the Python prediction tables). d2-joint peaks at 3.2 GB resident, because it reads its eight dumps whole and in parallel. The dumps take 315 MB per field (9.5 GB, gitignored).
