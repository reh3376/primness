//! Homogeneous evaluation F(a, b) = Σ c_i a^i b^(d−i) of f(x) = Σ c_i x^i.
//! F(a, b) is the value an NFS sieve tests for smoothness at the lattice
//! point (a, b); F(a, 1) = f(a).

use num_bigint::BigInt;
use num_traits::{One, Zero};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Homogeneous {
    coeffs: Vec<BigInt>,
}

impl Homogeneous {
    /// `coeffs[i]` is c_i. Trailing zero coefficients are dropped; the
    /// zero polynomial keeps a single zero coefficient.
    pub fn new(coeffs: &[BigInt]) -> Self {
        let mut coeffs = coeffs.to_vec();
        while coeffs.len() > 1 && coeffs.last().is_some_and(Zero::is_zero) {
            coeffs.pop();
        }
        if coeffs.is_empty() {
            coeffs.push(BigInt::zero());
        }
        Self { coeffs }
    }

    pub fn degree(&self) -> usize {
        self.coeffs.len() - 1
    }

    pub fn coefficients(&self) -> &[BigInt] {
        &self.coeffs
    }

    /// b^0, b^1, …, b^d for reuse across one row of the region.
    pub fn powers_of_b(&self, b: i64) -> Vec<BigInt> {
        let mut powers = Vec::with_capacity(self.degree() + 1);
        powers.push(BigInt::one());
        for k in 1..=self.degree() {
            let next = &powers[k - 1] * b;
            powers.push(next);
        }
        powers
    }

    /// F(a, b) with `b_pows` from `powers_of_b(b)`.
    pub fn eval_with_powers(&self, a: i64, b_pows: &[BigInt]) -> BigInt {
        let d = self.degree();
        let mut acc = self.coeffs[d].clone();
        for i in (0..d).rev() {
            acc = acc * a + &self.coeffs[i] * &b_pows[d - i];
        }
        acc
    }

    pub fn eval(&self, a: i64, b: i64) -> BigInt {
        let pows = self.powers_of_b(b);
        self.eval_with_powers(a, &pows)
    }
}

pub fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a
}

/// gcd(|a|, |b|) = 1. (0, ±1) is coprime; (0, b) for |b| > 1 is not.
pub fn coprime(a: i64, b: i64) -> bool {
    gcd(a.unsigned_abs(), b.unsigned_abs()) == 1
}

#[cfg(test)]
mod tests {
    use super::*;

    fn poly(coeffs: &[i64]) -> Homogeneous {
        Homogeneous::new(&coeffs.iter().map(|&c| BigInt::from(c)).collect::<Vec<_>>())
    }

    #[test]
    fn evaluates_the_demo_quadratic() {
        let f = poly(&[1, -1, 1]);
        assert_eq!(f.degree(), 2);
        assert_eq!(f.eval(3, 2), BigInt::from(7));
        assert_eq!(f.eval(1, 1), BigInt::from(1));
        assert_eq!(f.eval(-3, 2), BigInt::from(19));
    }

    #[test]
    fn homogeneous_agrees_with_f_of_a_over_b() {
        let f = poly(&[-1, 0, 0, 0, 1]);
        assert_eq!(f.degree(), 4);
        assert_eq!(f.eval(2, 3), BigInt::from(16 - 81));
        for a in -5i64..=5 {
            assert_eq!(f.eval(a, 1), BigInt::from(a.pow(4) - 1));
        }
        let g = poly(&[-2, 1]);
        assert_eq!(g.eval(7, 3), BigInt::from(7 - 6));
    }

    #[test]
    fn matches_direct_summation() {
        let coeffs = [5i64, -3, 0, 2, 11];
        let f = poly(&coeffs);
        for a in [-13i64, -1, 0, 1, 9, 101] {
            for b in [1i64, 2, 7, 50] {
                let mut direct = BigInt::zero();
                for (i, &c) in coeffs.iter().enumerate() {
                    direct += BigInt::from(c) * BigInt::from(a).pow(i as u32) * BigInt::from(b).pow((4 - i) as u32);
                }
                assert_eq!(f.eval(a, b), direct, "a={a} b={b}");
            }
        }
    }

    #[test]
    fn drops_trailing_zeros_and_keeps_the_zero_polynomial() {
        assert_eq!(poly(&[1, 2, 0, 0]).degree(), 1);
        let zero = poly(&[0, 0]);
        assert_eq!(zero.degree(), 0);
        assert!(zero.eval(5, 7).is_zero());
    }

    #[test]
    fn coprime_pairs() {
        assert!(coprime(-3, 5));
        assert!(!coprime(4, 6));
        assert!(coprime(0, 1));
        assert!(!coprime(0, 2));
        assert!(coprime(1, 0));
        assert_eq!(gcd(0, 0), 0);
    }
}
