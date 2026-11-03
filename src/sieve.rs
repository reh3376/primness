//! Primes by the sieve of Eratosthenes and B-smoothness by trial division.
//!
//! `largest_prime_factor_if_smooth` divides a value by every prime up to B.
//! One pass gives the largest prime factor, so smoothness at any smaller
//! bound follows from that factor without a second pass.

use num_bigint::BigUint;
use num_traits::{ToPrimitive, Zero};

/// Every prime p ≤ bound, ascending.
pub fn primes_up_to(bound: u64) -> Vec<u64> {
    if bound < 2 {
        return Vec::new();
    }
    let n = usize::try_from(bound).expect("bound fits usize");
    // Odd-only bit sieve: index i stands for 2i + 1.
    let half = n / 2 + 1;
    let mut composite = vec![0u64; half.div_ceil(64)];
    let is_composite = |composite: &[u64], i: usize| (composite[i / 64] >> (i % 64)) & 1 == 1;
    let mut i = 1usize;
    while (2 * i + 1) * (2 * i + 1) <= n {
        if !is_composite(&composite, i) {
            let p = 2 * i + 1;
            let mut j = (p * p - 1) / 2;
            while j < half {
                composite[j / 64] |= 1 << (j % 64);
                j += p;
            }
        }
        i += 1;
    }
    let mut primes = Vec::with_capacity(n / 8 + 8);
    primes.push(2);
    for i in 1..half {
        let value = 2 * i + 1;
        if value > n {
            break;
        }
        if !is_composite(&composite, i) {
            primes.push(value as u64);
        }
    }
    primes
}

/// The largest prime factor of `value` when every prime factor is at most
/// `bound`, given `primes` = all primes ≤ bound in ascending order. `None`
/// when some prime factor exceeds `bound`, and for 0. The value 1 gives 1.
pub fn largest_prime_factor_if_smooth(value: &BigUint, primes: &[u64], bound: u64) -> Option<u64> {
    if value.is_zero() {
        return None;
    }
    let mut largest = 1u64;
    let mut idx = 0usize;
    let mut big = value.clone();
    while big.bits() > 127 {
        let &p = primes.get(idx)?;
        if (&big % p).is_zero() {
            largest = p;
            loop {
                big /= p;
                if !(&big % p).is_zero() {
                    break;
                }
            }
        }
        idx += 1;
    }
    let mut wide: u128 = big.to_u128().expect("at most 127 bits");
    while wide > u64::MAX as u128 {
        let &p = primes.get(idx)?;
        let p = p as u128;
        if wide % p == 0 {
            largest = p as u64;
            while wide % p == 0 {
                wide /= p;
            }
        }
        idx += 1;
    }
    let mut n = wide as u64;
    for &p in &primes[idx..] {
        if n == 1 {
            break;
        }
        match p.checked_mul(p) {
            Some(square) if square <= n => {}
            _ => break,
        }
        if n % p == 0 {
            largest = p;
            while n % p == 0 {
                n /= p;
            }
        }
    }
    if n == 1 {
        Some(largest)
    } else if n <= bound {
        Some(largest.max(n))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sieve_counts_and_edges() {
        assert!(primes_up_to(0).is_empty());
        assert!(primes_up_to(1).is_empty());
        assert_eq!(primes_up_to(2), vec![2]);
        assert_eq!(primes_up_to(3), vec![2, 3]);
        assert_eq!(primes_up_to(30), vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29]);
        assert_eq!(primes_up_to(100).len(), 25);
        assert_eq!(primes_up_to(1_000_000).len(), 78_498);
        let primes = primes_up_to(1 << 16);
        assert_eq!(primes.len(), 6_542);
        assert_eq!(*primes.last().unwrap(), 65_521);
    }

    fn lpf(value: &BigUint, bound: u64) -> Option<u64> {
        let primes = primes_up_to(bound);
        largest_prime_factor_if_smooth(value, &primes, bound)
    }

    #[test]
    fn smoothness_by_trial_division() {
        let value = BigUint::from(1024u32 * 243 * 7);
        assert_eq!(lpf(&value, 7), Some(7));
        assert_eq!(lpf(&value, 100), Some(7));
        assert_eq!(lpf(&value, 5), None);
        assert_eq!(lpf(&BigUint::from(1u32), 2), Some(1));
        assert_eq!(lpf(&BigUint::from(0u32), 2), None);
        let prime = BigUint::from(1_000_003u32);
        assert_eq!(lpf(&prime, 1 << 20), Some(1_000_003));
        assert_eq!(lpf(&prime, 1000), None);
        let two_primes = BigUint::from(1_000_003u64 * 1_000_033);
        assert_eq!(lpf(&two_primes, 1 << 20), Some(1_000_033));
        assert_eq!(lpf(&two_primes, 1_000_010), None);
    }

    #[test]
    fn smoothness_through_the_wide_phases() {
        let two_pow_130 = BigUint::from(1u32) << 130;
        assert_eq!(lpf(&two_pow_130, 2), Some(2));
        let mixed = &two_pow_130 * BigUint::from(1_000_003u32);
        assert_eq!(lpf(&mixed, 1 << 20), Some(1_000_003));
        assert_eq!(lpf(&mixed, 1 << 16), None);
        let wide_prime_cofactor = (BigUint::from(1u32) << 70) * BigUint::from(4_294_967_311u64);
        assert_eq!(lpf(&wide_prime_cofactor, 1 << 20), None);
        let three_pow_90 = BigUint::from(3u32).pow(90) * BigUint::from(65_521u32);
        assert_eq!(lpf(&three_pow_90, 1 << 16), Some(65_521));
    }
}
