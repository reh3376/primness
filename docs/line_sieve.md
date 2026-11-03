# Line sieve for the smoothness count

Trial division over every prime up to B reaches 2^20 at about 300,000 values a second on this machine and would not reach the protocol's 2^28. From this change the harness decides B-smoothness with a line sieve (`src/linesieve.rs`), the classical method, and keeps trial division behind `--method trial` for cross-checks.

## Method

1. **Factor base.** For every prime p ≤ B the roots of f mod p (`src/fp.rs`: gcd with x^p − x, then Cantor–Zassenhaus splitting; primes below 64 by brute force), ln p scaled by 64 into a u16, and whether p divides the leading coefficient. The rational polynomial gets its own factor base the same way.
2. **Sieve a row.** For row b and every prime p with p ∤ b, every root r marks the positions a ≡ r·b (mod p) in [−A, A] with ln p; when p divides both b and the leading coefficient every position is marked (the projective root). The sum at a position is the log of the product of the distinct primes ≤ B dividing F(a, b).
3. **Candidates.** A position is a candidate when its sum is at least ln|F(a, b)| − slack, with slack = 2 ln B + 1 (the 1 covers rounding of the scaled logs).
4. **Resieve and verify.** A second pass collects the primes that hit each candidate. Dividing them out fully decides smoothness exactly: the cofactor is 1 if and only if every prime factor is at most B, because every prime ≤ B that divides the value was marked at least once. The largest prime factor comes from the same division, so every smaller bound is answered by the same pass.

## The one approximation

A non-candidate is counted as not smooth without verification. It can be smooth only if its repeated prime powers (the part of the factorization the single marking does not see) exceed e^slack = B², for example 2^56 at B = 2^28. For values of the sizes measured here that is far below Poisson noise, and the harness records the slack in every result (`smoothness.slack_ln`). On d1-01 at B = 2^14 through 2^20 the sieve's counts equal the trial-division counts exactly at every bound on both sides, over 5,472,915 values (`data/results/d1-01.scaled-down.json` against the `--method trial` run of the same region).

## Cost

Per row the sieve walks every prime once, so a row costs about π(B) operations plus (2A + 1)·ln ln B markings; at B = 2^28 and A = 3000 that is about 15 million operations a row, a second or two per 18 rows on this machine. The factor bases cost one root computation per prime, parallel over threads, and about 250 MB at 2^28. Memory beyond that is one u16 per position per side per thread.
