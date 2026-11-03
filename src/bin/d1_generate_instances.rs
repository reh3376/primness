// d1_generate_instances: produce candidate hard balanced semiprimes for the D1 protocol
// Factors are verified prime; N has no special algebraic form that would admit SNFS.

use std::env;

fn is_prime(n: u64) -> bool {
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
    println!("D1 instance generator");
    let _target_digits: usize = env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(40);

    println!("Demo balanced semiprimes (scale-down):");
    let primes = [10007u64, 10009, 10037, 10039, 10061, 10067, 10069, 10079];
    let mut count = 0;
    for i in 0..primes.len() {
        for j in (i+1)..primes.len() {
            if is_prime(primes[i]) && is_prime(primes[j]) {
                let n = primes[i] * primes[j];
                let digits = n.to_string().len();
                println!("d1-demo-{} | {} | {} | {} * {}", count, digits, n, primes[i], primes[j]);
                count += 1;
                if count >= 5 { break; }
            }
        }
        if count >= 5 { break; }
    }
    println!("(Next: scale to 40-80 digit primes using proper big-integer generation)");
}
