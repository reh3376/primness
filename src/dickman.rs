//! Dickman–de Bruijn ρ(u).
//!
//! ρ(u) is the limiting proportion of integers up to X with no prime factor
//! above X^(1/u). It satisfies ρ(u) = 1 for 0 ≤ u ≤ 1 and, for u > 1,
//! u·ρ'(u) = −ρ(u−1), which integrates to ρ(u) = (1/u)·∫_{u−1}^{u} ρ(t) dt.
//! On [1, 2] that gives the closed form ρ(u) = 1 − ln u.
//!
//! The table applies the integral equation on a uniform grid with Simpson's
//! rule over the unit window and solves for the right endpoint. Nothing else
//! is applied: no Murphy α, no second-order correction. The caller supplies
//! u = ln x / ln B.

use num_bigint::BigUint;
use num_traits::ToPrimitive;

pub const STEPS_PER_UNIT: usize = 4096;
pub const U_MAX: usize = 40;
pub const METHOD: &str = "simpson-grid";

#[derive(Clone, Debug)]
pub struct Dickman {
    h: f64,
    table: Vec<f64>,
}

impl Default for Dickman {
    fn default() -> Self {
        Self::new()
    }
}

impl Dickman {
    pub fn new() -> Self {
        Self::with_resolution(STEPS_PER_UNIT, U_MAX)
    }

    /// `steps` grid points per unit of u (even, at least 2); the table ends at u = `u_max`.
    pub fn with_resolution(steps: usize, u_max: usize) -> Self {
        assert!(steps >= 2 && steps % 2 == 0, "steps must be even and at least 2");
        assert!(u_max >= 2, "u_max must be at least 2");
        let h = 1.0 / steps as f64;
        let n = steps * u_max;
        let mut table = vec![0.0f64; n + 1];
        for value in table.iter_mut().take(steps + 1) {
            *value = 1.0;
        }
        for (k, value) in table.iter_mut().enumerate().take(2 * steps + 1).skip(steps + 1) {
            *value = 1.0 - (k as f64 * h).ln();
        }
        // Simpson over [u−1, u] with `steps` intervals: weight 1 at both ends,
        // 4 at the odd offsets, 2 at the even interior offsets. The two running
        // sums hold the interior values split by global index parity; they are
        // rebuilt once per unit so rounding error does not accumulate.
        let mut sums = [0.0f64; 2];
        let rebuild = |sums: &mut [f64; 2], base: usize, k: usize, table: &[f64]| {
            *sums = [0.0, 0.0];
            for j in base + 1..k {
                sums[j % 2] += table[j];
            }
        };
        let first = 2 * steps + 1;
        rebuild(&mut sums, first - steps, first, &table);
        for k in first..=n {
            let base = k - steps;
            if k != first && (k - first) % steps == 0 {
                rebuild(&mut sums, base, k, &table);
            }
            let u = k as f64 * h;
            let odd_parity = (base + 1) % 2;
            let weighted = table[base] + 4.0 * sums[odd_parity] + 2.0 * sums[1 - odd_parity];
            // u·ρ(u) = (h/3)·(weighted + ρ(u))
            let value = (h / 3.0) * weighted / (u - h / 3.0);
            table[k] = value;
            if k < n {
                sums[(base + 1) % 2] -= table[base + 1];
                sums[k % 2] += value;
            }
        }
        Self { h, table }
    }

    /// ρ(u). Exact on [0, 2]; linear interpolation of the table above 2;
    /// 0 beyond the table, where ρ is below 1e-40.
    pub fn rho(&self, u: f64) -> f64 {
        if u.is_nan() {
            return f64::NAN;
        }
        if u < 0.0 {
            return 0.0;
        }
        if u <= 1.0 {
            return 1.0;
        }
        if u <= 2.0 {
            return 1.0 - u.ln();
        }
        let x = u / self.h;
        let k = x.floor() as usize;
        if k + 1 >= self.table.len() {
            return 0.0;
        }
        let t = x - k as f64;
        self.table[k] * (1.0 - t) + self.table[k + 1] * t
    }

    /// ρ'(u) = −ρ(u − 1) / u for u > 1, and 0 on [0, 1].
    pub fn rho_prime(&self, u: f64) -> f64 {
        if u.is_nan() || u <= 1.0 {
            return 0.0;
        }
        -self.rho(u - 1.0) / u
    }

    /// Local density of B-smooth integers near x, from d/dx of x·ρ(ln x / ln B):
    /// ρ(u) + ρ'(u) / ln B. The raw ρ(u) is the density averaged over [1, x].
    pub fn local_density(&self, u: f64, ln_b: f64) -> f64 {
        (self.rho(u) + self.rho_prime(u) / ln_b).max(0.0)
    }

    /// u = ln x / ln B for a value x given by its natural log.
    pub fn u(ln_value: f64, bound: u64) -> f64 {
        ln_value / (bound as f64).ln()
    }

    /// ρ(ln x / ln B): the Dickman prediction that x is B-smooth.
    pub fn smooth_probability(&self, ln_value: f64, bound: u64) -> f64 {
        self.rho(Self::u(ln_value, bound))
    }
}

/// Natural log of a positive integer, from its top 64 bits.
pub fn ln_biguint(n: &BigUint) -> f64 {
    let bits = n.bits();
    if bits == 0 {
        return f64::NEG_INFINITY;
    }
    let shift = bits.saturating_sub(64);
    let top = (n >> shift).to_u64().expect("at most 64 bits remain");
    (top as f64).ln() + shift as f64 * std::f64::consts::LN_2
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::OnceLock;

    fn table() -> &'static Dickman {
        static TABLE: OnceLock<Dickman> = OnceLock::new();
        TABLE.get_or_init(Dickman::new)
    }

    fn close(actual: f64, expected: f64, rel: f64) {
        let err = ((actual - expected) / expected).abs();
        assert!(err <= rel, "actual {actual} expected {expected} relative error {err}");
    }

    #[test]
    fn rho_is_one_up_to_one() {
        let d = table();
        assert_eq!(d.rho(0.0), 1.0);
        assert_eq!(d.rho(0.5), 1.0);
        assert_eq!(d.rho(1.0), 1.0);
        assert_eq!(d.rho(-0.5), 0.0);
    }

    #[test]
    fn rho_two_is_one_minus_ln_two() {
        let d = table();
        assert_eq!(d.rho(2.0), 1.0 - std::f64::consts::LN_2);
        assert_eq!(d.rho(1.5), 1.0 - 1.5f64.ln());
    }

    #[test]
    fn table_joins_the_closed_form_at_two() {
        let d = table();
        let h = 1.0 / STEPS_PER_UNIT as f64;
        let u = 2.0 + h;
        // Exact ρ(2+ε) = 1 − ln(2+ε) + O(ε²).
        assert!((d.rho(u) - (1.0 - u.ln())).abs() < 1e-6);
        assert!(d.rho(2.0 + 2.0 * h) < d.rho(2.0 + h));
    }

    #[test]
    fn tabulated_values_match_the_literature() {
        let d = table();
        close(d.rho(3.0), 0.0486083883, 1e-5);
        close(d.rho(4.0), 0.0049109256, 1e-4);
        close(d.rho(5.0), 0.00035472470, 1e-4);
        close(d.rho(10.0), 2.7701718e-11, 1e-2);
    }

    #[test]
    fn derivative_and_local_density() {
        let d = table();
        assert_eq!(d.rho_prime(0.5), 0.0);
        assert!((d.rho_prime(1.5) + 1.0 / 1.5).abs() < 1e-12);
        assert!((d.rho_prime(3.0) + d.rho(2.0) / 3.0).abs() < 1e-12);
        let ln_b = (1u64 << 20) as f64;
        let ln_b = ln_b.ln();
        assert!(d.local_density(1.84, ln_b) < d.rho(1.84));
        assert!((d.local_density(1.84, ln_b) - (d.rho(1.84) - 1.0 / (1.84 * ln_b))).abs() < 1e-12);
        assert_eq!(d.local_density(0.5, ln_b), 1.0);
    }

    #[test]
    fn coarse_and_fine_tables_agree() {
        let coarse = Dickman::with_resolution(512, 8);
        let fine = table();
        for u in [2.5, 3.0, 4.5, 6.0, 7.5] {
            close(coarse.rho(u), fine.rho(u), 1e-4);
        }
    }

    #[test]
    fn u_and_ln_of_big_integers() {
        assert!((Dickman::u(40.0 * std::f64::consts::LN_2, 1 << 20) - 2.0).abs() < 1e-12);
        let ten_pow_30 = BigUint::from(10u32).pow(30);
        close(ln_biguint(&ten_pow_30), 30.0 * std::f64::consts::LN_10, 1e-12);
        let two_pow_200 = BigUint::from(1u32) << 200;
        close(ln_biguint(&two_pow_200), 200.0 * std::f64::consts::LN_2, 1e-12);
        assert_eq!(ln_biguint(&BigUint::from(1u32)), 0.0);
        let d = table();
        assert_eq!(d.smooth_probability(0.0, 1 << 20), 1.0);
    }
}
