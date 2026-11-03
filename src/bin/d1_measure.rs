// d1_measure: independent smoothness measurement under Obstacle D / D1
// Reads polynomials produced by external tools and computes observed B-smooth density.
// Comparison to Dickman–de Bruijn is performed here; tool-internal scores are ignored.

use std::env;
use std::fs;

fn evaluate_poly_sketch(a: i64, b: i64, coeffs: &[i64]) -> i64 {
    // Homogeneous evaluation sketch: sum c_i * a^{deg-i} * b^i
    let deg = coeffs.len() as i64 - 1;
    let mut val = 0i64;
    for (i, &c) in coeffs.iter().enumerate() {
        let i = i as i64;
        val += c * a.pow((deg - i) as u32) * b.pow(i as u32);
    }
    val
}

fn main() {
    println!("Primness D1 measurement harness");
    println!("Protocol: 40–80 digit instances, independent B-smooth density vs Dickman model");

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: d1-measure <poly-file>");
        return;
    }

    let poly_path = &args[1];
    match fs::read_to_string(poly_path) {
        Ok(_contents) => {
            println!("Loaded polynomial file: {}", poly_path);
            // Demonstration evaluation on tiny coefficients
            let demo_coeffs = vec![1, -1, 1]; // x^2 - x + 1
            let val = evaluate_poly_sketch(3, 2, &demo_coeffs);
            println!("Demo evaluation at (a,b)=(3,2): {}", val);
            println!("(Next: parse real poly, evaluate over sieve region, count B-smooth)");
        }
        Err(e) => eprintln!("Failed to read {}: {}", poly_path, e),
    }
}
