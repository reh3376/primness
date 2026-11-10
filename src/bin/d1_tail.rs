//! d1-tail: the root counts of the instrument polynomials at every prime
//! between the rough bound and B, summed in bins of ln p.
//!
//! `d1-measure` removes the primes up to the rough bound y exactly. At a prime
//! p above y a homogeneous F(a, b) over coprime pairs is divisible by p with
//! probability r_p / (p + 1), where r_p counts the roots of F in P^1(F_p),
//! against 1/p for an integer. The comparison needs, per prime, the excess
//!
//!     η_p = (r_p / (p + 1) − 1/p) · p / (p − 1) = (r_p p − p − 1) / (p² − 1),
//!
//! which is also −1/ln p times the prime's term of Murphy α whenever p divides
//! neither the discriminant nor the leading coefficient. A sieve region whose
//! b stays below p never meets the root at infinity, (1 : 0), and there the
//! affine roots alone count, each with probability 1/p:
//!
//!     η_p^aff = (r_p^aff − 1) / (p − 1).
//!
//! The two differ by about 1/p at a prime dividing the leading coefficient and
//! by about 1/p² elsewhere. In the same way a region whose |a| stays below p
//! meets the root 0 only at (0, 1); Σ 1/(p − 1) over the primes dividing the
//! constant coefficient is what that removes from η_p^aff. The run counts the
//! roots at every prime in (y, B] and writes, per polynomial, side and bin of
//! ln p: the primes, Σ r_p, Σ η_p, Σ η_p ln p, Σ η_p^aff and that sum
//! (`root-tail/1`, table beside it).
//! docs/baseline_calibration.md section 9.

use num_bigint::BigInt;
use num_traits::{ToPrimitive, Zero};
use primness::schema::Registry;
use primness::{alpha, cofactor, fp, metrics, poly, sieve};
use serde_json::json;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

pub const RESULT_V1: &str = "root-tail/1";
const BOUND_MAX: u64 = 1 << 32;

fn usage() -> ! {
    eprintln!("Usage: d1-tail [--from Y] [--bound B] [--bin W] [--threads N] [--instruments DIR] [--out FILE] [poly-file ...]");
    std::process::exit(2);
}

fn write_text(path: &Path, text: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("{}: {err}", parent.display()))?;
    }
    fs::write(path, text).map_err(|err| format!("{}: {err}", path.display()))
}

/// One side of one polynomial: its coefficients, and the same in i128 when
/// every one fits, which makes the reduction mod p a machine operation.
struct Side {
    coeffs: Vec<BigInt>,
    small: Option<Vec<i128>>,
}

impl Side {
    fn new(coeffs: &[BigInt]) -> Self {
        let mut coeffs = coeffs.to_vec();
        while coeffs.len() > 1 && coeffs.last().is_some_and(Zero::is_zero) {
            coeffs.pop();
        }
        let small = coeffs.iter().map(ToPrimitive::to_i128).collect();
        Self { coeffs, small }
    }

    /// The roots of the homogeneous form in P^1(F_p): the affine ones, and
    /// whether the point at infinity is one (p divides the leading
    /// coefficient) and whether 0 is one (p divides the constant
    /// coefficient). A form vanishing mod p has every point.
    fn roots(&self, p: u64) -> (u64, bool, bool) {
        let (reduced, lead_vanishes) = match &self.small {
            Some(c) => {
                let r: Vec<u64> = c.iter().map(|&x| x.rem_euclid(p as i128) as u64).collect();
                let lead = r.last() == Some(&0);
                (r, lead)
            }
            None => {
                let r = fp::reduce(&self.coeffs, p);
                let lead = r.len() < self.coeffs.len();
                (r, lead)
            }
        };
        if reduced.iter().all(|&c| c == 0) {
            return (p, true, true);
        }
        let zero = reduced.first() == Some(&0);
        (fp::count_roots(&reduced, p) as u64, lead_vanishes, zero)
    }

    /// r_p, the roots in P^1(F_p).
    #[cfg(test)]
    fn projective_roots(&self, p: u64) -> u64 {
        let (affine, infinity, _) = self.roots(p);
        affine + u64::from(infinity)
    }
}

/// η_p for r_p roots: (r_p p − p − 1) / (p² − 1).
fn eta(roots: u64, p: u64) -> f64 {
    let pf = p as f64;
    ((roots as f64 - 1.0) * pf - 1.0) / (pf * pf - 1.0)
}

/// η_p^aff for r_p^aff affine roots: (r_p^aff − 1) / (p − 1).
fn eta_affine(affine: u64, p: u64) -> f64 {
    (affine as f64 - 1.0) / (p as f64 - 1.0)
}

#[derive(Clone, Default)]
struct Acc {
    roots: u64,
    eta: f64,
    eta_ln_p: f64,
    eta_affine: f64,
    eta_zero: f64,
}

/// One bin of ln p: its primes, in order, and the sums per side.
struct BinSums {
    bin: usize,
    primes: usize,
    sum_ln_p: f64,
    sides: Vec<Acc>,
}

fn sum_bin(bin: usize, primes: &[u64], sides: &[Side]) -> BinSums {
    let mut out = BinSums { bin, primes: primes.len(), sum_ln_p: 0.0, sides: vec![Acc::default(); sides.len()] };
    for &p in primes {
        let ln_p = (p as f64).ln();
        out.sum_ln_p += ln_p;
        for (acc, side) in out.sides.iter_mut().zip(sides) {
            let (affine, infinity, zero) = side.roots(p);
            let r = affine + u64::from(infinity);
            let e = eta(r, p);
            acc.roots += r;
            acc.eta += e;
            acc.eta_ln_p += e * ln_p;
            acc.eta_affine += eta_affine(affine, p);
            if zero {
                acc.eta_zero += 1.0 / (p as f64 - 1.0);
            }
        }
    }
    out
}

fn run() -> Result<(), String> {
    let mut from = alpha::DEFAULT_BOUND;
    let mut bound: u64 = 1 << 28;
    let mut width = cofactor::DEFAULT_BIN_WIDTH / 8.0;
    let mut threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
    let mut instruments = PathBuf::from("data/instruments");
    let mut out = PathBuf::from("data/results/root_tail.json");
    let mut files: Vec<(String, PathBuf)> = Vec::new();
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--from" => from = args.next().and_then(|v| v.parse().ok()).filter(|n| *n >= 2).unwrap_or_else(|| usage()),
            "--bound" => bound = args.next().and_then(|v| v.parse().ok()).filter(|n| *n >= 3 && *n <= BOUND_MAX).unwrap_or_else(|| usage()),
            "--bin" => width = args.next().and_then(|v| v.parse().ok()).filter(|w: &f64| *w > 0.0).unwrap_or_else(|| usage()),
            "--threads" => threads = args.next().and_then(|v| v.parse().ok()).filter(|n| *n >= 1).unwrap_or_else(|| usage()),
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
    if from >= bound {
        return Err(format!("--from {from} is not below --bound {bound}"));
    }
    if files.is_empty() {
        files = poly::instrument_files(&instruments)?;
    }
    if files.is_empty() {
        return Err(format!("no polynomial files under {}", instruments.display()));
    }
    let registry = Registry::from_env()?;
    let started = Instant::now();

    // Sides in the order (instance, algebraic), (instance, rational), ...
    let mut polys = Vec::new();
    let mut sides = Vec::new();
    for (instance, path) in &files {
        let text = fs::read_to_string(path).map_err(|err| format!("{}: {err}", path.display()))?;
        let p = poly::parse_instrument(&text).map_err(|err| format!("{}: {err}", path.display()))?;
        sides.push(Side::new(&p.algebraic));
        sides.push(Side::new(&p.rational));
        polys.push((instance.clone(), path.clone(), p.degree(), p.n_digits()));
    }

    let primes: Vec<u64> = sieve::primes_up_to(bound).into_iter().filter(|&p| p > from).collect();
    // Contiguous runs of primes with the same bin of ln p.
    let mut runs: Vec<(usize, usize, usize)> = Vec::new();
    for (i, &p) in primes.iter().enumerate() {
        let bin = ((p as f64).ln() / width) as usize;
        match runs.last_mut() {
            Some((b, _, end)) if *b == bin => *end = i + 1,
            _ => runs.push((bin, i, i + 1)),
        }
    }
    eprintln!("{} primes in ({from}, {bound}], {} bins, {} sides, {threads} threads", primes.len(), runs.len(), sides.len());

    // Each bin is summed by one thread in prime order, so the sums do not
    // depend on the number of threads.
    let next = AtomicUsize::new(0);
    let mut sums: Vec<BinSums> = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..threads)
            .map(|_| {
                scope.spawn(|| {
                    let mut mine = Vec::new();
                    loop {
                        let i = next.fetch_add(1, Ordering::Relaxed);
                        let Some(&(bin, start, end)) = runs.get(i) else { break };
                        mine.push(sum_bin(bin, &primes[start..end], &sides));
                    }
                    mine
                })
            })
            .collect();
        workers.into_iter().flat_map(|w| w.join().expect("worker")).collect()
    });
    sums.sort_by_key(|s| s.bin);
    let duration_ms = started.elapsed().as_millis() as u64;

    let mut csv = String::from("instance,side,bin,ln_p_lo,ln_p_hi,primes,mean_ln_p,roots,eta,eta_ln_p,eta_affine,eta_zero\n");
    let mut summaries = Vec::new();
    for (k, (instance, path, degree, n_digits)) in polys.iter().enumerate() {
        let mut side_json = Vec::new();
        for (j, name) in ["algebraic", "rational"].into_iter().enumerate() {
            let s = 2 * k + j;
            let (mut roots, mut eta_ln_p) = (0u64, 0.0f64);
            for b in &sums {
                let acc = &b.sides[s];
                roots += acc.roots;
                eta_ln_p += acc.eta_ln_p;
                csv.push_str(&format!(
                    "{instance},{name},{},{:.9},{:.9},{},{:.9},{},{:.9e},{:.9e},{:.9e},{:.9e}\n",
                    b.bin,
                    b.bin as f64 * width,
                    (b.bin + 1) as f64 * width,
                    b.primes,
                    b.sum_ln_p / b.primes as f64,
                    acc.roots,
                    acc.eta,
                    acc.eta_ln_p,
                    acc.eta_affine,
                    acc.eta_zero
                ));
            }
            side_json.push(json!({
                "alpha_tail": -eta_ln_p,
                "roots_per_prime": roots as f64 / primes.len() as f64,
            }));
        }
        println!(
            "{instance}: algebraic tail {:+.6} ({:.4} roots per prime); rational tail {:+.6}",
            side_json[0]["alpha_tail"].as_f64().unwrap_or(0.0),
            side_json[0]["roots_per_prime"].as_f64().unwrap_or(0.0),
            side_json[1]["alpha_tail"].as_f64().unwrap_or(0.0)
        );
        summaries.push(json!({
            "instance": instance,
            "path": path.to_string_lossy(),
            "degree": degree,
            "n_digits": n_digits,
            "algebraic": side_json[0],
            "rational": side_json[1],
        }));
    }
    let stem = out.display().to_string();
    let stem = stem.strip_suffix(".json").unwrap_or(&stem).to_string();
    let csv_path = PathBuf::from(format!("{stem}.csv"));
    write_text(&csv_path, &csv)?;
    let result = json!({
        "schema_version": RESULT_V1,
        "ts_ms": metrics::now_ms(),
        "from": from,
        "bound": bound,
        "bin_width": width,
        "method": "projective-root-count",
        "definition": "docs/baseline_calibration.md",
        "primes": primes.len(),
        "threads": threads,
        "duration_ms": duration_ms,
        "polynomials": summaries,
        "tables": [{
            "name": "root_tail",
            "path": csv_path.to_string_lossy(),
            "columns": ["instance", "side", "bin", "ln_p_lo", "ln_p_hi", "primes", "mean_ln_p", "roots", "eta", "eta_ln_p", "eta_affine", "eta_zero"],
            "rows": polys.len() * 2 * sums.len(),
            "description": "per polynomial, side and bin of ln p over the primes in (from, bound]: the primes, the mean ln p, the sum of the projective root counts r_p, the sum of eta_p = (r_p p - p - 1)/(p^2 - 1), the sum of eta_p ln p, and the sum of eta_p^aff = (r_p^aff - 1)/(p - 1) over the affine roots alone, for a region whose b never reaches p, and the sum of 1/(p - 1) over the primes at which 0 is a root, which a region whose |a| never reaches p meets only at (0, 1)",
        }],
    });
    registry.validate_value(RESULT_V1, &result)?;
    let text = serde_json::to_string_pretty(&result).map_err(|err| err.to_string())?;
    write_text(&out, &(text + "\n"))?;
    println!("wrote {} and {} ({} primes, {} ms)", out.display(), csv_path.display(), primes.len(), duration_ms);
    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn side(c: &[i64]) -> Side {
        Side::new(&c.iter().map(|&x| BigInt::from(x)).collect::<Vec<_>>())
    }

    /// r_p by evaluating the form at every point of P^1(F_p).
    fn brute(c: &[i64], p: u64) -> u64 {
        let pi = p as i128;
        let value = |a: i128, b: i128| {
            let mut acc = 0i128;
            for (i, &ci) in c.iter().enumerate() {
                let mut term = ci as i128 % pi;
                for _ in 0..i {
                    term = term * a % pi;
                }
                for _ in i..c.len() - 1 {
                    term = term * b % pi;
                }
                acc = (acc + term) % pi;
            }
            acc.rem_euclid(pi)
        };
        let affine = (0..pi).filter(|&a| value(a, 1) == 0).count() as u64;
        affine + u64::from(value(1, 0) == 0)
    }

    #[test]
    fn projective_roots_match_evaluation_at_every_point() {
        let forms: [&[i64]; 5] = [
            &[-85186479, -38330143, 45773675, 567699, 7560],
            &[24, -50, 35, -10, 1],
            &[1, 0, 1],
            &[197150570, 63638117],
            &[5, 3, 0, 7 * 11 * 13],
        ];
        for c in forms {
            let s = side(c);
            for &p in sieve::primes_up_to(400).iter() {
                assert_eq!(s.projective_roots(p), brute(c, p), "c={c:?} p={p}");
            }
        }
        // The BigInt path agrees with the i128 one.
        let big = Side { coeffs: side(&[5, 3, 0, 1001]).coeffs, small: None };
        for &p in sieve::primes_up_to(400).iter() {
            assert_eq!(big.projective_roots(p), brute(&[5, 3, 0, 1001], p), "p={p}");
        }
    }

    #[test]
    fn eta_is_minus_the_alpha_term_over_ln_p_without_ramification() {
        // Murphy's term (1/(p − 1) − E_p) ln p with E_p from the exact lifting.
        let c = [-85186479i64, -38330143, 45773675, 567699, 7560];
        let f: Vec<BigInt> = c.iter().map(|&x| BigInt::from(x)).collect();
        let s = side(&c);
        for &p in sieve::primes_up_to(3000).iter().filter(|&&p| p > 2000) {
            let pf = p as f64;
            let term = 1.0 / (pf - 1.0) - alpha::expected_valuation(&f, p);
            let from_roots = -eta(s.projective_roots(p), p);
            assert!((term - from_roots).abs() < 1e-12, "p={p}: {term} vs {from_roots}");
        }
        // One root, as for a linear form: 1/p − 1/(p + 1), scaled.
        assert!((eta(1, 7) + 1.0 / 48.0).abs() < 1e-15);
        assert!((eta(0, 7) + 1.0 / 6.0).abs() < 1e-15);
        // A linear form whose leading coefficient p divides: its one root is at
        // infinity, which a region with b below p never meets.
        let g = side(&[5, 7 * 11]);
        assert_eq!(g.roots(7), (0, true, false));
        assert_eq!(g.roots(13), (1, false, false));
        assert_eq!(g.roots(5), (1, false, true));
        assert!((eta_affine(0, 7) + 1.0 / 6.0).abs() < 1e-15);
        assert_eq!(eta_affine(1, 13), 0.0);
    }
}
