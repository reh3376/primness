// d1_generate_instances: produce candidate hard balanced semiprimes for the D1 protocol

use num_bigint::BigUint;
use std::env;

fn main() {
    println!("D1 instance generator");
    let _target: usize = env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(40);

    let pairs = [
        (1000000007u64, 1000000009u64),
        (1000000021u64, 1000000033u64),
        (1000000087u64, 1000000093u64),
    ];

    println!("ID | Digits | N");
    for (i, (p, q)) in pairs.iter().enumerate() {
        let bp = BigUint::from(*p);
        let bq = BigUint::from(*q);
        let n = &bp * &bq;
        let digits = n.to_string().len();
        println!("d1-{:02} | {} | {}", i + 1, digits, n);
    }
    println!("(Next: replace with properly generated and proven primes in the 40-80 digit range)");
}
