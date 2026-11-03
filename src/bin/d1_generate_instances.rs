// d1_generate_instances: prove the ten 40–80 digit semiprimes used by the D1 protocol.

use primness::primes::{self, semiprime, Urandom};
use std::env;
use std::fs;
use std::process::ExitCode;

const DIGITS: [usize; 10] = [40, 44, 48, 52, 56, 60, 64, 68, 72, 80];

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let mut write_path = None;
    while let Some(arg) = args.next() {
        if arg == "--write" {
            match args.next() {
                Some(path) => write_path = Some(path),
                None => {
                    eprintln!("Usage: d1_generate_instances --write <path>");
                    return ExitCode::from(2);
                }
            }
        } else {
            eprintln!("Usage: d1_generate_instances --write <path>");
            return ExitCode::from(2);
        }
    }
    let Some(write_path) = write_path else {
        eprintln!("Usage: d1_generate_instances --write <path>");
        return ExitCode::from(2);
    };

    let mut rng = Urandom;
    let mut rows = Vec::with_capacity(DIGITS.len());
    for (index, digits) in DIGITS.iter().copied().enumerate() {
        let id = format!("d1-{:02}", index + 1);
        eprintln!("proving {id} ({digits} digits)");
        match semiprime(digits, &mut rng) {
            Ok((p, q)) => {
                let n = &p.n * &q.n;
                eprintln!("{id} | {digits} | {n}");
                rows.push((id, digits, p, q));
            }
            Err(err) => {
                eprintln!("{id}: {err}");
                return ExitCode::from(1);
            }
        }
    }
    let text = primes::render_instance_file(&rows);
    if let Err(err) = fs::write(&write_path, text) {
        eprintln!("write {write_path}: {err}");
        return ExitCode::from(1);
    }
    eprintln!("wrote {write_path}");
    ExitCode::SUCCESS
}
