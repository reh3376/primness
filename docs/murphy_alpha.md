# Murphy α: the root-property correction

**Definition recorded** 2026-11-03 so that the harness can compute it. The sources are Murphy, *Polynomial Selection for the Number Field Sieve Integer Factorisation Algorithm*, PhD thesis, Australian National University, 1999, §3.2 (root properties), and Bai, Bouvier, Kruppa, Zimmermann, *Better polynomials for GNFS*, Mathematics of Computation 85 (2016), §2, which gives the exact expected valuation used by CADO-NFS (`polyselect/auxiliary.c`). Nothing from CADO's code is copied; the harness implements the formula below from this page.

## Statement

For a homogeneous polynomial F(a, b) of degree d with integer coefficients, f(x) = F(x, 1), and a prime bound B_α:

α(F) = Σ over primes p ≤ B_α of ( 1/(p − 1) − E_p(F) ) · ln p

where E_p(F) is the expected p-adic valuation of F(a, b) for (a, b) drawn uniformly from coprime pairs, and 1/(p − 1) is the expected p-adic valuation of a random integer. A negative α means F's values carry more small-prime content than random integers of their size, so they are smoother. The correction enters the Dickman prediction as a size shift: the smoothness probability of a value V is taken as ρ((ln|V| + α) / ln B) instead of ρ(ln|V| / ln B) (Murphy §3.2.3, §5.2).

## Expected valuation over coprime pairs

Write a coprime pair mod p. With probability p/(p + 1) the prime does not divide b, a/b is uniform in Z_p, and v_p(F(a, b)) = v_p(f(x)) for uniform x. With probability 1/(p + 1) the prime divides b, b/a lies in pZ_p, and v_p(F(a, b)) = v_p(f_rev(p·t)) for uniform t, where f_rev(x) = x^d f(1/x) is the reversed polynomial. So

E_p(F) = p/(p + 1) · A_p(f) + 1/(p + 1) · P_p(f),

A_p(f) = E over uniform x in Z_p of v_p(f(x)), and P_p(f) = E over uniform t in Z_p of v_p(f_rev(p·t)).

A_p is computed exactly by lifting (Bai et al. §2; the same recursion CADO uses): for each residue r in 0..p−1 with f(r) ≡ 0 (mod p), if f'(r) ≢ 0 (mod p) the root is simple and contributes 1/(p − 1); otherwise let g(x) = f(r + p·x) / p^k with p^k the largest prime power dividing every coefficient (k ≥ 1), and the residue contributes (k + A_p(g)) / p. P_p is the same lift applied once to f_rev at the residue 0: if f_rev(0) = c_d ≢ 0 (mod p) it is 0; otherwise P_p = k + A_p(h) with h(t) = f_rev(p·t) / p^k. The recursion ends when no multiple root remains; a polynomial with a repeated p-adic root would recurse forever, so the harness stops at depth 24, where the remaining contribution is below p^−24.

For a prime that divides neither the discriminant of f nor its leading coefficient this reduces to Murphy's simple form: A_p = n_p / (p − 1) with n_p the number of roots of f mod p, P_p = 0, and E_p = n_p · p / (p² − 1).

## Bound and conventions

B_α = 2000 by default, the value CADO-NFS uses for ranking; it is recorded in every result. The harness computes α for the algebraic polynomial and for the rational linear polynomial G(a, b) = Y1·a + Y0·b with the same formula (for the linear case E_p = p/(p² − 1) when p ∤ Y1, so α(G) is a small positive constant, about +0.57 at B_α = 2000). The instrument's own α estimate is never read; the comment line in `cado.poly` stays a comment.

## What α does not do

It corrects the size of the values for their small-prime content; it does not change the distribution of large prime factors. A residual gap between the α-adjusted Dickman prediction and the count is what the D1 claim is about. The rejection criterion in `docs/d1_falsifiable_prediction.md` applies to that residual.

α is the first moment of the log small-prime part. Its second moment, the excess variance α₂ = Σ (Var[v_p(F)] − Var[v_p(n)]) ln² p, is computed by the same lifting (`valuation_moments`, `alpha2` in `src/alpha.rs`, binary `d1-alpha`) and enters the prediction as the convexity term α₂ ρ″/(2ρ ln² B). `docs/baseline_calibration.md` derives it, measures the baseline it corrects, and shows it accounts for most of the residual after α on the ten instances.
