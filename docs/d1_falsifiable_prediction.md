# Falsifiable Prediction under Obstacle D / D1

**Claim (D1)**  
On hard balanced semiprimes of 40–80 decimal digits, the observed density of B-smooth values of an optimized degree-4 or degree-5 polynomial (Murphy-E maximized) exceeds the Dickman–de Bruijn prediction by a measurable margin after accounting only for the measured root properties (Murphy α). The excess is large enough to be distinguished from measurement noise on a fixed sieve region.

**Measurement protocol**
1. Select 10 hard balanced semiprimes N = p·q in the 40–80 digit range (no special algebraic form).
2. For each N obtain one polynomial from CADO-NFS and one from msieve (external instruments only).
3. Independently evaluate the homogeneous polynomial values over a fixed rectangular sieve region (a,b) of size sufficient for the digit length.
4. For several smoothness bounds B (e.g. 2^20, 2^24, 2^28), count the observed fraction of B-smooth values.
5. Compare the observed fraction to the Dickman–de Bruijn ρ(u) prediction, adjusting only by the measured Murphy α of the polynomial. Do not use the tools’ internal scores.
6. Record the gap (observed − predicted) and its variance across the 10 instances.

**Rejection criterion**  
The claim is rejected if, on these instances, the mean gap is smaller than two standard deviations of the measurement noise (determined by re-sampling the sieve region or by bootstrap).

**Hardware envelope**  
All measurements must complete on the primary laptop+GPU constraints stated in the roadmap. No distributed cluster is assumed.

**Relation to roadmap**  
This is the first concrete, testable claim under Obstacle D. A positive result weakens D; a negative result (rejection) forces a pivot or refinement of the sub-claim before larger speculation.
