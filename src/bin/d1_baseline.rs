//! d1-baseline: the empirical Dickman baseline for the D1 comparison.
//!
//! For every size X on a grid of ln X, sieve the consecutive integers in
//! [X − L/2, X + L/2) with the line sieve on f(x) = x + X at b = 1, decide
//! B-smoothness exactly for every candidate, and record, for each bound B,
//! the observed density beside three Dickman forms at u = ln X / ln B:
//! ρ(u), ρ(u) + ρ'(u)/ln B, and ρ(u) + γ ρ'(u)/ln B. Integers in a short
//! interval near X have no polynomial structure, so their smoothness is the
//! quantity Murphy's α heuristic compares a polynomial value against.
//! docs/baseline_calibration.md records the definitions and the use.

use num_bigint::{BigInt, BigUint};
use num_traits::One;
use primness::dickman::{self, Dickman};
use primness::hardware::{self, Sampler};
use primness::linesieve::{self, FactorBase};
use primness::metrics::{self, Event, Sink};
use primness::schema::Registry;
use serde_json::json;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

pub const RESULT_V1: &str = "d1-baseline/1";
const SCALE: &str = "scaled-down";
const METHOD: &str = "consecutive-integers";
/// Euler–Mascheroni constant, the coefficient of ρ'(u)/ln B in the de Bruijn local density.
pub const EULER_GAMMA: f64 = 0.577_215_664_901_532_9;

struct Options {
    bounds: Vec<u64>,
    ln_x_min: f64,
    ln_x_max: f64,
    ln_x_step: f64,
    length: u64,
    chunk: u64,
    threads: usize,
    out: PathBuf,
    hw_sample: bool,
    hw_interval_ms: u64,
}

fn usage() -> ! {
    eprintln!(
        "Usage: d1-baseline [--bounds 2^14,2^16,2^18,2^20,2^24,2^28] [--ln-x MIN:MAX:STEP] \
         [--length 2^24] [--chunk 2^20] [--threads N] [--out FILE] [--no-hw-sample] [--hw-interval-ms N]"
    );
    std::process::exit(2);
}

fn parse_count(text: &str) -> Option<u64> {
    let text = text.trim();
    if let Some((base, exp)) = text.split_once('^') {
        let base: u64 = base.trim().parse().ok()?;
        let exp: u32 = exp.trim().parse().ok()?;
        return base.checked_pow(exp);
    }
    text.parse().ok()
}

fn parse_options() -> Options {
    let mut opts = Options {
        bounds: vec![1 << 14, 1 << 16, 1 << 18, 1 << 20, 1 << 24, 1 << 28],
        ln_x_min: 20.0,
        ln_x_max: 120.0,
        ln_x_step: 1.0,
        length: 1 << 24,
        chunk: 1 << 20,
        threads: std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1),
        out: PathBuf::from("data/results/baseline.consecutive.json"),
        hw_sample: true,
        hw_interval_ms: hardware::DEFAULT_INTERVAL_MS,
    };
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--bounds" => {
                let list = args.next().unwrap_or_else(|| usage());
                let mut bounds = Vec::new();
                for part in list.split(',') {
                    bounds.push(parse_count(part).filter(|b| *b >= 2).unwrap_or_else(|| usage()));
                }
                bounds.sort_unstable();
                bounds.dedup();
                if bounds.is_empty() {
                    usage();
                }
                opts.bounds = bounds;
            }
            "--ln-x" => {
                let spec = args.next().unwrap_or_else(|| usage());
                let parts: Vec<f64> = spec.split(':').map(|s| s.trim().parse().unwrap_or(f64::NAN)).collect();
                if parts.len() != 3 || parts.iter().any(|v| !v.is_finite()) || parts[2] <= 0.0 || parts[0] > parts[1] {
                    usage();
                }
                opts.ln_x_min = parts[0];
                opts.ln_x_max = parts[1];
                opts.ln_x_step = parts[2];
            }
            "--length" => opts.length = args.next().and_then(|v| parse_count(&v)).filter(|n| *n >= 2).unwrap_or_else(|| usage()),
            "--chunk" => opts.chunk = args.next().and_then(|v| parse_count(&v)).filter(|n| *n >= 1).unwrap_or_else(|| usage()),
            "--threads" => opts.threads = args.next().and_then(|v| v.parse().ok()).filter(|n| *n >= 1).unwrap_or_else(|| usage()),
            "--out" => opts.out = PathBuf::from(args.next().unwrap_or_else(|| usage())),
            "--no-hw-sample" => opts.hw_sample = false,
            "--hw-interval-ms" => opts.hw_interval_ms = args.next().and_then(|v| v.parse().ok()).filter(|n| *n >= 100).unwrap_or_else(|| usage()),
            other => {
                eprintln!("Unknown option {other}");
                usage();
            }
        }
    }
    opts
}

/// The integer nearest e^t, from a 53-bit mantissa and a power of two.
fn integer_near_exp(t: f64) -> BigUint {
    let e = t / std::f64::consts::LN_2;
    if e < 63.0 {
        return BigUint::from(t.exp().round() as u64);
    }
    let shift = e.floor() as u64 - 52;
    let mantissa = (e - shift as f64).exp2().round() as u64;
    BigUint::from(mantissa) << shift
}

struct SizeTally {
    smooth: Vec<u64>,
    candidates: u64,
}

/// Sieve the positions [a_lo, a_lo + len) of f(x) = x + X at b = 1, where
/// position j holds the integer X + a_lo + j, and count the smooth ones per bound.
fn sieve_chunk(fb: &FactorBase, x: &BigUint, a_lo: i64, len: usize, ln_x: f64, slack: f64, bounds: &[u64]) -> SizeTally {
    let base = (BigInt::from(x.clone()) + BigInt::from(a_lo))
        .to_biguint()
        .expect("the interval stays above zero");
    let mut acc = vec![0u16; len];
    linesieve::sieve_range(fb, 1, a_lo, &mut acc);
    let threshold = ln_x - slack;
    let candidate: Vec<bool> = acc.iter().map(|&s| s as f64 / linesieve::LOG_SCALE >= threshold).collect();
    let hits = linesieve::resieve_range(fb, 1, a_lo, &candidate);
    let mut tally = SizeTally { smooth: vec![0; bounds.len()], candidates: 0 };
    let mut k = 0usize;
    let mut ps = Vec::new();
    for (j, &is_candidate) in candidate.iter().enumerate() {
        if !is_candidate {
            continue;
        }
        tally.candidates += 1;
        ps.clear();
        while k < hits.len() && (hits[k].0 as usize) < j {
            k += 1;
        }
        while k < hits.len() && hits[k].0 as usize == j {
            ps.push(hits[k].1);
            k += 1;
        }
        let value = &base + BigUint::from(j as u64);
        if let Some(lpf) = linesieve::verify(&value, &ps) {
            for (i, &b) in bounds.iter().enumerate() {
                if lpf <= b {
                    tally.smooth[i] += 1;
                }
            }
        }
    }
    tally
}

fn write_text(path: &Path, text: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("{}: {err}", parent.display()))?;
    }
    fs::write(path, text).map_err(|err| format!("{}: {err}", path.display()))
}

fn event(run_id: &str, kind: &str, b: u64, evaluated: u64, smooth: u64, elapsed_ms: u64, rate_per_s: f64) -> Event {
    Event {
        schema_version: primness::schema::V1.into(),
        ts_ms: metrics::now_ms(),
        run_id: run_id.to_string(),
        source: "d1-baseline".into(),
        kind: kind.into(),
        scale: SCALE.into(),
        instance: "baseline-consecutive".into(),
        b,
        evaluated,
        smooth,
        elapsed_ms,
        rate_per_s,
        residues: String::new(),
    }
}

fn run(opts: &Options) -> Result<(), String> {
    let registry = Registry::from_env()?;
    let dickman = Dickman::new();
    let bounds = &opts.bounds;
    let b_top = *bounds.last().expect("bounds are non-empty");
    let ln_bounds: Vec<f64> = bounds.iter().map(|&b| (b as f64).ln()).collect();
    let slack = linesieve::default_slack(b_top);
    let length = opts.length;
    let half = length / 2;
    let chunk = opts.chunk.min(length) as usize;
    let threads = opts.threads.max(1);

    let mut sizes = Vec::new();
    let mut t = opts.ln_x_min;
    while t <= opts.ln_x_max + 1e-9 {
        let x = integer_near_exp(t);
        // The interval must stay above 1 and far from the sieve's own size.
        if x > BigUint::from(2 * length) {
            sizes.push(x);
        }
        t += opts.ln_x_step;
    }
    if sizes.is_empty() {
        return Err("no size on the grid exceeds twice the interval length".into());
    }

    let run_id = format!("d1b-{}-{}", std::process::id(), metrics::now_ms());
    let mut sink = Sink::from_env(&run_id);
    sink.emit(&event(&run_id, "start", b_top, 0, 0, 0, 0.0));
    let started = Instant::now();
    let sampler = if opts.hw_sample {
        match Sampler::start(&run_id, opts.hw_interval_ms) {
            Ok(sampler) => {
                println!("hardware trace: {} every {} ms", sampler.path.display(), opts.hw_interval_ms);
                Some(sampler)
            }
            Err(err) => {
                eprintln!("hardware sampling unavailable: {err}");
                None
            }
        }
    } else {
        None
    };
    println!(
        "baseline: {} sizes, ln X {}..{} step {}, {} consecutive integers each, bounds {:?}, slack {:.2}, {} threads",
        sizes.len(),
        opts.ln_x_min,
        opts.ln_x_max,
        opts.ln_x_step,
        length,
        bounds,
        slack,
        threads
    );

    let mut results = Vec::new();
    let mut points = String::from(
        "ln_x,x_digits,b,u,count,smooth,observed,observed_stderr,rho,local,debruijn_local,ratio_rho,ratio_local,ratio_debruijn\n",
    );
    let mut factor_base_primes = 0usize;
    let mut total_evaluated = 0u64;
    let mut total_smooth_top = 0u64;
    for (idx, x) in sizes.iter().enumerate() {
        let size_started = Instant::now();
        let f = [BigInt::from(x.clone()), BigInt::one()];
        let fb = FactorBase::build(&f, b_top, threads);
        factor_base_primes = fb.len();
        let ln_x = dickman::ln_biguint(x);
        let a_lo0 = -(half as i64);
        let chunks: Vec<(i64, usize)> = (0..length)
            .step_by(chunk)
            .map(|offset| (a_lo0 + offset as i64, chunk.min((length - offset) as usize)))
            .collect();
        let tallies: Vec<SizeTally> = std::thread::scope(|scope| {
            let per_thread = chunks.len().div_ceil(threads).max(1);
            let handles: Vec<_> = chunks
                .chunks(per_thread)
                .map(|mine| {
                    let (fb, x, bounds) = (&fb, x, bounds.as_slice());
                    scope.spawn(move || {
                        let mut acc = SizeTally { smooth: vec![0; bounds.len()], candidates: 0 };
                        for &(a_lo, len) in mine {
                            let t = sieve_chunk(fb, x, a_lo, len, ln_x, slack, bounds);
                            for i in 0..bounds.len() {
                                acc.smooth[i] += t.smooth[i];
                            }
                            acc.candidates += t.candidates;
                        }
                        acc
                    })
                })
                .collect();
            handles.into_iter().map(|h| h.join().expect("sieve thread")).collect()
        });
        let mut smooth = vec![0u64; bounds.len()];
        let mut candidates = 0u64;
        for t in &tallies {
            for i in 0..bounds.len() {
                smooth[i] += t.smooth[i];
            }
            candidates += t.candidates;
        }
        total_evaluated += length;
        total_smooth_top += smooth[bounds.len() - 1];
        let x_digits = x.to_string().len();
        let mut per_bound = Vec::new();
        for (i, &b) in bounds.iter().enumerate() {
            let u = ln_x / ln_bounds[i];
            let observed = smooth[i] as f64 / length as f64;
            let stderr = (observed * (1.0 - observed) / length as f64).sqrt();
            let rho = dickman.rho(u);
            let local = dickman.local_density(u, ln_bounds[i]);
            let debruijn = (rho + EULER_GAMMA * dickman.rho_prime(u) / ln_bounds[i]).max(0.0);
            let ratio = |pred: f64| if pred > 0.0 { observed / pred } else { f64::NAN };
            per_bound.push(json!({
                "b": b, "u": u, "smooth": smooth[i], "observed": observed, "observed_stderr": stderr,
                "rho": rho, "local": local, "debruijn_local": debruijn,
            }));
            points.push_str(&format!(
                "{ln_x:.6},{x_digits},{b},{u:.6},{length},{},{observed:.9},{stderr:.9},{rho:.9},{local:.9},{debruijn:.9},{:.6},{:.6},{:.6}\n",
                smooth[i],
                ratio(rho),
                ratio(local),
                ratio(debruijn)
            ));
        }
        results.push(json!({
            "x": x.to_string(), "ln_x": ln_x, "x_digits": x_digits, "count": length, "bounds": per_bound,
        }));
        let secs = size_started.elapsed().as_secs_f64();
        println!(
            "[{}/{}] ln X = {ln_x:.3} ({x_digits} digits): {} candidates, smooth at top bound {} ({:.3e}), {:.1} s",
            idx + 1,
            sizes.len(),
            candidates,
            smooth[bounds.len() - 1],
            smooth[bounds.len() - 1] as f64 / length as f64,
            secs
        );
        let elapsed_ms = started.elapsed().as_millis() as u64;
        sink.emit(&event(
            &run_id,
            "sample",
            b_top,
            total_evaluated,
            total_smooth_top,
            elapsed_ms,
            length as f64 / secs.max(1e-9),
        ));
    }
    let elapsed_ms = started.elapsed().as_millis() as u64;
    sink.emit(&event(
        &run_id,
        "finish",
        b_top,
        total_evaluated,
        total_smooth_top,
        elapsed_ms,
        total_evaluated as f64 / (elapsed_ms as f64 / 1000.0).max(1e-9),
    ));

    let hardware = sampler.and_then(|sampler| {
        let path = sampler.path.clone();
        match sampler.stop() {
            Ok(summary) => {
                println!(
                    "hardware: {} samples over {} ms; cpu active mean {:.2}, memory used mean {:.2}, cpu temp max {:.1} C",
                    summary.samples, summary.duration_ms, summary.cpu_active_ratio.mean, summary.memory_used_ratio.mean, summary.cpu_temp_c.max
                );
                Some(json!({"schema_version": hardware::V1, "samples_file": path.to_string_lossy(), "summary": summary}))
            }
            Err(err) => {
                eprintln!("hardware sampling: {err}");
                None
            }
        }
    });

    let stem = opts.out.display().to_string();
    let stem = stem.strip_suffix(".json").unwrap_or(&stem).to_string();
    let points_path = PathBuf::from(format!("{stem}.points.csv"));
    write_text(&points_path, &points)?;
    println!("wrote {}", points_path.display());
    let header = points.lines().next().unwrap_or("");
    let tables = vec![json!({
        "name": "points",
        "path": points_path.to_string_lossy(),
        "columns": header.split(',').collect::<Vec<_>>(),
        "rows": points.lines().count().saturating_sub(1),
        "description": "per size X and bound B: u = ln X / ln B, count of consecutive integers, smooth count, observed density and its Poisson standard error, rho(u), the derivative form rho + rho'/ln B, the de Bruijn local form rho + gamma rho'/ln B, and observed over each",
    })];
    let mut result = json!({
        "schema_version": RESULT_V1,
        "run_id": run_id,
        "ts_ms": metrics::now_ms(),
        "elapsed_ms": elapsed_ms,
        "scale": SCALE,
        "method": METHOD,
        "bounds": bounds,
        "sizes": {"ln_x_min": opts.ln_x_min, "ln_x_max": opts.ln_x_max, "ln_x_step": opts.ln_x_step, "count": sizes.len()},
        "interval_length": length,
        "chunk": chunk,
        "threads": threads,
        "dickman": {"method": dickman::METHOD, "steps_per_unit": dickman::STEPS_PER_UNIT, "u_max": dickman::U_MAX},
        "euler_gamma": EULER_GAMMA,
        "smoothness": {"method": "line-sieve", "slack_ln": slack, "factor_base_primes": factor_base_primes},
        "results": results,
        "tables": tables,
    });
    if let Some(hardware) = hardware {
        result["hardware"] = hardware;
    }
    registry.validate_value(RESULT_V1, &result)?;
    let text = serde_json::to_string_pretty(&result).map_err(|err| err.to_string())?;
    write_text(&opts.out, &(text + "\n"))?;
    println!(
        "wrote {} ({} sizes, {} integers, {:.1} s)",
        opts.out.display(),
        sizes.len(),
        total_evaluated,
        elapsed_ms as f64 / 1000.0
    );
    Ok(())
}

fn main() {
    let opts = parse_options();
    if let Err(err) = run(&opts) {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_traits::Zero;
    use primness::sieve::{largest_prime_factor_if_smooth, primes_up_to};

    #[test]
    fn integer_near_exp_is_close() {
        // Rounding to an integer moves ln x by about 1/x; above e^20 that is below 1e-8.
        for &t in &[20.0, 44.3, 90.7, 120.0] {
            let x = integer_near_exp(t);
            assert!((dickman::ln_biguint(&x) - t).abs() < 1e-8, "t={t}");
        }
        assert_eq!(integer_near_exp(5.0), BigUint::from(148u32));
        assert_eq!(integer_near_exp(0.0), BigUint::one());
        assert!(!integer_near_exp(200.0).is_zero());
    }

    #[test]
    fn chunk_counts_agree_with_trial_division() {
        let bounds = [1u64 << 8, 1 << 10, 1 << 12];
        let b_top = bounds[2];
        let x = integer_near_exp(30.0);
        let f = [BigInt::from(x.clone()), BigInt::one()];
        let fb = FactorBase::build(&f, b_top, 2);
        let ln_x = dickman::ln_biguint(&x);
        let slack = linesieve::default_slack(b_top);
        let primes = primes_up_to(b_top);
        // Three chunks at different offsets, as the run splits an interval.
        for &(a_lo, len) in &[(-30_000i64, 20_000usize), (-10_000, 10_000), (0, 15_000)] {
            let t = sieve_chunk(&fb, &x, a_lo, len, ln_x, slack, &bounds);
            let base = (BigInt::from(x.clone()) + BigInt::from(a_lo)).to_biguint().unwrap();
            let mut expected = vec![0u64; bounds.len()];
            for j in 0..len {
                let v = &base + BigUint::from(j as u64);
                if let Some(lpf) = largest_prime_factor_if_smooth(&v, &primes, b_top) {
                    for (i, &b) in bounds.iter().enumerate() {
                        if lpf <= b {
                            expected[i] += 1;
                        }
                    }
                }
            }
            assert_eq!(t.smooth, expected, "a_lo={a_lo}");
            assert!(expected[2] > 0);
        }
    }
}
