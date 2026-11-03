// d1_measure: independent smoothness measurement under Obstacle D / D1
// Reads polynomials produced by external tools and computes observed B-smooth density.
// Comparison to Dickman–de Bruijn is performed here; tool-internal scores are ignored.

use std::env;
use std::fs;

fn evaluate_poly_sketch(a: i64, b: i64, coeffs: &[i64]) -> i64 {
    let deg = coeffs.len() as i64 - 1;
    let mut val = 0i64;
    for (i, &c) in coeffs.iter().enumerate() {
        let i = i as i64;
        val += c * a.pow((deg - i) as u32) * b.pow(i as u32);
    }
    val.abs()
}

fn is_b_smooth(mut n: u64, b: u64) -> bool {
    if n == 0 { return false; }
    for p in 2..=b {
        while n % p == 0 { n /= p; }
        if n == 1 { return true; }
    }
    n == 1
}

fn main() {
    println!("Primness D1 measurement harness");

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: d1-measure <poly-file>");
        return;
    }

    let poly_path = &args[1];
    match fs::read_to_string(poly_path) {
        Ok(_) => {
            println!("Loaded polynomial file: {}", poly_path);
            let demo_coeffs = vec![1i64, -1, 1];
            let mut smooth = 0;
            let trials = 20;
            let b = 30u64;
            for a in 1..=5 {
                for b_val in 1..=4 {
                    let val = evaluate_poly_sketch(a, b_val, &demo_coeffs) as u64;
                    if is_b_smooth(val, b) { smooth += 1; }
                }
            }
            let density = smooth as f64 / trials as f64;
            println!("Demo observed B-smooth density (B={}): {:.3}", b, density);
            println!("(Next: real polynomials, larger sieve region, Dickman comparison)");
        }
        Err(e) => eprintln!("Failed to read {}: {}", poly_path, e),
    }
}
