// d1_measure: independent smoothness measurement under Obstacle D / D1.
// The default region is a scaled-down toy. It is not the 40–80 digit protocol.

use primness::metrics::{self, Event, Sink};
use std::env;
use std::fs;
use std::time::Instant;

fn evaluate_poly_sketch(a: i64, b: i64, coeffs: &[i64]) -> u64 {
    let deg = coeffs.len() as i64 - 1;
    let mut val: i128 = 0;
    for (i, &c) in coeffs.iter().enumerate() {
        let i = i as u32;
        let av = (a as i128).pow((deg as u32).saturating_sub(i));
        let bv = (b as i128).pow(i);
        val += c as i128 * av * bv;
    }
    val.unsigned_abs().try_into().unwrap_or(u64::MAX)
}

fn is_b_smooth(mut n: u64, b: u64) -> bool {
    if n <= 1 {
        return n == 1;
    }
    if b < 2 {
        return false;
    }
    for p in 2..=b {
        while n % p == 0 {
            n /= p;
        }
        if n == 1 {
            return true;
        }
    }
    false
}

fn parse_coeffs(text: &str) -> Vec<i64> {
    let coeffs: Vec<i64> = text
        .lines()
        .filter_map(|line| {
            let line = line.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                None
            } else {
                line.parse().ok()
            }
        })
        .collect();
    if coeffs.is_empty() {
        vec![1, -1, 1]
    } else {
        coeffs
    }
}

fn main() {
    let mut region: Option<i64> = None;
    let mut poly_path: Option<String> = None;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--region" {
            region = args.next().and_then(|value| value.parse().ok());
            if region.is_none() {
                eprintln!("Usage: d1-measure [--region N] <poly-file>");
                std::process::exit(2);
            }
        } else if arg.starts_with("--") {
            eprintln!("Unknown option {arg}");
            std::process::exit(2);
        } else {
            poly_path = Some(arg);
        }
    }
    let Some(poly_path) = poly_path else {
        eprintln!("Usage: d1-measure [--region N] <poly-file>");
        std::process::exit(2);
    };
    let text = match fs::read_to_string(&poly_path) {
        Ok(text) => text,
        Err(err) => {
            eprintln!("Failed to read {poly_path}: {err}");
            std::process::exit(1);
        }
    };
    let coeffs = parse_coeffs(&text);
    let (a_max, b_max, bounds): (i64, i64, Vec<u64>) = match region {
        Some(n) => {
            let n = n.clamp(2, 2000);
            (n, (n / 2).max(1), vec![32, 128, 512])
        }
        None => (10, 5, vec![10, 30, 100]),
    };

    println!("Primness D1 measurement harness");
    println!("Loaded polynomial file: {poly_path}");
    println!("scale: toy (not the 40–80 digit protocol)");

    let started = Instant::now();
    let run_id = format!("d1m-{}-{}", std::process::id(), metrics::now_ms());
    let instance = poly_path.clone();
    let mut sink = Sink::from_env(&run_id);
    sink.emit(&event(&run_id, &instance, "start", 0, 0, 0, 0, 0.0, ""));

    let mut evaluated: u64 = 0;
    let mut smooth: u64 = 0;
    let mut residues = String::new();
    let mut since = Instant::now();
    let mut since_count: u64 = 0;
    let chunk = 250u64;

    for &bound in &bounds {
        for a in 1..=a_max {
            for b in 1..=b_max {
                let value = evaluate_poly_sketch(a, b, &coeffs);
                let hit = is_b_smooth(value, bound);
                if hit {
                    smooth += 1;
                }
                evaluated += 1;
                since_count += 1;
                residues.push(if hit { '1' } else { '0' });
                if residues.len() > 64 {
                    residues.remove(0);
                }
                if since_count >= chunk {
                    emit_sample(
                        &mut sink,
                        &run_id,
                        &instance,
                        started,
                        since,
                        since_count,
                        bound,
                        evaluated,
                        smooth,
                        &residues,
                    );
                    since = Instant::now();
                    since_count = 0;
                }
            }
        }
        if since_count > 0 {
            emit_sample(
                &mut sink,
                &run_id,
                &instance,
                started,
                since,
                since_count,
                bound,
                evaluated,
                smooth,
                &residues,
            );
            since = Instant::now();
            since_count = 0;
        }
        let density = metrics::density(smooth, evaluated);
        println!(
            "B={bound:<4}  observed density: {density:.3}  ({smooth}/{evaluated})"
        );
    }

    let elapsed_ms = started.elapsed().as_millis() as u64;
    let rate = if elapsed_ms == 0 {
        0.0
    } else {
        evaluated as f64 / (elapsed_ms as f64 / 1000.0)
    };
    sink.emit(&event(
        &run_id,
        &instance,
        "finish",
        bounds.last().copied().unwrap_or(0),
        evaluated,
        smooth,
        elapsed_ms,
        rate,
        &residues,
    ));
    println!("(Next: real polynomials from external instruments + Dickman comparison)");
}

fn emit_sample(
    sink: &mut Sink,
    run_id: &str,
    instance: &str,
    started: Instant,
    since: Instant,
    since_count: u64,
    bound: u64,
    evaluated: u64,
    smooth: u64,
    residues: &str,
) {
    let chunk_us = since.elapsed().as_micros().max(1) as u64;
    let rate = since_count as f64 / (chunk_us as f64 / 1_000_000.0);
    sink.emit(&event(
        run_id,
        instance,
        "sample",
        bound,
        evaluated,
        smooth,
        started.elapsed().as_millis() as u64,
        rate,
        residues,
    ));
}

fn event(
    run_id: &str,
    instance: &str,
    kind: &str,
    b: u64,
    evaluated: u64,
    smooth: u64,
    elapsed_ms: u64,
    rate_per_s: f64,
    residues: &str,
) -> Event {
    Event {
        ts_ms: metrics::now_ms(),
        run_id: run_id.to_string(),
        source: "d1-measure".into(),
        kind: kind.into(),
        scale: "toy".into(),
        instance: instance.into(),
        b,
        evaluated,
        smooth,
        elapsed_ms,
        rate_per_s,
        residues: residues.into(),
    }
}
