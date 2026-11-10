# Falsifiable Prediction under Obstacle D / D2, part 1: several number fields

Written on 2026-11-10, before any D2 statistic was computed.

**Scope**
D2 claims that more algebraic structure gives relation collection better than the L_N[1/3] analysis allows: several number fields, fields of higher degree, or sieving on higher-dimensional varieties. Every one of those analyses rests on two things. The first is that each value is smooth as often as its size and root properties say, which is D1's question, closed at this scale. The second is that the values of different polynomials at the same (a, b) are smooth independently. This document tests the second, in the configuration where it carries the most weight: several algebraic fields sharing the rational side, the multiple number field sieve (MNFS), whose L[1/3] constant of about 1.902 against 1.923 comes from counting relations as independent events across fields. Higher degree and sieving in dimension 3 are left for later parts of D2.

**Claim (D2, part 1)**
On the ten D1 instances and the D1 region, pairs of values that belong together in the NFS are jointly smooth more often than their individual smooth probabilities, multiplied as independent events, predict. The pairs are the rational value G(a, b) with the value of any of four algebraic fields at the same (a, b), and the values of two algebraic fields at the same (a, b). The four fields share G, and so share the root m modulo N. An excess of this kind is the only way more fields can collect relations faster than the independent-event count, since every value is already as smooth as its own size and root counts say.

**The fields**
For each instance, F_0 is the D1 instrument polynomial f (from CADO-NFS). F_1, F_2, F_3 are rotations f + v·g for nonzero integers v with |v| ≤ 16, where g = Y1·x + Y0 is the shared rational polynomial. Every rotation has the same root modulo N, the same leading coefficient and the same resultant with g. It differs from f only in the two lowest coefficients. The three are chosen before any smoothness is measured, by this rule:
1. Each candidate v gets the score mean ln|F_v(a, b)| + α(F_v). The mean is over the coprime pairs of the D1 region and α is Murphy's α over the primes up to 2000, as d1-measure computes it. A lower score means a better field.
2. A candidate is used only if its irreducibility over Q is certified: the degrees of its irreducible factors modulo the primes up to 10^4 must leave no proper factor degree that all of them allow.
3. The three certified candidates with the lowest score are F_1, F_2, F_3. Ties go to the smaller |v|, then to the positive v.

**What "predict" means**
Every value of every field gets the smooth probability that D1 and D5 use: π_B(v) = Q_B(c_v) + s_B(c_v). That is the conditional baseline at the value's own cofactor after the primes up to y = 2000, plus the exact first-order term of that polynomial's own root counts between y and B (`docs/baseline_calibration.md` sections 7–9, `docs/d5_falsifiable_prediction.md`). Each field and bound has its own calibration κ = O/P, the observed smooth count over the predicted one. It is taken over the values whose cofactor exceeds B, since a value with a smaller cofactor is smooth for certain (π = 1). The joint prediction uses p = min(1, κπ), and p = 1 for the certain values. An offset of a whole field, which is D1's question, therefore stays out of D2's. The κ of the thirty rotated fields are reported as a D1 check on polynomials that were not root-optimized.

The model predicts no excess, and a small deficit. A prime above y that divides two of the four algebraic values at one (a, b) divides (v_i − v_j)·b³·G(a, b). It cannot divide v_i − v_j (|v_i − v_j| ≤ 32) or b (b ≤ 1500). So it divides G, and then the resultant of f and g. On these instances that resultant is N times 1, 3, 6 or 10, and the prime factors of N are far above 2^28. The same argument applies to F_i and G directly. Two related values therefore share no prime between y and B, where independent values would sometimes share one, and that makes joint smoothness slightly rarer than the product. The deficit is of order a few tenths of a percent at most. The primes up to y are removed exactly per value, so what the related values share there is already in each value's π.

**Measurement protocol**
1. Values: the D1 region, a in [−3000, 3000] and b in [1, 1500] with gcd(a, b) = 1, which is 5,472,915 values. Per field and value, the measurement records ln|F|, the largest prime factor when F is 2^28-smooth, and the log of the part made of the primes up to 2000 (`d1-measure --dump`, at the bounds 2^14, 2^16, 2^18, 2^20, 2^24 and 2^28). Root counts between 2000 and 2^28 come from `d1-tail`, per field.
2. Per field c and bound B, the counts n, O_c and P_c = Σπ_c, and κ_c = O_c/P_c over the values with π < 1.
3. Per pair (s, t) of fields at one (a, b): O_st, the number of values at which both are B-smooth, and E_st = Σ p_s·p_t.
4. Classes of pairs per instance:
   - **cross**: (F_i, G) for i = 0..3, four pairs;
   - **algebraic**: (F_i, F_j) for i < j, six pairs.
   For a class, O = ΣO_st, E = ΣE_st, and the ratio R = O/E − 1. The error is √V/E, where V is the exact variance of O when every value's smooth events are independent with probabilities p. V includes the covariance between pairs that share a field.
5. Over the ten instances: the mean of R weighted by 1/error². Its error is the weighted standard error times max(1, √(χ²/9)), where χ² is taken about the weighted mean. Instances whose noise differs fifty-fold are combined by their noise, and a spread beyond the noise widens the error.

**Control**
These are the same classes on pairs that share (a, b) but no arithmetic. Instance X's fields are paired with the next instance's (d1-10 with d1-01):
- (F_i of X, G of the next) for i = 0..3;
- (F_i of X, F_j of the next) for i < j.

Unrelated polynomials evaluated at the same points carry whatever the method, the region or the per-value prediction put into joint counts. They carry none of the arithmetic that ties the fields of one instance together. The control should give R near zero, and the related classes should give R at or slightly below it.

**Rejection criterion**
There are six comparisons: the two classes at the bounds 2^20, 2^24 and 2^28. The claim is supported only if, at one or more of them, the related class's weighted mean R exceeds zero by more than three errors and exceeds the control's by more than three combined errors. Otherwise it is rejected on these instances. A supported excess below 1% is recorded as structure without practical effect at this scale. A constant excess does not change an exponent, so a supported excess would next be tested for growth with the size of the values.

**Reported beside the verdict, not part of it**
- The standard pair (F_0, G) alone. It is the independence of the two sides that every NFS analysis assumes, and that D1 and D5 left untested.
- The κ of the rotated fields at every bound.
- The MNFS yield: relations from the four fields together, Σ_i O(F_i, G), over relations from F_0 alone, and the number of pairs (a, b) that give at least one relation.
- The bounds 2^14, 2^16 and 2^18.

**Hardware envelope**
As for D1: the primary laptop constraints of the roadmap.

**Relation to D1 and D5**
D1 found the values as smooth as their sizes and root counts make them, on average over the region. D5 found the same region by region. Both treated each side by itself. D2 part 1 asks whether values that the NFS combines are smooth together more often than apart.
