# D2 part 1 on the D1 instances: several number fields sharing the rational side

`docs/d2_falsifiable_prediction.md` states the claim, the protocol and the rejection criterion, written before any statistic below was computed. This document records the method as built and the result.

## 1. Method

**Fields.** `d2-fields` scores every rotation f + v·g of each instrument polynomial with 0 < |v| ≤ 16 by its mean ln|F_v| over the D1 region plus its α to 2000. It certifies irreducibility over Q from the factor degrees modulo small primes (`fp::certify_irreducible`), and all 320 candidates are certified, by one or two primes below 60. It writes the three best per instance to `data/instruments/d2/<id>.r<v>/cado.poly`, with the full table of candidates in `data/results/d2/fields.csv`. Every rotation keeps the instrument's resultant with g, and N divides it. The rotated fields score 1.3 to 4.1 worse than the instruments, because they lose the instruments' root optimization: their α is −3.0 to +0.5 against −3.2 to −5.9. The region is not skewed, so a rotation that shrinks the constant coefficient can give smaller values than the instrument. In seven instances at least one chosen field does (d1-03, d1-04, d1-05, d1-07, d1-08, d1-09, d1-10), by up to 2.5 in mean ln|F|, but never by enough to make up the α.

**Values.** `d1-tail` counts each field's roots at every prime up to 2^28 (`data/results/d2/root_tail.json`). `d1-measure` measures each field over the D1 region at the six bounds 2^14 to 2^28, with the per-value dump (`data/results/d2/<field>.*`, dumps under `data/dumps/d2/`, gitignored). The instruments' dumps are D5's.

**Prediction.** Every value of every column (F_0, F_1, F_2, F_3, G) gets D5's smooth probability π per bound, from its own cofactor histogram and its own root counts. `scripts/d2_joint.py` builds the tables, and the instruments' totals agree with D5's to 4.5 × 10⁻⁶, within the six-digit rounding of D5's table. Each column is scaled by its own κ = O/P over the values with π < 1, and p = min(1, κπ).

**Joint counts.** `d2-joint` reads the ten columns of an instance and the next instance's from their dumps, checks that every dump lists the same (a, b) in the same order, and sums the following per pair of columns and bound:

- the values at which both are smooth, O;
- E = Σ p_s p_t;
- the variance of O under independence.

Per class of pairs it adds the covariance through shared columns, from the second moment Σ over ordered pairs of pairs of Π p over the columns they use (tested against enumeration). A union counts the (a, b) at which G and at least one algebraic field are smooth. Each instance takes under ten seconds, prediction tables included, and a rerun reproduces its sums byte for byte. The thirty field measurements took 41 to 64 s each.

## 2. Result

**The criterion.** For each related class: its weighted mean R over its error, then the related minus the control over their combined error. Support needs both above 3.

| class | 2^20 | | 2^24 | | 2^28 | |
|---|---:|---:|---:|---:|---:|---:|
| | related | minus control | related | minus control | related | minus control |
| cross (F_i, G) | +1.1 | −0.3 | +0.2 | −0.4 | −0.4 | −0.5 |
| algebraic (F_i, F_j) | −0.9 | −0.6 | +1.4 | +0.7 | −0.4 | −0.7 |

**The claim is rejected on these instances.** The largest value is +1.4, for the algebraic pairs at 2^24.

**R in size.** The weighted mean over the ten instances, ± its error:

| class | 2^20 | 2^24 | 2^28 |
|---|---:|---:|---:|
| cross | +0.14% ± 0.12% | +0.01% ± 0.07% | −0.01% ± 0.03% |
| cross control | +0.19% ± 0.16% | +0.05% ± 0.06% | +0.01% ± 0.04% |
| algebraic | −0.25% ± 0.27% | +0.15% ± 0.11% | −0.02% ± 0.06% |
| algebraic control | −0.02% ± 0.31% | +0.04% ± 0.12% | +0.03% ± 0.06% |

Values that belong together are jointly smooth as often as the product of their own probabilities says, to 0.03% for G with the fields and 0.06% for two fields at 2^28. An excess of 0.1–0.2% at 2^28, or 0.2–0.3% at 2^24, would have reached three errors. At the protocol bounds the χ² of the ten instances about their mean is 2.6 to 12.6 on ten, consistent with the per-instance errors. Where it exceeds 9, the error is widened by √(χ²/9), which is at most 1.18 at the protocol bounds. The largest χ² are the algebraic control's at 2^14 and 2^16 (20.8 and 14.1), where a few joint counts carry an instance.

**The standard pair** (F_0, G) is +0.02% ± 0.26% at 2^20, +0.04% ± 0.10% at 2^24 and −0.03% ± 0.05% at 2^28. The two sides of the ordinary NFS are independent given their sizes and root counts, to the precision D1 reached for each side alone.

**The predicted deficit.** Related values share no prime between 2000 and B, and the prediction expected a small deficit from that. It is not resolved. At 2^28 the related classes sit below their controls by −0.02% ± 0.05% (cross) and −0.06% ± 0.08% (algebraic), so the deficit is smaller than about 0.1% for G with a field and 0.2% for two fields.

**The rotated fields as a D1 check.** None of the thirty was root-optimized beyond the choice of v, yet they are as smooth as their sizes and root counts predict. Their κ − 1 has χ² about zero of 30.1 on thirty at 2^24 and 32.7 at 2^28. The weighted mean is +0.01% ± 0.04% and −0.05% ± 0.02%; the largest single deviation is d1-08.r-15 at 2^28, −3.1 errors. The instruments' κ in the same run is +0.06% ± 0.06% and −0.01% ± 0.04%. D1's closure therefore holds on forty polynomials, not ten.

**MNFS yield.** Summed over the instances, the four fields give 2.71, 3.00 and 3.21 times the relations of the instrument alone at 2^20, 2^24 and 2^28. They give 2.54, 2.59 and 2.50 times the pairs (a, b) with at least one relation. Each rotated field is worse than the instrument, so three of them add 1.7 to 2.2 instruments' worth of relations. The cross class's R says this gain is exactly what the fields' individual probabilities predict.

## 3. Reading

Over the D1 region of the ten instances, the values of four number fields that share a rational side are smooth independently of each other and of the rational value, given each value's size and root counts. That independence is what the L[1/3] analysis of the multiple number field sieve assumes, and here it holds to 0.03–0.06% at 2^28. The fields bring more relations only by adding more independent chances. That is the known MNFS gain, which lowers the constant, not the exponent, and the measured yield is exactly that. D2's claim, in the multiple-field form tested here, is rejected at this scale.

The test leaves the other forms of D2 open: fields of higher degree than the optimum, and sieving in dimension 3 or on other varieties. Both change the sizes of the values, which D1's closure prices. Dimension 3 also changes which values are combined, the question asked here. A degree-5 or degree-6 field for the same N, measured with the same harness, would extend the D1 closure to degrees other than 4. It would need CADO-NFS selection at those degrees and a check that the conditional baseline covers the larger cofactors.

## Files

| File | Role |
|---|---|
| `docs/d2_falsifiable_prediction.md` | The claim, the fields' selection rule, the protocol, the control and the criterion, fixed before measuring. |
| `src/bin/d2_fields.rs` | The rotations, their scores, the irreducibility certificates, the chosen fields and the table. |
| `src/fp.rs` | `factor_degrees` (distinct-degree factorization mod p) and `certify_irreducible`. |
| `src/bin/d2_joint.rs` | Joint smooth counts against the product of the predictions, with the exact variance per class. |
| `scripts/d2_joint.py` | The prediction tables per column, the manifests, the classes and controls, the combination over instances and the criterion; writes `data/results/d2_multiple_fields.{csv,md}`. |
| `scripts/d2_run_all.sh` | Fields, root counts, measurements, dumps and analysis end to end. |
| `data/instruments/d2/`, `data/results/d2/` | The thirty fields; their root tails, measurements and the per-instance joint sums (`joint/<id>.csv`). |
