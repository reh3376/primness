//! Polynomials over F_p for primes p < 2^32: the roots of f mod p, which a
//! sieve needs for every prime in its factor base. Roots are found by the
//! textbook route: gcd(x^p − x, f) collects the distinct linear factors,
//! and equal-degree splitting with gcd((x + a)^((p−1)/2) − 1, g) separates
//! them (Cantor–Zassenhaus). Small primes are brute-forced.

use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::ToPrimitive;

const BRUTE_FORCE_BELOW: u64 = 64;

#[inline]
fn mulmod(a: u64, b: u64, p: u64) -> u64 {
    ((a as u128 * b as u128) % p as u128) as u64
}

fn powmod(mut a: u64, mut e: u64, p: u64) -> u64 {
    let mut r = 1 % p;
    a %= p;
    while e > 0 {
        if e & 1 == 1 {
            r = mulmod(r, a, p);
        }
        a = mulmod(a, a, p);
        e >>= 1;
    }
    r
}

fn inv(a: u64, p: u64) -> u64 {
    powmod(a, p - 2, p)
}

/// Coefficients of f reduced into 0..p, trailing zeros removed.
pub fn reduce(f: &[BigInt], p: u64) -> Vec<u64> {
    let p_big = BigInt::from(p);
    let mut out: Vec<u64> = f
        .iter()
        .map(|c| c.mod_floor(&p_big).to_u64().expect("residue fits"))
        .collect();
    trim(&mut out);
    out
}

fn trim(f: &mut Vec<u64>) {
    while f.last() == Some(&0) {
        f.pop();
    }
}

fn eval(f: &[u64], x: u64, p: u64) -> u64 {
    let mut acc = 0u64;
    for &c in f.iter().rev() {
        acc = (mulmod(acc, x, p) + c) % p;
    }
    acc
}

/// a mod m over F_p (m non-empty).
fn rem(a: &[u64], m: &[u64], p: u64) -> Vec<u64> {
    let mut r = a.to_vec();
    trim(&mut r);
    let dm = m.len() - 1;
    let lead_inv = inv(m[dm], p);
    while r.len() > dm {
        let dr = r.len() - 1;
        let q = mulmod(r[dr], lead_inv, p);
        if q != 0 {
            for i in 0..=dm {
                let sub = mulmod(q, m[i], p);
                r[dr - dm + i] = (r[dr - dm + i] + p - sub) % p;
            }
        }
        r.pop();
        trim(&mut r);
    }
    r
}

fn mul(a: &[u64], b: &[u64], p: u64) -> Vec<u64> {
    if a.is_empty() || b.is_empty() {
        return Vec::new();
    }
    let mut out = vec![0u64; a.len() + b.len() - 1];
    for (i, &x) in a.iter().enumerate() {
        if x == 0 {
            continue;
        }
        for (j, &y) in b.iter().enumerate() {
            out[i + j] = (out[i + j] + mulmod(x, y, p)) % p;
        }
    }
    trim(&mut out);
    out
}

fn mulmod_poly(a: &[u64], b: &[u64], m: &[u64], p: u64) -> Vec<u64> {
    rem(&mul(a, b, p), m, p)
}

/// base^e mod m.
fn powmod_poly(base: &[u64], mut e: u64, m: &[u64], p: u64) -> Vec<u64> {
    let mut result = rem(&[1], m, p);
    let mut b = rem(base, m, p);
    while e > 0 {
        if e & 1 == 1 {
            result = mulmod_poly(&result, &b, m, p);
        }
        e >>= 1;
        if e > 0 {
            b = mulmod_poly(&b, &b, m, p);
        }
    }
    result
}

fn sub(a: &[u64], b: &[u64], p: u64) -> Vec<u64> {
    let mut out = vec![0u64; a.len().max(b.len())];
    for (i, slot) in out.iter_mut().enumerate() {
        let x = a.get(i).copied().unwrap_or(0);
        let y = b.get(i).copied().unwrap_or(0);
        *slot = (x + p - y) % p;
    }
    trim(&mut out);
    out
}

fn monic(mut f: Vec<u64>, p: u64) -> Vec<u64> {
    if let Some(&lead) = f.last() {
        if lead != 1 {
            let li = inv(lead, p);
            for c in f.iter_mut() {
                *c = mulmod(*c, li, p);
            }
        }
    }
    f
}

fn gcd(a: &[u64], b: &[u64], p: u64) -> Vec<u64> {
    let mut x = a.to_vec();
    let mut y = b.to_vec();
    trim(&mut x);
    trim(&mut y);
    while !y.is_empty() {
        let r = rem(&x, &y, p);
        x = y;
        y = r;
    }
    monic(x, p)
}

fn divide_exact(a: &[u64], b: &[u64], p: u64) -> Vec<u64> {
    // a / b assuming b | a.
    let mut r = a.to_vec();
    trim(&mut r);
    let db = b.len() - 1;
    let lead_inv = inv(b[db], p);
    let mut q = vec![0u64; r.len().saturating_sub(db)];
    while r.len() > db {
        let dr = r.len() - 1;
        let c = mulmod(r[dr], lead_inv, p);
        q[dr - db] = c;
        for i in 0..=db {
            let s = mulmod(c, b[i], p);
            r[dr - db + i] = (r[dr - db + i] + p - s) % p;
        }
        r.pop();
        trim(&mut r);
    }
    q
}

/// Roots in 0..p of a product of distinct monic linear factors g.
fn split(g: &[u64], p: u64, seed: &mut u64, out: &mut Vec<u64>) {
    let deg = g.len() - 1;
    if deg == 0 {
        return;
    }
    if deg == 1 {
        out.push((p - g[0]) % p);
        return;
    }
    let half = (p - 1) / 2;
    loop {
        *seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
        let shift = (*seed >> 33) % p;
        // h = gcd((x + shift)^((p-1)/2) - 1, g)
        let base = [shift, 1];
        let powered = powmod_poly(&base, half, g, p);
        let minus_one = sub(&powered, &[1], p);
        let h = gcd(&minus_one, g, p);
        let dh = h.len().saturating_sub(1);
        if dh > 0 && dh < deg {
            let other = divide_exact(g, &h, p);
            split(&h, p, seed, out);
            split(&monic(other, p), p, seed, out);
            return;
        }
    }
}

/// The distinct roots of f mod p, ascending. f is given over the integers.
pub fn roots_mod_p(f: &[BigInt], p: u64) -> Vec<u64> {
    let fp = reduce(f, p);
    if fp.len() <= 1 {
        // Constant (or zero): zero polynomial is the only case with roots, and
        // a zero polynomial mod p means every residue; the caller treats a
        // polynomial vanishing mod p separately.
        return Vec::new();
    }
    if p < BRUTE_FORCE_BELOW {
        let mut out: Vec<u64> = (0..p).filter(|&x| eval(&fp, x, p) == 0).collect();
        out.sort_unstable();
        return out;
    }
    if fp.len() == 2 {
        return vec![mulmod((p - fp[0]) % p, inv(fp[1], p), p)];
    }
    let fm = monic(fp, p);
    // x^p mod f, then gcd(x^p - x, f)
    let xp = powmod_poly(&[0, 1], p, &fm, p);
    let xp_minus_x = sub(&xp, &[0, 1], p);
    let g = gcd(&xp_minus_x, &fm, p);
    if g.len() <= 1 {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut seed = p ^ 0x9e37_79b9_7f4a_7c15;
    split(&g, p, &mut seed, &mut out);
    out.sort_unstable();
    out.dedup();
    out
}

/// True when every coefficient of f is divisible by p.
pub fn vanishes_mod_p(f: &[BigInt], p: u64) -> bool {
    reduce(f, p).is_empty()
}

/// The largest degree `x_pow_p_word` handles.
const WORD_DEGREE: usize = 8;

/// x^p mod m over F_p, for a monic m of degree 2..=WORD_DEGREE and p < 2^32:
/// the value `powmod_poly(&[0, 1], p, m, p)` gives, computed in fixed arrays
/// with every product in one machine word.
fn x_pow_p_word(m: &[u64], p: u64) -> Vec<u64> {
    let d = m.len() - 1;
    debug_assert!((2..=WORD_DEGREE).contains(&d) && p < 1 << 32);
    // x^d ≡ Σ_{i<d} neg[i] x^i (mod m).
    let mut neg = [0u64; WORD_DEGREE];
    for i in 0..d {
        neg[i] = (p - m[i]) % p;
    }
    // Left to right over the bits of p; the top bit gives x itself.
    let mut r = [0u64; WORD_DEGREE];
    r[1] = 1;
    let top = 63 - p.leading_zeros();
    for bit in (0..top).rev() {
        let mut t = [0u64; 2 * WORD_DEGREE - 1];
        for i in 0..d {
            if r[i] == 0 {
                continue;
            }
            for j in 0..d {
                t[i + j] = (t[i + j] + r[i] * r[j] % p) % p;
            }
        }
        // Products of two residues below 2^32 and a residue added stay below 2^64.
        for k in (d..=2 * d - 2).rev() {
            let c = t[k];
            if c != 0 {
                for i in 0..d {
                    t[k - d + i] = (t[k - d + i] + c * neg[i]) % p;
                }
            }
        }
        r[..d].copy_from_slice(&t[..d]);
        if (p >> bit) & 1 == 1 {
            let c = r[d - 1];
            for i in (1..d).rev() {
                r[i] = r[i - 1];
            }
            r[0] = 0;
            if c != 0 {
                for i in 0..d {
                    r[i] = (r[i] + c * neg[i]) % p;
                }
            }
        }
    }
    let mut out = r[..d].to_vec();
    trim(&mut out);
    out
}

/// The number of distinct roots in F_p of f, given reduced mod p (`reduce`):
/// the degree of gcd(x^p − x, f). It equals `roots_mod_p(f, p).len()` without
/// separating the roots, and for p < 2^32 and degree up to 8 it raises x to
/// the p-th power in fixed arrays, fast enough for every prime up to 2^28
/// (`d1-tail`). A constant, the zero polynomial included, has none.
pub fn count_roots(fp: &[u64], p: u64) -> usize {
    let mut f = fp.to_vec();
    trim(&mut f);
    if f.len() <= 1 {
        return 0;
    }
    if f.len() == 2 {
        return 1;
    }
    if p < BRUTE_FORCE_BELOW {
        return (0..p).filter(|&x| eval(&f, x, p) == 0).count();
    }
    let fm = monic(f, p);
    let xp = if p < 1 << 32 && fm.len() - 1 <= WORD_DEGREE {
        x_pow_p_word(&fm, p)
    } else {
        powmod_poly(&[0, 1], p, &fm, p)
    };
    gcd(&sub(&xp, &[0, 1], p), &fm, p).len() - 1
}

fn derivative(f: &[u64], p: u64) -> Vec<u64> {
    let mut out: Vec<u64> = f.iter().enumerate().skip(1).map(|(i, &c)| mulmod(c, i as u64 % p, p)).collect();
    trim(&mut out);
    out
}

/// The degrees of the irreducible factors of f mod p, ascending, by
/// distinct-degree factorization. None when p divides the leading
/// coefficient or f mod p has a repeated factor: the pattern then says
/// nothing about how f factors over Q.
pub fn factor_degrees(f: &[BigInt], p: u64) -> Option<Vec<usize>> {
    let mut f = f.to_vec();
    while f.len() > 1 && f.last().is_some_and(num_traits::Zero::is_zero) {
        f.pop();
    }
    let degree = f.len() - 1;
    let fp = reduce(&f, p);
    if degree < 1 || fp.len() != degree + 1 {
        return None;
    }
    let mut rest = monic(fp, p);
    if gcd(&rest, &derivative(&rest, p), p).len() != 1 {
        return None;
    }
    let mut out = Vec::new();
    let mut h = vec![0, 1];
    let mut d = 1;
    while rest.len() - 1 >= 2 * d {
        // h = x^(p^d) mod rest; gcd(h − x, rest) is the product of the factors of degree d.
        h = powmod_poly(&h, p, &rest, p);
        let g = gcd(&sub(&h, &[0, 1], p), &rest, p);
        let dg = g.len() - 1;
        if dg > 0 {
            out.extend(std::iter::repeat(d).take(dg / d));
            rest = monic(divide_exact(&rest, &g, p), p);
            h = rem(&h, &rest, p);
        }
        d += 1;
    }
    if rest.len() > 1 {
        out.push(rest.len() - 1);
    }
    Some(out)
}

/// A certificate that f is irreducible over Q, from the factor degrees of f
/// modulo the primes up to `max_p`. A factor of f over Q of degree k would
/// reduce, modulo every prime that keeps f's degree and squarefreeness, to a
/// product of some of the factors there, so k would be a sum of some of
/// their degrees at every such prime. The certificate is the list of primes
/// (with their degrees) that leave no proper k; None when the primes up to
/// `max_p` do not suffice (x^4 + 1, irreducible but split into quadratics at
/// every prime, is never certified).
pub fn certify_irreducible(f: &[BigInt], max_p: u64) -> Option<Vec<(u64, Vec<usize>)>> {
    let degree = degree_of(f);
    if degree < 1 || degree >= 128 {
        return None;
    }
    if degree == 1 {
        return Some(Vec::new());
    }
    // Bit k set: a factor of degree k is still possible.
    let mut possible: u128 = ((1u128 << degree) - 1) & !1;
    let mut used = Vec::new();
    for p in crate::sieve::primes_up_to(max_p) {
        let Some(degrees) = factor_degrees(f, p) else { continue };
        let mut sums: u128 = 1;
        for &k in &degrees {
            sums |= sums << k;
        }
        if possible & !sums != 0 {
            possible &= sums;
            used.push((p, degrees));
            if possible == 0 {
                return Some(used);
            }
        }
    }
    None
}

fn degree_of(f: &[BigInt]) -> usize {
    let mut d = f.len().saturating_sub(1);
    while d > 0 && num_traits::Zero::is_zero(&f[d]) {
        d -= 1;
    }
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    fn poly(c: &[i64]) -> Vec<BigInt> {
        c.iter().map(|&x| BigInt::from(x)).collect()
    }

    fn brute(f: &[BigInt], p: u64) -> Vec<u64> {
        let fp = reduce(f, p);
        (0..p).filter(|&x| eval(&fp, x, p) == 0).collect()
    }

    #[test]
    fn matches_brute_force_on_small_and_medium_primes() {
        let polys = [
            poly(&[24, -50, 35, -10, 1]),
            poly(&[-1, 0, 0, 0, 1]),
            poly(&[1, 0, 1]),
            poly(&[-85186479, -38330143, 45773675, 567699, 7560]),
            poly(&[197150570, 63638117]),
            poly(&[0, 0, 1]),
            poly(&[7, 0, 0, 0, 0, 3]),
        ];
        let primes = crate::sieve::primes_up_to(3000);
        for f in &polys {
            for &p in primes.iter().filter(|&&p| p >= 2) {
                let expected = brute(f, p);
                let got = roots_mod_p(f, p);
                assert_eq!(got, expected, "f={f:?} p={p}");
            }
        }
    }

    #[test]
    fn large_prime_roots_are_roots() {
        let f = poly(&[-85186479, -38330143, 45773675, 567699, 7560]);
        for &p in &[1_000_003u64, 2_147_483_647, 4_294_967_291] {
            let fp = reduce(&f, p);
            for r in roots_mod_p(&f, p) {
                assert_eq!(eval(&fp, r, p), 0, "p={p} r={r}");
            }
        }
        // x^4 - 1 has four roots mod p when p ≡ 1 (mod 4).
        let g = poly(&[-1, 0, 0, 0, 1]);
        assert_eq!(roots_mod_p(&g, 1_000_000_009).len(), 4);
        assert_eq!(roots_mod_p(&g, 1_000_000_007).len(), 2);
    }

    #[test]
    fn word_powering_matches_the_general_one() {
        let primes = crate::sieve::primes_up_to(1 << 20);
        let mut seed = 12_345u64;
        for &p in primes.iter().filter(|&&p| p >= BRUTE_FORCE_BELOW).step_by(997).chain([4_294_967_291u64].iter()) {
            for d in 2..=WORD_DEGREE {
                let mut m: Vec<u64> = (0..d)
                    .map(|_| {
                        seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
                        (seed >> 33) % p
                    })
                    .collect();
                m.push(1);
                assert_eq!(x_pow_p_word(&m, p), powmod_poly(&[0, 1], p, &m, p), "p={p} m={m:?}");
            }
        }
    }

    #[test]
    fn counted_roots_match_the_roots_found() {
        let polys = [
            poly(&[24, -50, 35, -10, 1]),
            poly(&[-1, 0, 0, 0, 1]),
            poly(&[1, 0, 1]),
            poly(&[-85186479, -38330143, 45773675, 567699, 7560]),
            poly(&[197150570, 63638117]),
            poly(&[0, 0, 1]),
            poly(&[7, 0, 0, 0, 0, 3]),
            poly(&[3, 1, 4, 1, 5, 9, 2, 6, 5, 3]),
        ];
        let primes = crate::sieve::primes_up_to(20_000);
        for f in &polys {
            for &p in primes.iter().chain([1_000_003u64, 268_435_399, 4_294_967_291].iter()) {
                assert_eq!(count_roots(&reduce(f, p), p), roots_mod_p(f, p).len(), "f={f:?} p={p}");
            }
        }
        assert_eq!(count_roots(&[], 7), 0);
        assert_eq!(count_roots(&[5], 7), 0);
    }

    #[test]
    fn factor_degrees_are_the_factorization_mod_p() {
        // x^4 − 1 = (x − 1)(x + 1)(x^2 + 1): four linear factors when p ≡ 1 (mod 4).
        let f = poly(&[-1, 0, 0, 0, 1]);
        assert_eq!(factor_degrees(&f, 5), Some(vec![1, 1, 1, 1]));
        assert_eq!(factor_degrees(&f, 7), Some(vec![1, 1, 2]));
        assert_eq!(factor_degrees(&f, 2), None, "repeated factor mod 2");
        assert_eq!(factor_degrees(&poly(&[1, 0, 1]), 3), Some(vec![2]));
        assert_eq!(factor_degrees(&poly(&[1, 0, 3]), 3), None, "p divides the leading coefficient");
        assert_eq!(factor_degrees(&poly(&[1, -2, 1]), 101), None, "(x − 1)^2");
        // x^4 − 2 is irreducible mod 5 (2 is not a square mod 5, and so not a fourth power).
        assert_eq!(factor_degrees(&poly(&[-2, 0, 0, 0, 1]), 5), Some(vec![4]));
        // The degrees sum to the degree, and the linear ones are the roots.
        let polys = [
            poly(&[-85186479, -38330143, 45773675, 567699, 7560]),
            poly(&[3, 1, 4, 1, 5, 9, 2, 6, 5, 3]),
            poly(&[24, -50, 35, -10, 1]),
        ];
        for f in &polys {
            for p in crate::sieve::primes_up_to(3000) {
                if let Some(d) = factor_degrees(f, p) {
                    assert_eq!(d.iter().sum::<usize>(), f.len() - 1, "f={f:?} p={p}");
                    assert_eq!(d.iter().filter(|&&k| k == 1).count(), roots_mod_p(f, p).len(), "f={f:?} p={p}");
                }
            }
        }
    }

    #[test]
    fn irreducibility_is_certified_only_when_true() {
        // Irreducible, with primes that keep it whole.
        assert!(certify_irreducible(&poly(&[-2, 0, 0, 0, 1]), 10_000).is_some());
        assert!(certify_irreducible(&poly(&[-85186479, -38330143, 45773675, 567699, 7560]), 10_000).is_some());
        assert!(certify_irreducible(&poly(&[3, 1, 4, 1, 5, 9, 2, 6, 5, 3]), 10_000).is_some());
        // Reducible: never certified.
        assert!(certify_irreducible(&poly(&[3, 0, 4, 0, 1]), 10_000).is_none(), "(x^2 + 1)(x^2 + 3)");
        assert!(certify_irreducible(&poly(&[-1, 0, 1, -1, 1]), 10_000).is_none(), "(x − 1)(x^3 + x + 1)");
        assert!(certify_irreducible(&poly(&[-2, -2, -1, 1, 1]), 10_000).is_none(), "(x^2 + x + 1)(x^2 − 2)");
        assert!(certify_irreducible(&poly(&[-1, 0, 1, 0, 0]), 10_000).is_none(), "(x − 1)(x + 1), trailing zeros");
        // Irreducible but split into quadratics at every prime: not certified.
        assert!(certify_irreducible(&poly(&[1, 0, 0, 0, 1]), 10_000).is_none(), "x^4 + 1");
    }
}
