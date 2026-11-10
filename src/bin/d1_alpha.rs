//! d1-alpha: Murphy α and the excess variance α₂ for instrument polynomials.
//!
//! Reads every `<instruments>/<id>/cado.poly` (or the files named on the
//! command line), computes both sides' α (docs/murphy_alpha.md) and α₂
//! (docs/baseline_calibration.md) over primes up to the bound, and writes an
//! `alpha-moments/1` result with a CSV table beside it.

use primness::alpha;
use primness::metrics;
use primness::poly;
use primness::schema::Registry;
use serde_json::json;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub const RESULT_V1: &str = "alpha-moments/1";

fn usage() -> ! {
    eprintln!("Usage: d1-alpha [--bound N] [--instruments DIR] [--out FILE] [poly-file ...]");
    std::process::exit(2);
}

fn write_text(path: &Path, text: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("{}: {err}", parent.display()))?;
    }
    fs::write(path, text).map_err(|err| format!("{}: {err}", path.display()))
}

fn run() -> Result<(), String> {
    let mut bound = alpha::DEFAULT_BOUND;
    let mut instruments = PathBuf::from("data/instruments");
    let mut out = PathBuf::from("data/results/alpha_moments.json");
    let mut files: Vec<(String, PathBuf)> = Vec::new();
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--bound" => bound = args.next().and_then(|v| v.parse().ok()).filter(|n| *n >= 2).unwrap_or_else(|| usage()),
            "--instruments" => instruments = PathBuf::from(args.next().unwrap_or_else(|| usage())),
            "--out" => out = PathBuf::from(args.next().unwrap_or_else(|| usage())),
            other if other.starts_with("--") => usage(),
            _ => {
                let path = PathBuf::from(&arg);
                let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("poly").to_string();
                files.push((stem, path));
            }
        }
    }
    if files.is_empty() {
        let mut dirs: Vec<PathBuf> = fs::read_dir(&instruments)
            .map_err(|err| format!("{}: {err}", instruments.display()))?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.join("cado.poly").is_file())
            .collect();
        dirs.sort();
        for dir in dirs {
            let id = dir.file_name().and_then(|s| s.to_str()).unwrap_or("instrument").to_string();
            files.push((id, dir.join("cado.poly")));
        }
    }
    if files.is_empty() {
        return Err(format!("no polynomial files under {}", instruments.display()));
    }
    let registry = Registry::from_env()?;
    let mut polynomials = Vec::new();
    let mut csv = String::from("instance,side,degree,n_digits,alpha,alpha2\n");
    for (instance, path) in &files {
        let text = fs::read_to_string(path).map_err(|err| format!("{}: {err}", path.display()))?;
        let p = poly::parse_instrument(&text).map_err(|err| format!("{}: {err}", path.display()))?;
        let mut sides = Vec::new();
        for (name, coeffs) in [("algebraic", &p.algebraic), ("rational", &p.rational)] {
            let a = alpha::alpha(coeffs, bound);
            let a2 = alpha::alpha2(coeffs, bound);
            csv.push_str(&format!("{instance},{name},{},{},{a:.6},{a2:.6}\n", p.degree(), p.n_digits()));
            sides.push(json!({"alpha": a, "alpha2": a2}));
        }
        println!(
            "{instance}: degree {} ({} digits): algebraic α {:+.4} α₂ {:+.3}; rational α {:+.4} α₂ {:+.3}",
            p.degree(),
            p.n_digits(),
            sides[0]["alpha"].as_f64().unwrap_or(0.0),
            sides[0]["alpha2"].as_f64().unwrap_or(0.0),
            sides[1]["alpha"].as_f64().unwrap_or(0.0),
            sides[1]["alpha2"].as_f64().unwrap_or(0.0)
        );
        polynomials.push(json!({
            "instance": instance,
            "path": path.to_string_lossy(),
            "degree": p.degree(),
            "n_digits": p.n_digits(),
            "algebraic": sides[0],
            "rational": sides[1],
        }));
    }
    let stem = out.display().to_string();
    let stem = stem.strip_suffix(".json").unwrap_or(&stem).to_string();
    let csv_path = PathBuf::from(format!("{stem}.csv"));
    write_text(&csv_path, &csv)?;
    let result = json!({
        "schema_version": RESULT_V1,
        "ts_ms": metrics::now_ms(),
        "bound": bound,
        "method": "murphy-valuation-moments",
        "definition": "docs/baseline_calibration.md",
        "polynomials": polynomials,
        "tables": [{
            "name": "alpha_moments",
            "path": csv_path.to_string_lossy(),
            "columns": ["instance", "side", "degree", "n_digits", "alpha", "alpha2"],
            "rows": files.len() * 2,
            "description": "per polynomial and side: Murphy alpha and the excess variance alpha2 of the log small-prime part, over primes up to bound",
        }],
    });
    registry.validate_value(RESULT_V1, &result)?;
    let text = serde_json::to_string_pretty(&result).map_err(|err| err.to_string())?;
    write_text(&out, &(text + "\n"))?;
    println!("wrote {} and {}", out.display(), csv_path.display());
    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
