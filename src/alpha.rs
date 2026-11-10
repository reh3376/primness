//! Murphy α: the root-property correction to the Dickman prediction.
//!
//! The definition and its sources are recorded in docs/murphy_alpha.md.
//! α(F) = Σ_{p ≤ B} (1/(p − 1) − E_p(F)) ln p, where E_p(F) is the expected
//! p-adic valuation of F(a, b) over coprime pairs, computed exactly by
//! lifting roots mod p. Nothing here reads an instrument's score.

use crate::fp::{roots_mod_p, vanishes_mod_p};
use crate::sieve::primes_up_to;
use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{ToPrimitive, Zero};

pub const DEFAULT_BOUND: u64 = 2000;
pub const METHOD: &str = "murphy-expected-valuation";
const MAX_DEPTH: u32 = 24;

/// Reduce a coefficient to 0..p.
fn modp(c: &BigInt, p: u64) -> u64 {
    let r = c.mod_floor(&BigInt::from(p));
    r.to_u64().expect("residue fits")
}

/// f(x) mod p for x in 0..p, by Horner in u128.
fn eval_mod(coeffs_mod: &[u64], x: u64, p: u64) -> u64 {
    let mut acc: u128 = 0;
    for &c in coeffs_mod.iter().rev() {
        acc = (acc * x as u128 + c as u128) % p as u128;
    }
    acc as u64
}

/// Coefficients of f(r + p·x) as BigInt, lowest degree first.
fn shift_scale(f: &[BigInt], r: u64, p: u64) -> Vec<BigInt> {
    // Expand Σ c_i (r + p x)^i by repeated multiplication.
    let d = f.len() - 1;
    let mut out = vec![BigInt::zero(); d + 1];
    // (r + p x)^i computed incrementally as a polynomial in x.
    let mut power = vec![BigInt::from(1)];
    let r_big = BigInt::from(r);
    let p_big = BigInt::from(p);
    for (i, c) in f.iter().enumerate() {
        if i > 0 {
            // power *= (r + p x)
            let mut next = vec![BigInt::zero(); power.len() + 1];
            for (j, coeff) in power.iter().enumerate() {
                next[j] += coeff * &r_big;
                next[j + 1] += coeff * &p_big;
            }
            power = next;
        }
        for (j, coeff) in power.iter().enumerate() {
            out[j] += c * coeff;
        }
    }
    out
}

/// Divide out the largest power of p dividing every coefficient; returns the exponent.
fn remove_content(f: &mut [BigInt], p: u64) -> u32 {
    let p_big = BigInt::from(p);
    let mut k = 0u32;
    loop {
        if f.iter().all(|c| c.is_zero()) {
            return k;
        }
        if f.iter().all(|c| (c % &p_big).is_zero()) {
            for c in f.iter_mut() {
                *c /= &p_big;
            }
            k += 1;
        } else {
            return k;
        }
    }
}

/// The residues that can be roots of f mod p: its roots, found without
/// scanning every residue, or every residue when f vanishes mod p.
fn candidate_roots(f: &[BigInt], p: u64) -> Vec<u64> {
    if vanishes_mod_p(f, p) {
        (0..p).collect()
    } else {
        roots_mod_p(f, p)
    }
}

/// Expected p-adic valuation of f(x) for x uniform in Z_p.
pub fn affine_valuation(f: &[BigInt], p: u64) -> f64 {
    affine_depth(f, p, 0)
}

fn affine_depth(f: &[BigInt], p: u64, depth: u32) -> f64 {
    if f.iter().all(|c| c.is_zero()) || depth > MAX_DEPTH {
        return 0.0;
    }
    let coeffs_mod: Vec<u64> = f.iter().map(|c| modp(c, p)).collect();
    // f'(x) mod p: Σ i c_i x^{i-1}.
    let deriv_mod: Vec<u64> = (1..f.len())
        .map(|i| ((i as u128 * coeffs_mod[i] as u128) % p as u128) as u64)
        .collect();
    let mut total = 0.0;
    for r in candidate_roots(f, p) {
        if eval_mod(&coeffs_mod, r, p) != 0 {
            continue;
        }
        let simple = !deriv_mod.is_empty() && eval_mod(&deriv_mod, r, p) != 0;
        if simple {
            total += 1.0 / (p as f64 - 1.0);
        } else {
            let mut g = shift_scale(f, r, p);
            let k = remove_content(&mut g, p);
            total += (k as f64 + affine_depth(&g, p, depth + 1)) / p as f64;
        }
    }
    total
}

/// Expected p-adic valuation of f_rev(p·t) for t uniform in Z_p, the
/// contribution of the projective root (p dividing the leading coefficient).
pub fn projective_valuation(f: &[BigInt], p: u64) -> f64 {
    let mut rev: Vec<BigInt> = f.iter().rev().cloned().collect();
    if !(&rev[0] % BigInt::from(p)).is_zero() {
        return 0.0;
    }
    // h(t) = f_rev(p t): scale coefficient i by p^i, then remove content.
    let p_big = BigInt::from(p);
    let mut scale = BigInt::from(1);
    for c in rev.iter_mut() {
        *c *= &scale;
        scale *= &p_big;
    }
    let k = remove_content(&mut rev, p);
    k as f64 + affine_depth(&rev, p, 1)
}

/// (E[v], E[v²]) of v_p(f(x)) for x uniform in Z_p, by the same lifting as
/// `affine_valuation`. A simple root contributes a geometric valuation:
/// E[v·1] = 1/(p − 1), E[v²·1] = (p + 1)/(p − 1)². A repeated root r gives
/// v = k + V with V the valuation of the lifted polynomial, at weight 1/p.
pub fn affine_moments(f: &[BigInt], p: u64) -> (f64, f64) {
    affine_moments_depth(f, p, 0)
}

fn affine_moments_depth(f: &[BigInt], p: u64, depth: u32) -> (f64, f64) {
    if f.iter().all(|c| c.is_zero()) || depth > MAX_DEPTH {
        return (0.0, 0.0);
    }
    let pf = p as f64;
    let coeffs_mod: Vec<u64> = f.iter().map(|c| modp(c, p)).collect();
    let deriv_mod: Vec<u64> = (1..f.len())
        .map(|i| ((i as u128 * coeffs_mod[i] as u128) % p as u128) as u64)
        .collect();
    let (mut m1, mut m2) = (0.0, 0.0);
    for r in candidate_roots(f, p) {
        if eval_mod(&coeffs_mod, r, p) != 0 {
            continue;
        }
        let simple = !deriv_mod.is_empty() && eval_mod(&deriv_mod, r, p) != 0;
        if simple {
            m1 += 1.0 / (pf - 1.0);
            m2 += (pf + 1.0) / ((pf - 1.0) * (pf - 1.0));
        } else {
            let mut g = shift_scale(f, r, p);
            let k = remove_content(&mut g, p) as f64;
            let (e1, e2) = affine_moments_depth(&g, p, depth + 1);
            m1 += (k + e1) / pf;
            m2 += (k * k + 2.0 * k * e1 + e2) / pf;
        }
    }
    (m1, m2)
}

/// (E[v], E[v²]) of the projective contribution, as `projective_valuation`.
pub fn projective_moments(f: &[BigInt], p: u64) -> (f64, f64) {
    let mut rev: Vec<BigInt> = f.iter().rev().cloned().collect();
    if !(&rev[0] % BigInt::from(p)).is_zero() {
        return (0.0, 0.0);
    }
    let p_big = BigInt::from(p);
    let mut scale = BigInt::from(1);
    for c in rev.iter_mut() {
        *c *= &scale;
        scale *= &p_big;
    }
    let k = remove_content(&mut rev, p) as f64;
    let (e1, e2) = affine_moments_depth(&rev, p, 1);
    (k + e1, k * k + 2.0 * k * e1 + e2)
}

/// (E[v], E[v²]) over coprime pairs: the affine part at weight p/(p + 1)
/// and the projective part at weight 1/(p + 1).
pub fn valuation_moments(f: &[BigInt], p: u64) -> (f64, f64) {
    let pf = p as f64;
    let (a1, a2) = affine_moments(f, p);
    let (q1, q2) = projective_moments(f, p);
    (pf / (pf + 1.0) * a1 + q1 / (pf + 1.0), pf / (pf + 1.0) * a2 + q2 / (pf + 1.0))
}

/// Variance of v_p(n) for a random integer: E[v] = 1/(p − 1), E[v²] = (p + 1)/(p − 1)².
pub fn random_valuation_variance(p: u64) -> f64 {
    let pf = p as f64;
    pf / ((pf - 1.0) * (pf - 1.0))
}

/// α₂(F) = Σ_{p ≤ bound} (Var[v_p(F)] − Var[v_p(n)]) ln² p: the excess
/// variance of the log of the small-prime part of F over coprime pairs,
/// relative to a random integer, with the primes taken as independent as in
/// α. α fixes the mean of that log; this is its second moment
/// (docs/baseline_calibration.md).
pub fn alpha2(f: &[BigInt], bound: u64) -> f64 {
    let mut f: Vec<BigInt> = f.to_vec();
    while f.len() > 1 && f.last().is_some_and(Zero::is_zero) {
        f.pop();
    }
    primes_up_to(bound)
        .into_iter()
        .map(|p| {
            let (m1, m2) = valuation_moments(&f, p);
            let ln = (p as f64).ln();
            (m2 - m1 * m1 - random_valuation_variance(p)) * ln * ln
        })
        .sum()
}

/// E_p(F): expected valuation over coprime pairs.
pub fn expected_valuation(f: &[BigInt], p: u64) -> f64 {
    let pf = p as f64;
    pf / (pf + 1.0) * affine_valuation(f, p) + 1.0 / (pf + 1.0) * projective_valuation(f, p)
}

/// α(F) over primes p ≤ bound.
pub fn alpha(f: &[BigInt], bound: u64) -> f64 {
    let mut f: Vec<BigInt> = f.to_vec();
    while f.len() > 1 && f.last().is_some_and(Zero::is_zero) {
        f.pop();
    }
    primes_up_to(bound)
        .into_iter()
        .map(|p| {
            let pf = p as f64;
            (1.0 / (pf - 1.0) - expected_valuation(&f, p)) * pf.ln()
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn poly(c: &[i64]) -> Vec<BigInt> {
        c.iter().map(|&x| BigInt::from(x)).collect()
    }

    #[test]
    fn simple_roots_follow_murphys_form() {
        // x^2 + 1: no roots mod 3, two simple roots mod 5.
        let f = poly(&[1, 0, 1]);
        assert_eq!(affine_valuation(&f, 3), 0.0);
        assert!((affine_valuation(&f, 5) - 2.0 / 4.0).abs() < 1e-12);
        assert!((expected_valuation(&f, 5) - 2.0 * 5.0 / 24.0).abs() < 1e-12);
        assert_eq!(projective_valuation(&f, 5), 0.0);
    }

    #[test]
    fn repeated_root_lifts_to_the_exact_expectation() {
        // E[v_p(x^2)] = 2 E[v_p(x)] = 2/(p-1).
        let f = poly(&[0, 0, 1]);
        for p in [2u64, 3, 7] {
            assert!((affine_valuation(&f, p) - 2.0 / (p as f64 - 1.0)).abs() < 1e-6, "p={p}");
        }
    }

    #[test]
    fn projective_root_of_a_linear_form() {
        // G = 2a + b: over coprime pairs mod 2, E[v_2] = 2/3.
        let g = poly(&[1, 2]);
        assert_eq!(affine_valuation(&g, 2), 0.0);
        assert!((projective_valuation(&g, 2) - 2.0).abs() < 1e-9);
        assert!((expected_valuation(&g, 2) - 2.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn alpha_of_x_is_the_small_positive_constant() {
        // α(x) with bound 2: (1 − 2/3) ln 2.
        let f = poly(&[0, 1]);
        assert!((alpha(&f, 2) - (1.0 - 2.0 / 3.0) * 2f64.ln()).abs() < 1e-12);
        let a = alpha(&f, DEFAULT_BOUND);
        assert!(a > 0.5 && a < 0.6, "{a}");
    }

    #[test]
    fn moments_agree_with_the_expectation_and_with_the_square_identity() {
        let f = poly(&[-85186479, -38330143, 45773675, 567699, 7560]);
        for p in [2u64, 3, 5, 7, 11, 13, 101] {
            let (m1, _) = valuation_moments(&f, p);
            assert!((m1 - expected_valuation(&f, p)).abs() < 1e-12, "p={p}");
        }
        // v_p(x²) = 2 v_p(x): E[v] = 2/(p − 1), E[v²] = 4 (p + 1)/(p − 1)².
        let sq = poly(&[0, 0, 1]);
        for p in [2u64, 3, 7] {
            let pf = p as f64;
            let (m1, m2) = affine_moments(&sq, p);
            assert!((m1 - 2.0 / (pf - 1.0)).abs() < 1e-6, "p={p}");
            assert!((m2 - 4.0 * (pf + 1.0) / ((pf - 1.0) * (pf - 1.0))).abs() < 1e-5, "p={p} m2={m2}");
        }
        // One simple root at every prime has the random integer's variance.
        let x = poly(&[0, 1]);
        for p in [3u64, 5, 7] {
            let (m1, m2) = affine_moments(&x, p);
            assert!((m2 - m1 * m1 - random_valuation_variance(p)).abs() < 1e-12, "p={p}");
        }
    }

    #[test]
    fn alpha2_is_positive_for_a_polynomial_rich_in_roots() {
        // (x-1)(x-2)(x-3)(x-4): four roots mod every prime above 4, so the
        // small-prime part varies more than a random integer's.
        let f = poly(&[24, -50, 35, -10, 1]);
        assert!(alpha2(&f, 200) > 3.0, "{}", alpha2(&f, 200));
        // x² + 1 has no roots mod 3 or 7: less variance than random there.
        let g = poly(&[1, 0, 1]);
        let (m1, m2) = valuation_moments(&g, 3);
        assert!(m2 - m1 * m1 < random_valuation_variance(3));
    }

    #[test]
    fn alpha_is_negative_for_a_polynomial_rich_in_roots() {
        // (x-1)(x-2)(x-3)(x-4) has four roots mod every prime above 4.
        let f = poly(&[24, -50, 35, -10, 1]);
        let a = alpha(&f, 200);
        assert!(a < -3.0, "{a}");
    }
}
