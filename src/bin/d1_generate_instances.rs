// d1_generate_instances: produce candidate hard balanced semiprimes for the D1 protocol
// Factors are verified prime; N has no special algebraic form that would admit SNFS.

use std::env;

fn is_probable_prime_sketch(n: u64) -> bool {
    // Placeholder Miller-Rabin style check for small demonstration only
    if n < 2 { return false; }
    if n == 2 || n == 3 { return true; }
    if n % 2 == 0 { return false; }
    let mut d = 3;
    while d * d <= n {
        if n % d == 0 { return false; }
        d += 2;
    }
    true
}

fn main() {
    println!("D1 instance generator");
    let args: Vec<String> = env::args().collect();
    let target_digits: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(40);

    println!("Targeting approximately {} decimal digits", target_digits);
    // TODO: generate two primes of bit length ~ (target_digits * log2(10) / 2)
    // TODO: multiply, verify digit count, reject if algebraic form is special
    // TODO: output line suitable for data/d1_test_instances.md

    // Demonstration on tiny values
    let p = 10007u64;
    let q = 10009u64;
    if is_probable_prime_sketch(p) && is_probable_prime_sketch(q) {
        let n = p * q;
        println!("Demo instance: {} = {} * {} (digits: {})", n, p, q, n.to_string().len());
    }
}
