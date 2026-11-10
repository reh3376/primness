# Falsifiable Prediction under Obstacle D / D5

Written on 2026-11-10, before any D5 statistic was computed.

**Claim (D5)**
On the ten D1 instances (hard balanced semiprimes of 40–80 decimal digits, one CADO-NFS degree-4 polynomial each) and the D1 sieve region, the B-smooth values of the polynomial cluster in the (a, b) region beyond what each value's own size and root properties predict. The clusters are locatable: regions picked for an excess of smooth values on one half of the values show an excess on the other half.

**What "predict" means**
Each value v gets the smooth probability π_B(v) = Q_B(c_v) + s_B(c_v). Here c_v is v's cofactor after its primes up to y = 2000, removed exactly. Q_B is the conditional baseline measured on consecutive integers (`docs/baseline_calibration.md` section 7), interpolated at ln c_v. s_B is the exact first-order term of the polynomial's root counts at the primes in (y, B] (section 9), taken per cofactor bin. D1 found no residual against this model over each instance as a whole. D5 asks whether the model is also right about where in the region the smooth values are.

**Measurement protocol**
1. Values: the D1 region, a in [−3000, 3000] and b in [1, 1500] with gcd(a, b) = 1, both sides, from `d1-measure --dump` at the bounds 2^20, 2^24 and 2^28. The dump's largest prime factor gives smoothness at 2^14, 2^16 and 2^18 as well. Those bounds are reported but not used for the verdict.
2. Partitions, fixed here: the rectangle cut into n × n tiles of equal ranges of a and of b, for n = 2, 5, 10, 25 and 50, and 64 sectors of equal angle of (a, b) in (0, π).
3. Halves: every value is assigned to half 0 or half 1 by a fixed integer hash of (a, b), which ignores the arithmetic of the values.
4. Per side, bound, partition, region and half: the values n, the smooth values O, the prediction P = Σ π and the Bernoulli variance V = Σ π(1 − π).
5. Per instance, side, bound and partition:
   - **dispersion**: χ² = Σ (O − P)²/V over the regions with both halves together, against the number of regions;
   - **split-half correlation** r of z = (O − P)/√V between the two halves across the regions;
   - **held-out lift**: take the tenth of the regions with the largest z in one half (at least one region). On the other half, compare the ratio O/P over those regions with the ratio O/P over that whole half: lift = (O_sel/P_sel)/(O_half/P_half) − 1, with its error from V. Measuring against the half's own ratio keeps an offset of the whole instance, which is D1's question, out of D5's. The lift is the mean of the two directions.
6. Over the ten instances: the mean held-out lift, with an error that is the larger of the spread across instances over √10 and the mean per-instance error over √10, and the mean split-half correlation.

**Control**
The rational side is a linear form, with one root at every prime and no algebraic structure. Any clustering it shows belongs to the method: the prediction, the noise model or the region. The algebraic side is read against it.

**Rejection criterion**
There are eighteen comparisons: three protocol bounds times six partitions. The claim is supported only if, at one or more of them, the algebraic side's mean held-out lift exceeds zero by more than three standard errors and exceeds the rational side's by more than three combined standard errors. Otherwise it is rejected on these instances. Three standard errors rather than two keeps the chance of a false support across the eighteen near 2%. A supported lift below 1% is recorded as structure without practical effect at this scale, since D5's claim is about improving the exponent or the crossover.

**Hardware envelope**
As for D1: the primary laptop constraints of the roadmap.

**Relation to D1**
D1 asked whether the values are smoother than their size and root properties say, on average over the region. It found that they are not, to 0.03–0.05% at 2^24 and 2^28. D5 asks the same question region by region. A model that is right on average can still be wrong locally. D5's claim needs it to be wrong locally in a way that can be found in advance.
