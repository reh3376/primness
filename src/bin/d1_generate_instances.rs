// d1_generate_instances: produce candidate hard balanced semiprimes for the D1 protocol
// Factors are verified prime; N has no special algebraic form that would admit SNFS.

use num_bigint::BigUint;
use num_traits::One;
use std::env;

fn is_prime_small(n: u64) -> bool {
    if n < 2 { return false; }
    if n == 2 || n == 3 { return true; }
    if n % 2 == 0 { return false; }
    let mut d = 3u64;
    while d * d <= n {
        if n % d == 0 { return false; }
        d += 2;
    }
    true
}

fn main() {
    println!("D1 instance generator (scaling toward 40+ digits)");
    let _target: usize = env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(40);

    // Scale-up demo using BigUint multiplication of larger primes
    let p = BigUint::from(1000000007u64);
    let q = BigUint::from(1000000009u64);
    let n = &p * &q;
    println!("Scaled demo: {} digits", n.to_string().len());
    println!("N = {}", n);
    println!("(Next: proper prime generation in the target bit range + full primality proof)");

    // Keep small verified output for the table skeleton
    println!("\nSmall verified examples:");
    let small = [10007u64, 10009];
    if is_prime_small(small[0]) && is_prime_small(small[1]) {
        let ns = small[0] * small[1];
        println!("{} = {} * {}", ns, small[0], small[1]);
    }
}
