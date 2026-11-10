//! Line sieve over the rectangle a in [−A, A], one row b at a time.
//!
//! For every prime p ≤ B in the factor base and every root r of f mod p,
//! the positions a ≡ r·b (mod p) get ln p added; when p divides both b
//! and the leading coefficient, every position does. A position whose sum
//! reaches ln|F(a, b)| minus a slack is a candidate; a second pass
//! (resieve) collects the primes that hit each candidate, and dividing
//! them out fully decides smoothness exactly and gives the largest prime
//! factor. The one approximation: a value whose repeated prime powers
//! exceed the slack (e^slack = B² by default) is not a candidate and is
//! counted as not smooth. docs/line_sieve.md records this.

use crate::fp::{roots_mod_p, vanishes_mod_p};
use crate::sieve::primes_up_to;
use num_bigint::{BigInt, BigUint};
use num_traits::Zero;

pub const LOG_SCALE: f64 = 64.0;

pub struct FactorBase {
    pub bound: u64,
    pub primes: Vec<u32>,
    pub logs: Vec<u16>,
    root_start: Vec<u32>,
    roots: Vec<u32>,
    projective: Vec<bool>,
}

impl FactorBase {
    /// Primes up to `bound` with the roots of f mod p, computed on `threads` threads.
    pub fn build(f: &[BigInt], bound: u64, threads: usize) -> Self {
        let primes64 = primes_up_to(bound);
        let lead = f.iter().rev().find(|c| !c.is_zero()).cloned().unwrap_or_else(BigInt::zero);
        let threads = threads.max(1);
        let chunk = primes64.len().div_ceil(threads).max(1);
        let parts: Vec<(Vec<u32>, Vec<u16>, Vec<u32>, Vec<u32>, Vec<bool>)> = std::thread::scope(|scope| {
            let handles: Vec<_> = primes64
                .chunks(chunk)
                .map(|ps| {
                    let (f, lead) = (f, &lead);
                    scope.spawn(move || {
                        let mut primes = Vec::with_capacity(ps.len());
                        let mut logs = Vec::with_capacity(ps.len());
                        let mut starts = Vec::with_capacity(ps.len());
                        let mut roots = Vec::new();
                        let mut projective = Vec::with_capacity(ps.len());
                        for &p in ps {
                            primes.push(p as u32);
                            logs.push(((p as f64).ln() * LOG_SCALE).round() as u16);
                            starts.push(roots.len() as u32);
                            if vanishes_mod_p(f, p) {
                                // Every residue is a root: p divides the content.
                                roots.extend(0..p as u32);
                            } else {
                                roots.extend(roots_mod_p(f, p).into_iter().map(|r| r as u32));
                            }
                            projective.push((lead % BigInt::from(p)).is_zero());
                        }
                        (primes, logs, starts, roots, projective)
                    })
                })
                .collect();
            handles.into_iter().map(|h| h.join().expect("factor base thread")).collect()
        });
        let mut fb = FactorBase {
            bound,
            primes: Vec::new(),
            logs: Vec::new(),
            root_start: Vec::new(),
            roots: Vec::new(),
            projective: Vec::new(),
        };
        for (primes, logs, starts, roots, projective) in parts {
            let offset = fb.roots.len() as u32;
            fb.primes.extend(primes);
            fb.logs.extend(logs);
            fb.root_start.extend(starts.into_iter().map(|s| s + offset));
            fb.roots.extend(roots);
            fb.projective.extend(projective);
        }
        fb.root_start.push(fb.roots.len() as u32);
        fb
    }

    pub fn len(&self) -> usize {
        self.primes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.primes.is_empty()
    }

    pub fn total_roots(&self) -> usize {
        self.roots.len()
    }

    #[inline]
    fn roots_of(&self, i: usize) -> &[u32] {
        &self.roots[self.root_start[i] as usize..self.root_start[i + 1] as usize]
    }
}

/// Walk the hits of every prime on row b over the positions a = a_lo + j,
/// j in [0, len); `visit(index, prime_index)` is called for each hit.
fn walk_range<F: FnMut(usize, usize)>(fb: &FactorBase, b: i64, a_lo: i64, len: usize, mut visit: F) {
    for i in 0..fb.len() {
        let p = fb.primes[i] as u64;
        let b_mod = b.rem_euclid(p as i64) as u64;
        if b_mod == 0 {
            if fb.projective[i] {
                for j in 0..len {
                    visit(j, i);
                }
            }
            continue;
        }
        // a ≡ r·b (mod p) and a = a_lo + j give j ≡ r·b − a_lo (mod p).
        let shift = (-a_lo).rem_euclid(p as i64) as u64;
        for &r in fb.roots_of(i) {
            let start = ((r as u64 * b_mod + shift) % p) as usize;
            let mut j = start;
            while j < len {
                visit(j, i);
                j += p as usize;
            }
        }
    }
}

/// Walk the hits of every prime on row b. `acc` has one slot per a in
/// [−a_max, a_max]; `visit(index, prime_index)` is called for each hit.
fn walk_row<F: FnMut(usize, usize)>(fb: &FactorBase, b: i64, a_max: i64, len: usize, visit: F) {
    walk_range(fb, b, -a_max, len, visit);
}

/// Add ln p (scaled) at every position p divides on row b.
pub fn sieve_row(fb: &FactorBase, b: i64, a_max: i64, acc: &mut [u16]) {
    let len = acc.len();
    walk_row(fb, b, a_max, len, |j, i| {
        acc[j] = acc[j].saturating_add(fb.logs[i]);
    });
}

/// The primes hitting each candidate position, as (position, prime) pairs sorted by position.
pub fn resieve_row(fb: &FactorBase, b: i64, a_max: i64, candidate: &[bool]) -> Vec<(u32, u32)> {
    let mut hits = Vec::new();
    walk_row(fb, b, a_max, candidate.len(), |j, i| {
        if candidate[j] {
            hits.push((j as u32, fb.primes[i]));
        }
    });
    hits.sort_unstable();
    hits
}

/// Add ln p (scaled) at every position p divides on row b, for the
/// positions a = a_lo + j with j in [0, acc.len()).
pub fn sieve_range(fb: &FactorBase, b: i64, a_lo: i64, acc: &mut [u16]) {
    let len = acc.len();
    walk_range(fb, b, a_lo, len, |j, i| {
        acc[j] = acc[j].saturating_add(fb.logs[i]);
    });
}

/// The primes hitting each candidate position of the range starting at
/// a_lo, as (position, prime) pairs sorted by position.
pub fn resieve_range(fb: &FactorBase, b: i64, a_lo: i64, candidate: &[bool]) -> Vec<(u32, u32)> {
    let mut hits = Vec::new();
    walk_range(fb, b, a_lo, candidate.len(), |j, i| {
        if candidate[j] {
            hits.push((j as u32, fb.primes[i]));
        }
    });
    hits.sort_unstable();
    hits
}

/// Divide the hit primes out of `value` fully. `Some(largest)` when the
/// cofactor is 1 (the value is B-smooth), `None` otherwise.
pub fn verify(value: &BigUint, primes: &[u32]) -> Option<u64> {
    let mut n = value.clone();
    let mut largest = 1u64;
    for &p in primes {
        let p64 = p as u64;
        if (&n % p64).is_zero() {
            largest = largest.max(p64);
            loop {
                n /= p64;
                if !(&n % p64).is_zero() {
                    break;
                }
            }
        }
    }
    if n == BigUint::from(1u32) {
        Some(largest)
    } else {
        None
    }
}

/// Slack below ln|F| for a position to be a candidate: 2 ln B plus rounding.
pub fn default_slack(bound: u64) -> f64 {
    2.0 * (bound as f64).ln() + 1.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::{coprime, Homogeneous};
    use crate::sieve::largest_prime_factor_if_smooth;

    fn poly(c: &[i64]) -> Vec<BigInt> {
        c.iter().map(|&x| BigInt::from(x)).collect()
    }

    #[test]
    fn sieve_agrees_with_trial_division_on_a_small_region() {
        let f = poly(&[-85186479, -38330143, 45773675, 567699, 7560]);
        let g = poly(&[197150570, 63638117]);
        let bound = 1u64 << 12;
        let primes = primes_up_to(bound);
        let a_max = 400i64;
        for (coeffs, name) in [(&f, "algebraic"), (&g, "rational")] {
            let fb = FactorBase::build(coeffs, bound, 4);
            assert_eq!(fb.len(), primes.len());
            let hom = Homogeneous::new(coeffs);
            let len = (2 * a_max + 1) as usize;
            let slack = default_slack(bound);
            let mut agree = 0u64;
            for b in 1..=60i64 {
                let pows = hom.powers_of_b(b);
                let mut acc = vec![0u16; len];
                sieve_row(&fb, b, a_max, &mut acc);
                let mut candidate = vec![false; len];
                let mut values = Vec::with_capacity(len);
                for j in 0..len {
                    let a = j as i64 - a_max;
                    let v = hom.eval_with_powers(a, &pows);
                    let mag = v.magnitude().clone();
                    let ln = crate::dickman::ln_biguint(&mag);
                    if coprime(a, b) && !mag.is_zero() {
                        candidate[j] = acc[j] as f64 / LOG_SCALE >= ln - slack;
                    }
                    values.push(mag);
                }
                let hits = resieve_row(&fb, b, a_max, &candidate);
                let mut k = 0usize;
                for j in 0..len {
                    let a = j as i64 - a_max;
                    if !coprime(a, b) || values[j].is_zero() {
                        continue;
                    }
                    let mut ps = Vec::new();
                    while k < hits.len() && (hits[k].0 as usize) < j {
                        k += 1;
                    }
                    let mut kk = k;
                    while kk < hits.len() && hits[kk].0 as usize == j {
                        ps.push(hits[kk].1);
                        kk += 1;
                    }
                    let by_sieve = if candidate[j] { verify(&values[j], &ps) } else { None };
                    let by_trial = largest_prime_factor_if_smooth(&values[j], &primes, bound);
                    assert_eq!(by_sieve, by_trial, "{name} a={a} b={b}");
                    agree += 1;
                }
            }
            assert!(agree > 10_000);
        }
    }

    #[test]
    fn range_sieve_matches_row_sieve_and_marks_consecutive_integers() {
        let f = poly(&[-85186479, -38330143, 45773675, 567699, 7560]);
        let fb = FactorBase::build(&f, 1 << 10, 2);
        let a_max = 700i64;
        let mut whole = vec![0u16; (2 * a_max + 1) as usize];
        sieve_row(&fb, 17, a_max, &mut whole);
        // The window a in [−100, 299] of the same row, sieved as a range.
        let mut part = vec![0u16; 400];
        sieve_range(&fb, 17, -100, &mut part);
        assert_eq!(&whole[600..1000], &part[..]);
        let cand = vec![true; 400];
        let hits = resieve_range(&fb, 17, -100, &cand);
        let whole_cand = vec![true; whole.len()];
        let whole_hits: Vec<(u32, u32)> = resieve_row(&fb, 17, a_max, &whole_cand)
            .into_iter()
            .filter(|&(j, _)| (600..1000).contains(&(j as usize)))
            .map(|(j, p)| (j - 600, p))
            .collect();
        assert_eq!(hits, whole_hits);

        // x + X with b = 1: position j holds the integer X + a_lo + j, and
        // the marks are exactly the primes dividing it.
        let x0 = BigInt::parse_bytes(b"123456789012345678901234567890", 10).unwrap();
        let g = vec![x0.clone(), BigInt::from(1)];
        let gb = FactorBase::build(&g, 1 << 10, 2);
        let a_lo = 5_000i64;
        let len = 3000usize;
        let mut acc = vec![0u16; len];
        sieve_range(&gb, 1, a_lo, &mut acc);
        let cand = vec![true; len];
        let hits = resieve_range(&gb, 1, a_lo, &cand);
        let mut k = 0usize;
        for j in 0..len {
            let n = (&x0 + BigInt::from(a_lo + j as i64)).magnitude().clone();
            let mut expected = Vec::new();
            for &p in &gb.primes {
                if (&n % BigUint::from(p)).is_zero() {
                    expected.push(p);
                }
            }
            let mut got = Vec::new();
            while k < hits.len() && hits[k].0 as usize == j {
                got.push(hits[k].1);
                k += 1;
            }
            assert_eq!(got, expected, "j={j}");
            let sum: f64 = expected.iter().map(|&p| (p as f64).ln()).sum();
            assert!(((acc[j] as f64 / LOG_SCALE) - sum).abs() < 0.02 * expected.len().max(1) as f64, "j={j}");
        }
    }

    #[test]
    fn projective_and_content_primes() {
        // 6x + 1: leading coefficient 6, so rows with b ≡ 0 mod 2 or 3 are hit everywhere.
        let f = poly(&[1, 6]);
        let fb = FactorBase::build(&f, 10, 1);
        let mut acc = vec![0u16; 11];
        sieve_row(&fb, 6, 5, &mut acc);
        let ln2 = (2f64.ln() * LOG_SCALE).round() as u16;
        let ln3 = (3f64.ln() * LOG_SCALE).round() as u16;
        assert!(acc.iter().all(|&v| v >= ln2 + ln3));
        assert_eq!(verify(&BigUint::from(360u32), &[2, 3, 5]), Some(5));
        assert_eq!(verify(&BigUint::from(360u32 * 7), &[2, 3, 5]), None);
    }
}
