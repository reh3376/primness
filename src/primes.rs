//! Deterministic prime proofs for the D1 instance table.
//!
//! A prime of at most 24 decimal digits is below ψ_13 from Sorenson and Webster,
//! Mathematics of Computation 86 (2017), 985–1003. Every odd composite smaller
//! than that bound fails the strong pseudoprime test on at least one of the
//! first 13 prime bases, so Miller-Rabin with those bases is a proof there.
//!
//! A larger prime N is proved by the basic Pocklington criterion: a prime F
//! divides N−1, F² > N, and some witness a satisfies a^{N−1} ≡ 1 (mod N) and
//! gcd(a^{(N−1)/F} − 1, N) = 1. F is proved by the same functions.

use num_bigint::BigUint;
use num_traits::{One, Zero};
use std::fmt::Write as _;

const MR_BASES: [u32; 13] = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41];

/// ψ_13, the smallest strong pseudoprime to the first 13 prime bases.
pub const PSI_13: &str = "3317044064679887385961981";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Proof {
    MillerRabin,
    Pocklington {
        factor: Box<CertifiedPrime>,
        witness: u32,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CertifiedPrime {
    pub n: BigUint,
    pub proof: Proof,
}

pub trait Rng32 {
    fn next_u32(&mut self) -> u32;

    fn fill(&mut self, dest: &mut [u8]) {
        for chunk in dest.chunks_mut(4) {
            let bytes = self.next_u32().to_le_bytes();
            chunk.copy_from_slice(&bytes[..chunk.len()]);
        }
    }
}

pub struct Lcg(pub u64);

impl Rng32 for Lcg {
    fn next_u32(&mut self) -> u32 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        (self.0 >> 32) as u32
    }
}

pub struct Urandom;

impl Rng32 for Urandom {
    fn next_u32(&mut self) -> u32 {
        let mut bytes = [0u8; 4];
        self.fill(&mut bytes);
        u32::from_le_bytes(bytes)
    }

    fn fill(&mut self, dest: &mut [u8]) {
        use std::io::Read;
        let mut file = std::fs::File::open("/dev/urandom")
            .unwrap_or_else(|err| panic!("open /dev/urandom: {err}"));
        file.read_exact(dest)
            .unwrap_or_else(|err| panic!("read /dev/urandom: {err}"));
    }
}

pub fn psi_13() -> BigUint {
    BigUint::parse_bytes(PSI_13.as_bytes(), 10).expect("ψ_13 decimal")
}

pub fn prove_prime(digits: usize, rng: &mut impl Rng32) -> Result<CertifiedPrime, String> {
    if digits == 0 {
        return Err("digit count must be positive".into());
    }
    if digits <= 24 {
        prove_miller_rabin(digits, rng)
    } else {
        prove_pocklington(digits, rng)
    }
}

pub fn verify(cert: &CertifiedPrime) -> Result<(), String> {
    if cert.n < BigUint::from(2u32) {
        return Err("certificate is smaller than 2".into());
    }
    match &cert.proof {
        Proof::MillerRabin => {
            if cert.n >= psi_13() {
                return Err(format!("{} is not below ψ_13", cert.n));
            }
            if !passes_miller_rabin(&cert.n) {
                return Err(format!("{} failed deterministic Miller-Rabin", cert.n));
            }
            Ok(())
        }
        Proof::Pocklington { factor, witness } => {
            verify(factor)?;
            if !pocklington_holds(&cert.n, &factor.n, *witness) {
                return Err(format!(
                    "Pocklington conditions failed for {} with factor {} and witness {witness}",
                    cert.n, factor.n
                ));
            }
            Ok(())
        }
    }
}

pub fn proof_text(cert: &CertifiedPrime) -> Result<String, String> {
    match &cert.proof {
        Proof::MillerRabin => Ok("mr".into()),
        Proof::Pocklington { factor, witness } => {
            if !matches!(factor.proof, Proof::MillerRabin) {
                return Err(
                    "this record stores one Pocklington step whose factor is Miller-Rabin".into(),
                );
            }
            Ok(format!("pocklington {} {witness}", factor.n))
        }
    }
}

pub fn parse_proof(n: &str, proof: &str) -> Result<CertifiedPrime, String> {
    let n = parse_decimal(n)?;
    let mut parts = proof.split_whitespace();
    match parts.next() {
        Some("mr") => {
            if parts.next().is_some() {
                return Err(format!("trailing data in Miller-Rabin proof for {n}"));
            }
            Ok(CertifiedPrime {
                n,
                proof: Proof::MillerRabin,
            })
        }
        Some("pocklington") => {
            let factor = parts
                .next()
                .ok_or_else(|| format!("Pocklington proof for {n} is missing its factor"))?;
            let witness = parts
                .next()
                .ok_or_else(|| format!("Pocklington proof for {n} is missing its witness"))?;
            if parts.next().is_some() {
                return Err(format!("trailing data in Pocklington proof for {n}"));
            }
            let witness: u32 = witness
                .parse()
                .map_err(|_| format!("witness in the proof of {n} is not a u32"))?;
            Ok(CertifiedPrime {
                n,
                proof: Proof::Pocklington {
                    factor: Box::new(CertifiedPrime {
                        n: parse_decimal(factor)?,
                        proof: Proof::MillerRabin,
                    }),
                    witness,
                },
            })
        }
        Some(other) => Err(format!("unknown proof kind {other} for {n}")),
        None => Err(format!("missing proof for {n}")),
    }
}

fn prove_miller_rabin(digits: usize, rng: &mut impl Rng32) -> Result<CertifiedPrime, String> {
    for _ in 0..20_000 {
        let n = random_odd_exact_digits(digits, rng);
        if n >= psi_13() {
            return Err(format!("{n} reached ψ_13 inside the Miller-Rabin range"));
        }
        if passes_miller_rabin(&n) {
            return Ok(CertifiedPrime {
                n,
                proof: Proof::MillerRabin,
            });
        }
    }
    Err(format!(
        "no {digits}-digit Miller-Rabin prime in 20000 draws"
    ))
}

fn prove_pocklington(digits: usize, rng: &mut impl Rng32) -> Result<CertifiedPrime, String> {
    let factor_digits = digits / 2 + 1;
    if !(1..digits).contains(&factor_digits) {
        return Err(format!("no Pocklington factor width for {digits} digits"));
    }
    for _ in 0..40 {
        let factor = prove_prime(factor_digits, rng)?;
        let Some((r_min, r_max)) = r_bounds(&factor.n, digits) else {
            continue;
        };
        for _ in 0..4_000 {
            let mut r = &r_min + random_below(&(&r_max - &r_min + 1u32), rng);
            if r.bit(0) {
                if r >= r_max {
                    continue;
                }
                r += 1u32;
            }
            let n = &factor.n * &r + 1u32;
            if decimal_digits(&n) != digits || &factor.n * &factor.n <= n {
                continue;
            }
            for witness in 2u32..80 {
                if pocklington_holds(&n, &factor.n, witness) {
                    return Ok(CertifiedPrime {
                        n,
                        proof: Proof::Pocklington {
                            factor: Box::new(factor),
                            witness,
                        },
                    });
                }
            }
        }
    }
    Err(format!("no {digits}-digit Pocklington prime found"))
}

fn r_bounds(factor: &BigUint, digits: usize) -> Option<(BigUint, BigUint)> {
    let low = pow10(digits - 1);
    let high = pow10(digits);
    if &low <= &BigUint::one() {
        return None;
    }
    let r_min = ceil_div(&(low - 1u32), factor);
    let r_max_from_digits = (&high - 2u32) / factor;
    let r_max_from_square = factor - 1u32;
    let r_max = if r_max_from_digits < r_max_from_square {
        r_max_from_digits
    } else {
        r_max_from_square
    };
    if r_min.is_zero() || r_min > r_max {
        None
    } else {
        Some((r_min, r_max))
    }
}

fn passes_miller_rabin(n: &BigUint) -> bool {
    if n == &BigUint::from(2u32) {
        return true;
    }
    if n < &BigUint::from(2u32) || !n.bit(0) {
        return false;
    }
    MR_BASES.iter().all(|&base| {
        let a = BigUint::from(base);
        if &a % n == BigUint::zero() {
            return true;
        }
        strong_pseudoprime(n, &a)
    })
}

fn strong_pseudoprime(n: &BigUint, a: &BigUint) -> bool {
    let one = BigUint::one();
    let n_minus_1 = n - &one;
    let mut d = n_minus_1.clone();
    let mut s = 0u32;
    while !d.bit(0) {
        d >>= 1;
        s += 1;
    }
    let mut x = a.modpow(&d, n);
    if x == one || x == n_minus_1 {
        return true;
    }
    for _ in 1..s {
        x = x.modpow(&BigUint::from(2u32), n);
        if x == n_minus_1 {
            return true;
        }
    }
    false
}

fn pocklington_holds(n: &BigUint, factor: &BigUint, witness: u32) -> bool {
    if factor < &BigUint::from(2u32) || n <= factor {
        return false;
    }
    if factor * factor <= *n {
        return false;
    }
    let n_minus_1 = n - 1u32;
    if &n_minus_1 % factor != BigUint::zero() {
        return false;
    }
    let a = BigUint::from(witness);
    if gcd(a.clone(), n.clone()) != BigUint::one() {
        return false;
    }
    if a.modpow(&n_minus_1, n) != BigUint::one() {
        return false;
    }
    let exp = &n_minus_1 / factor;
    let powered = a.modpow(&exp, n);
    if powered.is_zero() {
        return false;
    }
    let predecessor = if powered == BigUint::one() {
        BigUint::zero()
    } else {
        powered - 1u32
    };
    gcd(predecessor, n.clone()) == BigUint::one()
}

fn random_odd_exact_digits(digits: usize, rng: &mut impl Rng32) -> BigUint {
    loop {
        let mut n = random_exact_digits(digits, rng);
        if !n.bit(0) {
            n += 1u32;
        }
        if decimal_digits(&n) == digits {
            return n;
        }
    }
}

fn random_exact_digits(digits: usize, rng: &mut impl Rng32) -> BigUint {
    let mut text = String::with_capacity(digits);
    text.push(char::from(b'1' + (rng.next_u32() % 9) as u8));
    for _ in 1..digits {
        text.push(char::from(b'0' + (rng.next_u32() % 10) as u8));
    }
    BigUint::parse_bytes(text.as_bytes(), 10).expect("decimal digits")
}

fn random_below(limit: &BigUint, rng: &mut impl Rng32) -> BigUint {
    if limit.is_zero() {
        return BigUint::zero();
    }
    let bits = limit.bits() as usize;
    let nbytes = bits.div_ceil(8);
    let excess = nbytes * 8 - bits;
    loop {
        let mut buf = vec![0u8; nbytes];
        rng.fill(&mut buf);
        if excess > 0 {
            buf[0] &= 0xff >> excess;
        }
        let candidate = BigUint::from_bytes_be(&buf);
        if &candidate < limit {
            return candidate;
        }
    }
}

fn pow10(exp: usize) -> BigUint {
    BigUint::from(10u32).pow(exp as u32)
}

fn decimal_digits(n: &BigUint) -> usize {
    n.to_string().len()
}

fn ceil_div(numerator: &BigUint, denominator: &BigUint) -> BigUint {
    (numerator + denominator - 1u32) / denominator
}

fn gcd(mut a: BigUint, mut b: BigUint) -> BigUint {
    while !b.is_zero() {
        let remainder = &a % &b;
        a = b;
        b = remainder;
    }
    a
}

fn parse_decimal(text: &str) -> Result<BigUint, String> {
    BigUint::parse_bytes(text.as_bytes(), 10)
        .ok_or_else(|| format!("not a decimal integer: {text}"))
}

pub fn render_instance_file(rows: &[(String, usize, CertifiedPrime, CertifiedPrime)]) -> String {
    let mut out = String::new();
    out.push_str("# D1 Test Instances (40–80 decimal digits)\n\n");
    out.push_str("Hard balanced semiprimes N = p · q with no chosen special algebraic form. Used for the first falsifiable prediction under Obstacle D / D1. External tools supply only the polynomials; the Primness harness computes independent B-smooth densities.\n\n");
    out.push_str("| ID | Digits | N (decimal) | Proof |\n");
    out.push_str("|----|--------|-------------|-------|\n");
    for (id, digits, p, q) in rows {
        let n = &p.n * &q.n;
        let _ = writeln!(
            out,
            "| {id} | {digits} | {n} | p and q proved; see Certificates |"
        );
    }
    out.push_str("\n**Generation rule**\n\n");
    out.push_str("Each N is the product of two primes with the same number of decimal digits, and N has the digit count in the table. A prime of at most 24 digits is proved by Miller-Rabin on the bases 2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, and 41. Sorenson and Webster, *Mathematics of Computation* 86 (2017), proved that the smallest strong pseudoprime to those bases is ψ_13 = ");
    out.push_str(PSI_13);
    out.push_str(". Every smaller integer that passes is prime. A larger prime N is proved by Pocklington’s criterion: a prime F divides N−1, F² > N, and a witness a satisfies a^{N−1} ≡ 1 (mod N) and gcd(a^{(N−1)/F} − 1, N) = 1. That F is proved by the Miller-Rabin test above.\n\n");
    out.push_str("The repository does not define a test for SNFS-friendly algebraic structure. These factors are random primes of equal length. They were not chosen as Cunningham numbers or another special form. That is not a proof that no SNFS polynomial exists.\n\n");
    out.push_str("**Certificates**\n\n");
    out.push_str(
        "Each factor line is `id role decimal mr` or `id role decimal pocklington F witness`.\n\n",
    );
    for (id, _, p, q) in rows {
        let n = &p.n * &q.n;
        let _ = writeln!(out, "{id} n {n}");
        let _ = writeln!(
            out,
            "{id} p {} {}",
            p.n,
            proof_text(p).expect("recorded proof")
        );
        let _ = writeln!(
            out,
            "{id} q {} {}",
            q.n,
            proof_text(q).expect("recorded proof")
        );
    }
    out.push_str("\n**Next**\n\n");
    out.push_str("Obtain one polynomial per instance from CADO-NFS and one from msieve, then run the independent evaluation inside `d1_measure`.\n");
    out
}

pub fn semiprime(
    digits: usize,
    rng: &mut impl Rng32,
) -> Result<(CertifiedPrime, CertifiedPrime), String> {
    if digits < 2 || digits % 2 != 0 {
        return Err(format!(
            "this generator builds an even digit count, got {digits}"
        ));
    }
    let half = digits / 2;
    for _ in 0..40 {
        let p = prove_prime(half, rng)?;
        let q = prove_prime(half, rng)?;
        if p.n == q.n {
            continue;
        }
        let n = &p.n * &q.n;
        if decimal_digits(&n) == digits {
            verify(&p)?;
            verify(&q)?;
            return Ok((p, q));
        }
    }
    Err(format!(
        "no {digits}-digit product of two {half}-digit primes"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn miller_rabin_accepts_a_prime_and_rejects_pseudoprimes() {
        let prime = CertifiedPrime {
            n: BigUint::from(97u32),
            proof: Proof::MillerRabin,
        };
        verify(&prime).unwrap();
        assert!(!passes_miller_rabin(&BigUint::from(2047u32)));
        assert!(!passes_miller_rabin(&BigUint::from(3_215_031_751u64)));
        assert!(!passes_miller_rabin(&BigUint::from(91u32)));
        assert!(passes_miller_rabin(&BigUint::from(97u32)));
    }

    #[test]
    fn eight_digit_prime_is_miller_rabin() {
        let mut rng = Lcg(0x1234_5678_9abc_def0);
        let cert = prove_prime(8, &mut rng).unwrap();
        assert_eq!(decimal_digits(&cert.n), 8);
        verify(&cert).unwrap();
        assert!(matches!(cert.proof, Proof::MillerRabin));
    }

    #[test]
    fn pocklington_prime_verifies() {
        let mut rng = Lcg(0x0fed_cba9_8765_4321);
        let cert = prove_prime(28, &mut rng).unwrap();
        assert_eq!(decimal_digits(&cert.n), 28);
        verify(&cert).unwrap();
        let text = proof_text(&cert).unwrap();
        let parsed = parse_proof(&cert.n.to_string(), &text).unwrap();
        verify(&parsed).unwrap();
    }

    #[test]
    fn committed_instance_file_rechecks() {
        let text = include_str!("../data/d1_test_instances.md");
        let mut factors: std::collections::BTreeMap<String, Vec<CertifiedPrime>> =
            std::collections::BTreeMap::new();
        let mut products: std::collections::BTreeMap<String, BigUint> =
            std::collections::BTreeMap::new();
        for line in text.lines() {
            let mut parts = line.split_whitespace();
            let Some(id) = parts.next() else { continue };
            if !id.starts_with("d1-") {
                continue;
            }
            let Some(role) = parts.next() else { continue };
            let Some(decimal) = parts.next() else {
                continue;
            };
            let proof = parts.collect::<Vec<_>>().join(" ");
            if role == "n" {
                products.insert(id.to_string(), parse_decimal(decimal).unwrap());
                continue;
            }
            let cert = parse_proof(decimal, &proof).unwrap();
            verify(&cert).unwrap();
            factors.entry(id.to_string()).or_default().push(cert);
        }
        assert_eq!(products.len(), 10);
        for (id, n) in products {
            let pair = factors.remove(&id).unwrap();
            assert_eq!(pair.len(), 2);
            assert_eq!(&pair[0].n * &pair[1].n, n);
            assert_eq!(decimal_digits(&pair[0].n), decimal_digits(&pair[1].n));
            assert_eq!(decimal_digits(&n), decimal_digits(&pair[0].n) * 2);
        }
        assert!(factors.is_empty());
    }
}
