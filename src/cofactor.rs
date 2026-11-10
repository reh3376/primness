//! The split n = s·c behind the conditional baseline.
//!
//! s is the part of n made of the primes up to a rough bound y, with their
//! full multiplicity, and c the cofactor, which has no prime factor up to y.
//! `d1-baseline` tabulates the smoothness of consecutive integers by the size
//! of c, and `d1-measure` bins polynomial values the same way, so both use
//! the definition and the bin rule here. docs/baseline_calibration.md,
//! section 7.

use crate::sieve::primes_up_to;
use num_bigint::BigUint;
use num_traits::{ToPrimitive, Zero};

/// Default width of the ln c bins: an eighth of a bit. With a width of
/// ln 2 over an integer, every bound that is a power of two falls on a bin
/// edge, and the density of smooth cofactors jumps exactly there (a cofactor
/// below B is always B-smooth; just above B the primes are not).
pub const DEFAULT_BIN_WIDTH: f64 = std::f64::consts::LN_2 / 8.0;

/// The bin of a cofactor of size ln c: bin k covers [k·width, (k+1)·width),
/// the last bin takes anything beyond, and bin 0 holds c = 1. A cofactor is
/// 1 or exceeds the rough bound, so anything below 0.5 is a 1 up to rounding.
#[inline]
pub fn bin_index(ln_c: f64, width: f64, bins: usize) -> usize {
    if ln_c < 0.5 {
        0
    } else {
        ((ln_c / width) as usize).min(bins - 1)
    }
}

/// ln c as tallied: exactly 0 for a cofactor of 1.
#[inline]
pub fn tallied_ln_c(ln_c: f64) -> f64 {
    if ln_c < 0.5 {
        0.0
    } else {
        ln_c
    }
}

/// Trial division by the primes up to a bound, grouped so that one bigint
/// remainder by the product of a group tests every prime of the group.
pub struct SmallPart {
    pub bound: u64,
    pub primes: Vec<u64>,
    groups: Vec<(u64, Vec<u64>)>,
}

impl SmallPart {
    pub fn new(bound: u64) -> Self {
        let primes = primes_up_to(bound);
        let mut groups: Vec<(u64, Vec<u64>)> = Vec::new();
        for &p in &primes {
            match groups.last_mut() {
                Some((product, members)) if product.checked_mul(p).is_some() => {
                    *product *= p;
                    members.push(p);
                }
                _ => groups.push((p, vec![p])),
            }
        }
        SmallPart { bound, primes, groups }
    }

    /// ln s(n): the sum of v_p(n)·ln p over the primes up to the bound. n must not be zero.
    pub fn ln(&self, n: &BigUint) -> f64 {
        let mut total = 0.0;
        let mut reduced: Option<BigUint> = None;
        for (product, members) in &self.groups {
            let r = (n % *product).to_u64().expect("a remainder below 2^64");
            for &p in members {
                if r % p != 0 {
                    continue;
                }
                // p divides n: count its multiplicity on a working copy.
                let m = reduced.get_or_insert_with(|| n.clone());
                let ln_p = (p as f64).ln();
                while (&*m % p).is_zero() {
                    *m /= p;
                    total += ln_p;
                }
            }
        }
        total
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn by_division(n: &BigUint, primes: &[u64]) -> f64 {
        let mut n = n.clone();
        let mut total = 0.0;
        for &p in primes {
            while (&n % p).is_zero() {
                n /= p;
                total += (p as f64).ln();
            }
        }
        total
    }

    #[test]
    fn grouped_division_agrees_with_plain_division() {
        let sp = SmallPart::new(2000);
        assert_eq!(sp.primes.len(), 303);
        assert!(sp.groups.len() < 80);
        let base = BigUint::parse_bytes(b"982451653000000000000000000000000000001", 10).unwrap();
        for j in 0..3000u32 {
            let n = &base + BigUint::from(j) * BigUint::from(7919u32);
            assert!((sp.ln(&n) - by_division(&n, &sp.primes)).abs() < 1e-9, "j={j}");
        }
        // Full multiplicity, a prime at the bound, and a prime just past it.
        let n = (BigUint::from(1u32) << 90u32) * BigUint::from(3u32).pow(40) * BigUint::from(1999u32).pow(3) * BigUint::from(2003u32);
        let want = 90.0 * 2f64.ln() + 40.0 * 3f64.ln() + 3.0 * 1999f64.ln();
        assert!((sp.ln(&n) - want).abs() < 1e-9);
        assert_eq!(sp.ln(&BigUint::from(2003u32)), 0.0);
        assert_eq!(sp.ln(&BigUint::from(1u32)), 0.0);
    }

    #[test]
    fn bins() {
        assert_eq!(bin_index(0.0, 0.25, 100), 0);
        assert_eq!(bin_index(1e-6, 0.25, 100), 0);
        assert_eq!(bin_index(7.61, 0.25, 100), 30);
        assert_eq!(bin_index(24.99, 0.25, 100), 99);
        assert_eq!(bin_index(500.0, 0.25, 100), 99);
        assert_eq!(tallied_ln_c(1e-6), 0.0);
        assert_eq!(tallied_ln_c(9.0), 9.0);
    }
}
