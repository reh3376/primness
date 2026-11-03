// d1_measure: independent smoothness measurement under Obstacle D / D1.
//
// Instrument path. A CADO or msieve polynomial file is parsed into
// `instrument-polynomial/1`. The homogeneous algebraic value F(a, b) and the
// rational value G(a, b) are evaluated in BigInt over the rectangle
// a in [-A, A], b in [1, B_MAX] with gcd(a, b) = 1. Each value is tested for
// B-smoothness by trial division over a sieved prime list, and the observed
// density is compared with the Dickman–de Bruijn prediction, the mean over
// the region of ρ(ln|F| / ln B). No Murphy α is applied. The run is labeled
// `scaled-down`: `full` needs the protocol's region for the digit length,
// which docs/d1_falsifiable_prediction.md has not fixed.
//
// Toy path. A file of plain integer coefficients (examples/demo.poly) keeps
// the earlier u64 demo so the bench has something to show. It is `toy`.

use num_traits::Zero;
use primness::dickman::{self, Dickman};
use primness::eval::{coprime, Homogeneous};
use primness::hardware::{self, Sampler};
use primness::metrics::{self, Event, Sink};
use primness::poly::{self, Source};
use primness::schema::Registry;
use primness::sieve::{largest_prime_factor_if_smooth, primes_up_to};
use serde_json::json;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

pub const RESULT_V1: &str = "d1-measurement/1";
const SCALE: &str = "scaled-down";

struct Options {
    poly_path: String,
    region: Option<i64>,
    a_max: i64,
    b_max: i64,
    bounds: Vec<u64>,
    instance: Option<String>,
    source: Source,
    out: Option<PathBuf>,
    threads: usize,
    hw_sample: bool,
    hw_interval_ms: u64,
}

fn usage() -> ! {
    eprintln!(
        "Usage: d1-measure [--a-max A] [--b-max B] [--bounds 2^14,2^16,2^18,2^20] \
         [--instance ID] [--source cado-nfs|msieve|yafu|unspecified] [--out FILE] \
         [--threads N] [--no-hw-sample] [--hw-interval-ms N] [--region N] <poly-file>"
    );
    std::process::exit(2);
}

fn parse_bound(text: &str) -> Option<u64> {
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
        poly_path: String::new(),
        region: None,
        a_max: 500,
        b_max: 250,
        bounds: vec![1 << 14, 1 << 16, 1 << 18, 1 << 20],
        instance: None,
        source: Source::Unspecified,
        out: None,
        threads: std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1),
        hw_sample: true,
        hw_interval_ms: hardware::DEFAULT_INTERVAL_MS,
    };
    let mut poly_path = None;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--region" => {
                opts.region = Some(
                    args.next()
                        .and_then(|v| v.parse().ok())
                        .unwrap_or_else(|| usage()),
                );
            }
            "--a-max" => {
                opts.a_max = args
                    .next()
                    .and_then(|v| v.parse().ok())
                    .filter(|n| *n >= 1)
                    .unwrap_or_else(|| usage());
            }
            "--b-max" => {
                opts.b_max = args
                    .next()
                    .and_then(|v| v.parse().ok())
                    .filter(|n| *n >= 1)
                    .unwrap_or_else(|| usage());
            }
            "--bounds" => {
                let list = args.next().unwrap_or_else(|| usage());
                let mut bounds = Vec::new();
                for part in list.split(',') {
                    let bound = parse_bound(part).filter(|b| *b >= 2).unwrap_or_else(|| usage());
                    bounds.push(bound);
                }
                bounds.sort_unstable();
                bounds.dedup();
                if bounds.is_empty() {
                    usage();
                }
                opts.bounds = bounds;
            }
            "--instance" => opts.instance = Some(args.next().unwrap_or_else(|| usage())),
            "--source" => {
                let text = args.next().unwrap_or_else(|| usage());
                opts.source = match text.as_str() {
                    "cado-nfs" => Source::CadoNfs,
                    "msieve" => Source::Msieve,
                    "yafu" => Source::Yafu,
                    "unspecified" => Source::Unspecified,
                    _ => usage(),
                };
            }
            "--out" => opts.out = Some(PathBuf::from(args.next().unwrap_or_else(|| usage()))),
            "--threads" => {
                opts.threads = args
                    .next()
                    .and_then(|v| v.parse().ok())
                    .filter(|n| *n >= 1)
                    .unwrap_or_else(|| usage());
            }
            "--no-hw-sample" => opts.hw_sample = false,
            "--hw-interval-ms" => {
                opts.hw_interval_ms = args
                    .next()
                    .and_then(|v| v.parse().ok())
                    .filter(|n| *n >= 100)
                    .unwrap_or_else(|| usage());
            }
            other if other.starts_with("--") => {
                eprintln!("Unknown option {other}");
                usage();
            }
            _ => poly_path = Some(arg),
        }
    }
    opts.poly_path = poly_path.unwrap_or_else(|| usage());
    opts
}

fn main() {
    let opts = parse_options();
    let text = match fs::read_to_string(&opts.poly_path) {
        Ok(text) => text,
        Err(err) => {
            eprintln!("Failed to read {}: {err}", opts.poly_path);
            std::process::exit(1);
        }
    };
    match poly::parse_instrument(&text) {
        Ok(poly) => {
            if let Err(err) = run_instrument(poly, &opts) {
                eprintln!("{err}");
                std::process::exit(1);
            }
        }
        Err(poly::Error::NotInstrument) => run_toy(&text, &opts),
        Err(err) => {
            eprintln!("Failed to parse {}: {err}", opts.poly_path);
            std::process::exit(1);
        }
    }
}

// ---------------------------------------------------------------------------
// Instrument path

/// Counts for one slice of the region. Vectors are indexed like `bounds`.
struct Tally {
    evaluated: u64,
    skipped_zero: u64,
    alg_smooth: Vec<u64>,
    rat_smooth: Vec<u64>,
    both: Vec<u64>,
    alg_pred: Vec<f64>,
    rat_pred: Vec<f64>,
    alg_u: Vec<f64>,
    rat_u: Vec<f64>,
    residues: String,
}

impl Tally {
    fn new(k: usize) -> Self {
        Self {
            evaluated: 0,
            skipped_zero: 0,
            alg_smooth: vec![0; k],
            rat_smooth: vec![0; k],
            both: vec![0; k],
            alg_pred: vec![0.0; k],
            rat_pred: vec![0.0; k],
            alg_u: vec![0.0; k],
            rat_u: vec![0.0; k],
            residues: String::new(),
        }
    }

    fn merge(&mut self, other: &Tally) {
        self.evaluated += other.evaluated;
        self.skipped_zero += other.skipped_zero;
        for i in 0..self.alg_smooth.len() {
            self.alg_smooth[i] += other.alg_smooth[i];
            self.rat_smooth[i] += other.rat_smooth[i];
            self.both[i] += other.both[i];
            self.alg_pred[i] += other.alg_pred[i];
            self.rat_pred[i] += other.rat_pred[i];
            self.alg_u[i] += other.alg_u[i];
            self.rat_u[i] += other.rat_u[i];
        }
        self.residues.push_str(&other.residues);
        if self.residues.len() > 64 {
            let drop = self.residues.len() - 64;
            self.residues.drain(..drop);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn measure_rows(
    rows: &[i64],
    a_max: i64,
    alg: &Homogeneous,
    rat: &Homogeneous,
    primes: &[u64],
    dickman: &Dickman,
    bounds: &[u64],
) -> Tally {
    let b_top = *bounds.last().expect("bounds are non-empty");
    let top = bounds.len() - 1;
    let ln_bounds: Vec<f64> = bounds.iter().map(|&b| (b as f64).ln()).collect();
    let mut tally = Tally::new(bounds.len());
    for &b in rows {
        let alg_pows = alg.powers_of_b(b);
        let rat_pows = rat.powers_of_b(b);
        for a in -a_max..=a_max {
            if !coprime(a, b) {
                continue;
            }
            let f = alg.eval_with_powers(a, &alg_pows);
            let g = rat.eval_with_powers(a, &rat_pows);
            if f.is_zero() || g.is_zero() {
                tally.skipped_zero += 1;
                continue;
            }
            let (f_mag, g_mag) = (f.magnitude(), g.magnitude());
            let (ln_f, ln_g) = (dickman::ln_biguint(f_mag), dickman::ln_biguint(g_mag));
            let f_lpf = largest_prime_factor_if_smooth(f_mag, primes, b_top);
            let g_lpf = largest_prime_factor_if_smooth(g_mag, primes, b_top);
            tally.evaluated += 1;
            for (i, &bound) in bounds.iter().enumerate() {
                let f_smooth = f_lpf.is_some_and(|l| l <= bound);
                let g_smooth = g_lpf.is_some_and(|l| l <= bound);
                if f_smooth {
                    tally.alg_smooth[i] += 1;
                }
                if g_smooth {
                    tally.rat_smooth[i] += 1;
                }
                if f_smooth && g_smooth {
                    tally.both[i] += 1;
                }
                let (u_f, u_g) = (ln_f / ln_bounds[i], ln_g / ln_bounds[i]);
                tally.alg_pred[i] += dickman.rho(u_f);
                tally.rat_pred[i] += dickman.rho(u_g);
                tally.alg_u[i] += u_f;
                tally.rat_u[i] += u_g;
            }
            tally
                .residues
                .push(if f_lpf.is_some_and(|l| l <= bounds[top]) { '1' } else { '0' });
            if tally.residues.len() > 64 {
                tally.residues.drain(..1);
            }
        }
    }
    tally
}

fn run_instrument(poly: poly::InstrumentPolynomial, opts: &Options) -> Result<(), String> {
    let (a_max, b_max) = match opts.region {
        Some(n) => (n.max(1), (n / 2).max(1)),
        None => (opts.a_max, opts.b_max),
    };
    let registry = Registry::from_env()?;
    let record = poly.record(opts.source);
    registry.validate_value(poly::V1, &record)?;
    let instance = opts.instance.clone().unwrap_or_else(|| {
        Path::new(&opts.poly_path)
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| opts.poly_path.clone())
    });
    let bounds = opts.bounds.clone();
    let b_top = *bounds.last().expect("bounds are non-empty");
    let top = bounds.len() - 1;
    let started = Instant::now();

    println!("Primness D1 measurement harness");
    println!("Loaded polynomial file: {}", opts.poly_path);
    println!(
        "{} format={} degree={} n_digits={} source={}",
        poly::V1,
        poly.format.as_str(),
        poly.degree(),
        poly.n_digits(),
        opts.source.as_str()
    );
    println!("scale: {SCALE} (not the 40–80 digit protocol)");
    println!(
        "region: a in [{}, {}], b in [1, {}], gcd(a, b) = 1, zero values skipped",
        -a_max, a_max, b_max
    );
    println!(
        "bounds: {}",
        bounds
            .iter()
            .map(|b| b.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
    let primes = primes_up_to(b_top);
    println!(
        "primes up to {b_top}: {} (sieved in {} ms)",
        primes.len(),
        started.elapsed().as_millis()
    );
    let dickman = Dickman::new();
    let alg = Homogeneous::new(&poly.algebraic);
    let rat = Homogeneous::new(&poly.rational);

    let run_id = format!("d1m-{}-{}", std::process::id(), metrics::now_ms());
    let mut sink = Sink::from_env(&run_id);
    sink.emit(&event(&run_id, &instance, SCALE, "start", b_top, 0, 0, 0, 0.0, ""));
    // Hardware telemetry runs beside the measurement and never fails it.
    let sampler = if opts.hw_sample {
        match Sampler::start(&run_id, opts.hw_interval_ms) {
            Ok(sampler) => {
                println!(
                    "hardware trace: {} every {} ms",
                    sampler.path.display(),
                    opts.hw_interval_ms
                );
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

    let mut total = Tally::new(bounds.len());
    let rows: Vec<i64> = (1..=b_max).collect();
    let threads = opts.threads.max(1);
    for batch in rows.chunks(threads * 4) {
        let batch_started = Instant::now();
        let per_thread = batch.len().div_ceil(threads).max(1);
        let tallies: Vec<Tally> = std::thread::scope(|scope| {
            let handles: Vec<_> = batch
                .chunks(per_thread)
                .map(|rows| {
                    let (alg, rat, primes, dickman, bounds) =
                        (&alg, &rat, &primes, &dickman, &bounds);
                    scope.spawn(move || measure_rows(rows, a_max, alg, rat, primes, dickman, bounds))
                })
                .collect();
            handles
                .into_iter()
                .map(|handle| handle.join().expect("measurement thread panicked"))
                .collect()
        });
        let mut batch_evaluated = 0u64;
        for tally in &tallies {
            batch_evaluated += tally.evaluated;
            total.merge(tally);
        }
        let secs = batch_started.elapsed().as_secs_f64().max(1e-9);
        sink.emit(&event(
            &run_id,
            &instance,
            SCALE,
            "sample",
            b_top,
            total.evaluated,
            total.alg_smooth[top],
            started.elapsed().as_millis() as u64,
            batch_evaluated as f64 / secs,
            &total.residues,
        ));
    }
    let elapsed_ms = started.elapsed().as_millis() as u64;
    let rate = total.evaluated as f64 / (elapsed_ms as f64 / 1000.0).max(1e-9);
    sink.emit(&event(
        &run_id,
        &instance,
        SCALE,
        "finish",
        b_top,
        total.evaluated,
        total.alg_smooth[top],
        elapsed_ms,
        rate,
        &total.residues,
    ));

    println!(
        "evaluated {} coprime pairs, skipped {} zero values, {} ms, {:.0} values/s",
        total.evaluated, total.skipped_zero, elapsed_ms, rate
    );
    println!(
        "{:>9} | {:>8} {:>8} {:>8} {:>8} {:>6} | {:>8} {:>8} {:>8} {:>8} {:>6} | {:>8}",
        "B", "alg_sm", "alg_obs", "alg_pred", "alg_gap", "u_alg", "rat_sm", "rat_obs",
        "rat_pred", "rat_gap", "u_rat", "both_sm"
    );
    let n = total.evaluated as f64;
    // (observed, binomial standard error, predicted, gap, mean u)
    let side = |smooth: u64, pred: f64, u: f64| -> (f64, f64, f64, f64, f64) {
        if n == 0.0 {
            (0.0, 0.0, 0.0, 0.0, 0.0)
        } else {
            let observed = smooth as f64 / n;
            let stderr = (observed * (1.0 - observed) / n).sqrt();
            let predicted = pred / n;
            (observed, stderr, predicted, observed - predicted, u / n)
        }
    };
    let mut results = Vec::new();
    for (i, &b) in bounds.iter().enumerate() {
        let (ao, ae, ap, ag, au) = side(total.alg_smooth[i], total.alg_pred[i], total.alg_u[i]);
        let (ro, re, rp, rg, ru) = side(total.rat_smooth[i], total.rat_pred[i], total.rat_u[i]);
        println!(
            "{b:>9} | {:>8} {ao:>8.5} {ap:>8.5} {ag:>+8.5} {au:>6.2} | {:>8} {ro:>8.5} {rp:>8.5} {rg:>+8.5} {ru:>6.2} | {:>8}",
            total.alg_smooth[i], total.rat_smooth[i], total.both[i]
        );
        println!(
            "{:>9} | observed stderr {ae:.5} (gap = {:+.1} stderr)          | observed stderr {re:.5} (gap = {:+.1} stderr)",
            "",
            if ae > 0.0 { ag / ae } else { 0.0 },
            if re > 0.0 { rg / re } else { 0.0 }
        );
        results.push(json!({
            "b": b,
            "algebraic": {"smooth": total.alg_smooth[i], "observed": ao, "observed_stderr": ae, "predicted": ap, "gap": ag, "mean_u": au},
            "rational": {"smooth": total.rat_smooth[i], "observed": ro, "observed_stderr": re, "predicted": rp, "gap": rg, "mean_u": ru},
            "both_smooth": total.both[i],
        }));
    }
    println!("alpha_applied: false (no Murphy α definition is recorded in this repository)");

    let hardware = sampler.and_then(|sampler| {
        let path = sampler.path.clone();
        match sampler.stop() {
            Ok(summary) => {
                println!(
                    "hardware: {} samples over {} ms; cpu active mean {:.2}, gpu device util mean {:.1}%, memory used mean {:.2}, gpu share of memory mean {:.3}, cpu temp max {:.1} C",
                    summary.samples,
                    summary.duration_ms,
                    summary.cpu_active_ratio.mean,
                    summary.gpu_device_utilization_pct.mean,
                    summary.memory_used_ratio.mean,
                    summary.gpu_share_of_total.mean,
                    summary.cpu_temp_c.max
                );
                Some(json!({
                    "schema_version": hardware::V1,
                    "samples_file": path.to_string_lossy(),
                    "summary": summary,
                }))
            }
            Err(err) => {
                eprintln!("hardware sampling: {err}");
                None
            }
        }
    });

    let mut result = json!({
        "schema_version": RESULT_V1,
        "run_id": run_id,
        "ts_ms": metrics::now_ms(),
        "elapsed_ms": elapsed_ms,
        "instance": instance,
        "n": poly.n.to_string(),
        "n_digits": poly.n_digits(),
        "polynomial": record,
        "region": {"a_min": -a_max, "a_max": a_max, "b_min": 1, "b_max": b_max, "coprime_only": true, "skip_zero": true},
        "scale": SCALE,
        "bounds": bounds,
        "evaluated": total.evaluated,
        "skipped_zero": total.skipped_zero,
        "threads": threads,
        "dickman": {"method": dickman::METHOD, "steps_per_unit": dickman::STEPS_PER_UNIT, "u_max": dickman::U_MAX},
        "alpha_applied": false,
        "results": results,
    });
    if let Some(hardware) = hardware {
        result["hardware"] = hardware;
    }
    registry.validate_value(RESULT_V1, &result)?;
    let out = opts
        .out
        .clone()
        .unwrap_or_else(|| PathBuf::from("data/results").join(format!("{instance}-{run_id}.json")));
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("{}: {err}", parent.display()))?;
    }
    let text = serde_json::to_string_pretty(&result).map_err(|err| err.to_string())?;
    fs::write(&out, text + "\n").map_err(|err| format!("{}: {err}", out.display()))?;
    println!("wrote {}", out.display());
    Ok(())
}

// ---------------------------------------------------------------------------
// Toy path

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

fn run_toy(text: &str, opts: &Options) {
    let coeffs = parse_coeffs(text);
    let (a_max, b_max, bounds): (i64, i64, Vec<u64>) = match opts.region {
        Some(n) => {
            let n = n.clamp(2, 2000);
            (n, (n / 2).max(1), vec![32, 128, 512])
        }
        None => (10, 5, vec![10, 30, 100]),
    };

    println!("Primness D1 measurement harness");
    println!("Loaded polynomial file: {}", opts.poly_path);
    println!("scale: toy (not the 40–80 digit protocol)");

    let started = Instant::now();
    let run_id = format!("d1m-{}-{}", std::process::id(), metrics::now_ms());
    let instance = opts.poly_path.clone();
    let mut sink = Sink::from_env(&run_id);
    sink.emit(&event(&run_id, &instance, "toy", "start", 0, 0, 0, 0, 0.0, ""));

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
        println!("B={bound:<4}  observed density: {density:.3}  ({smooth}/{evaluated})");
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
        "toy",
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

#[allow(clippy::too_many_arguments)]
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
        "toy",
        "sample",
        bound,
        evaluated,
        smooth,
        started.elapsed().as_millis() as u64,
        rate,
        residues,
    ));
}

#[allow(clippy::too_many_arguments)]
fn event(
    run_id: &str,
    instance: &str,
    scale: &str,
    kind: &str,
    b: u64,
    evaluated: u64,
    smooth: u64,
    elapsed_ms: u64,
    rate_per_s: f64,
    residues: &str,
) -> Event {
    Event {
        schema_version: primness::schema::V1.into(),
        ts_ms: metrics::now_ms(),
        run_id: run_id.to_string(),
        source: "d1-measure".into(),
        kind: kind.into(),
        scale: scale.into(),
        instance: instance.into(),
        b,
        evaluated,
        smooth,
        elapsed_ms,
        rate_per_s,
        residues: residues.into(),
    }
}
