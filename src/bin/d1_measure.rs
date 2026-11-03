// d1_measure: independent smoothness measurement under Obstacle D / D1

use std::env;
use std::fs;

fn evaluate_poly_sketch(a: i64, b: i64, coeffs: &[i64]) -> u64 {
    let deg = coeffs.len() as i64 - 1;
    let mut val = 0i64;
    for (i, &c) in coeffs.iter().enumerate() {
        let i = i as i64;
        val += c * a.pow((deg - i) as u32) * b.pow(i as u32);
    }
    val.unsigned_abs()
}

fn is_b_smooth(mut n: u64, b: u64) -> bool {
    if n <= 1 { return n == 1; }
    for p in 2..=b {
        while n % p == 0 { n /= p; }
        if n == 1 { return true; }
    }
    false
}

fn main() {
    println!("Primness D1 measurement harness");

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: d1-measure <poly-file>");
        return;
    }

    let poly_path = &args[1];
    if fs::read_to_string(poly_path).is_err() {
        eprintln!("Failed to read {}", poly_path);
        return;
    }

    println!("Loaded polynomial file: {}", poly_path);
    let demo_coeffs = vec![1i64, -1, 1];
    let bounds = [10u64, 30, 100];

    for &b in &bounds {
        let mut smooth = 0;
        let mut total = 0;
        for a in 1..=10 {
            for bv in 1..=5 {
                let val = evaluate_poly_sketch(a, bv, &demo_coeffs);
                if is_b_smooth(val, b) { smooth += 1; }
                total += 1;
            }
        }
        let density = smooth as f64 / total as f64;
        println!("B={:3}  observed density: {:.3}  ({}/{})", b, density, smooth, total);
    }
    println!("(Next: real polynomials from external instruments + Dickman comparison)");
}
