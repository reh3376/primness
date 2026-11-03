# D1 Test Instances (40–80 decimal digits)

Hard balanced semiprimes N = p · q with no special algebraic form. Used for the first falsifiable prediction under Obstacle D / D1. External tools supply only the polynomials; the Primness harness computes independent B-smooth densities.

| ID | Digits | N (decimal) | Notes |
|----|--------|-------------|-------|
| d1-01 | 40 | 1234567890123456789012345678901234567890 | placeholder; replace with genuine hard semiprime |
| d1-02 | 44 | ... | placeholder |
| d1-03 | 48 | ... | placeholder |
| d1-04 | 52 | ... | placeholder |
| d1-05 | 56 | ... | placeholder |
| d1-06 | 60 | ... | placeholder |
| d1-07 | 64 | ... | placeholder |
| d1-08 | 68 | ... | placeholder |
| d1-09 | 72 | ... | placeholder |
| d1-10 | 80 | ... | placeholder |

**Generation rule**  
Each N is the product of two primes of roughly equal size, verified prime by deterministic Miller-Rabin or equivalent, and free of SNFS-friendly algebraic structure. Actual values will be generated and substituted before the first measurement run.

**Next**  
Obtain one polynomial per instance from CADO-NFS and one from msieve, then run the independent evaluation inside `d1_measure`.
