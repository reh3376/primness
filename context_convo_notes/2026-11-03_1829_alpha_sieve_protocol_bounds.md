# Murphy α, the Line Sieve, and d1-01 at the Protocol Bounds (2026-11-03 18:29 EST)

**Metadata**
- **Created**: 2026-11-03 18:29 EST
- **Last Modified**: 2026-11-03 18:29 EST
- **Version**: 1.0
- **Tags**: obstacle-D, D1, murphy-alpha, line-sieve, protocol-bounds, dickman, measurement
- **Related Files**: docs/murphy_alpha.md, docs/line_sieve.md, src/alpha.rs, src/fp.rs, src/linesieve.rs, data/results/d1-01.scaled-down.json, data/results/d1-01.scaled-down.b28.json

## What was done

1. **Murphy α recorded and implemented.** `docs/murphy_alpha.md` states the definition with its sources (Murphy 1999 §3.2; Bai, Bouvier, Kruppa, Zimmermann 2016 §2): α(F) = Σ_p≤B (1/(p−1) − E_p(F)) ln p with E_p the expected p-adic valuation over coprime pairs, affine and projective parts, computed exactly by lifting roots mod p. `src/alpha.rs` implements it with tests on simple roots, repeated roots (x², E = 2/(p−1)), a projective root (2a + b, E_2 = 2/3), and α(x) = Σ ln p/(p²−1). For the d1-01 polynomial the harness computes α = -3.9640 (algebraic) and +0.5695 (rational). The instrument's unread comment line says −3.96.
2. **α applied.** The result contract moved to `d1-measurement/2` (`alpha_applied` true, `alpha` object, per side `predicted_alpha`, `gap_alpha`, and also `predicted_local`, `gap_local` for the local-density form ρ(u) + ρ′(u)/ln B). `implied_alpha` stays for comparison.
3. **Line sieve.** `src/fp.rs` finds roots of f mod p (gcd with x^p − x, Cantor–Zassenhaus; tested against brute force up to 3000 and on 32-bit primes). `src/linesieve.rs` sieves each row with ln p at a ≡ r·b (mod p), marks candidates within 2 ln B + 1 of ln|F|, resieves to collect the primes of each candidate and divides them out fully, so smoothness and the largest prime factor are exact for candidates; the one approximation is that a non-candidate (repeated prime powers above B²) counts as not smooth (`docs/line_sieve.md`). On d1-01 at 2^14 to 2^20 the sieve's counts equal the trial-division counts exactly at every bound on both sides over 5,472,915 values, in 0.5 s against 17.6 s. `--method trial` keeps trial division for cross-checks; `smoothness` in the result records which was used.
4. **Protocol bounds.** d1-01 measured at 2^20, 2^24, 2^28 on the same region: factor bases of 14,630,843 primes built in 16.5 s, sieve in 23 s, 40 s in all, CPU active 0.99, 84.6 °C peak. `data/results/d1-01.scaled-down.b28.json`.

## Results on d1-01

| run | bound | observed | raw Dickman | raw gap / se | α-adjusted | α gap / se | local density | local gap / se |
|---|---|---|---|---|---|---|---|---|
| 2^14–2^20 | 2^14 | 0.00416 | 0.00145 | +99 | 0.00403 | +4.7 | 0.00302 | +42 |
| 2^14–2^20 | 2^16 | 0.01320 | 0.00566 | +155 | 0.01311 | +1.9 | 0.01041 | +57 |
| 2^14–2^20 | 2^18 | 0.03101 | 0.01552 | +209 | 0.03131 | -4.1 | 0.02593 | +69 |
| 2^14–2^20 | 2^20 | 0.05888 | 0.03340 | +253 | 0.06057 | -16.8 | 0.05175 | +71 |
| 2^20–2^28 | 2^20 | 0.05888 | 0.03340 | +253 | 0.06057 | -16.8 | 0.05175 | +71 |
| 2^20–2^28 | 2^24 | 0.14509 | 0.09708 | +319 | 0.15041 | -35.3 | 0.13437 | +71 |
| 2^20–2^28 | 2^28 | 0.26010 | 0.19174 | +365 | 0.26916 | -48.3 | 0.24686 | +71 |

Algebraic side. The raw excess (+99 to +365 standard errors) is removed by α: the α-adjusted gaps are +4.7, +1.9, -4.1, -16.8 standard errors at 2^14 to 2^20 and -16.8, -35.3, -48.3 at 2^20 to 2^28, that is -2.8% of the density at 2^20 and -3.4% at 2^28. The sign at the larger bounds is negative: the values are slightly less often smooth than ρ(u + α/ln B) predicts. Within the 2^28 bound the ratio observed over α-adjusted prediction is 0.96 to 0.97 in every u bin from 1.8 to 2.5, a flat residual, not a trend. The local-density form errs the other way (+4% to +6%). The count sits between the two textbook approximations.

Rational side. α = +0.5695 leaves a deficit of -6.1% at 2^20 (−111 standard errors); the local-density form overshoots by +6%. The Dickman approximation's own accuracy at u near 2 is the limit.

## Reading for D1

D1 claims an excess of smooth values over Dickman adjusted only by the measured root properties. On d1-01 there is no such excess at any bound from 2^14 to 2^28: after the harness's own α the residual is a deficit of a few percent at the larger bounds and within noise at the smaller ones. The rejection criterion compares the mean gap over the ten instances with twice the measurement noise; on this instance the gap after α is negative, so the claim is not supported here. What the run also shows is that the baseline's own error (ρ(u) against the local density, several percent at u between 2 and 3) is now larger than the Poisson noise, so the criterion's noise term has to include the baseline's uncertainty, or the baseline has to be sharpened, before a verdict across ten instances is meaningful.

## Open

- A second-order Dickman baseline (or an empirical calibration against random integers of the same sizes at the same bounds) to settle which approximation the criterion should use.
- d1-02 through d1-10 at both bound sets (running).
- The protocol's region per digit length, so a run can be labeled `full`.
