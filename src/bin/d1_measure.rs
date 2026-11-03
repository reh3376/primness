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

use num_bigint::BigUint;
use num_traits::Zero;
use primness::alpha;
use primness::dickman::{self, Dickman};
use primness::eval::{coprime, Homogeneous};
use primness::linesieve::{self, FactorBase};
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

pub const RESULT_V2: &str = "d1-measurement/2";
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
    dump: Option<PathBuf>,
    alpha_bound: u64,
    sieve: bool,
}

fn usage() -> ! {
    eprintln!(
        "Usage: d1-measure [--a-max A] [--b-max B] [--bounds 2^14,2^16,2^18,2^20] \
         [--instance ID] [--source cado-nfs|msieve|yafu|unspecified] [--out FILE] \
         [--threads N] [--no-hw-sample] [--hw-interval-ms N] [--dump FILE] [--alpha-bound N] [--method sieve|trial] [--region N] <poly-file>"
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
        dump: None,
        alpha_bound: alpha::DEFAULT_BOUND,
        sieve: true,
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
            "--dump" => opts.dump = Some(PathBuf::from(args.next().unwrap_or_else(|| usage()))),
            "--method" => {
                let text = args.next().unwrap_or_else(|| usage());
                opts.sieve = match text.as_str() {
                    "sieve" => true,
                    "trial" => false,
                    _ => usage(),
                };
            }
            "--alpha-bound" => {
                opts.alpha_bound = args
                    .next()
                    .and_then(|v| v.parse().ok())
                    .filter(|n| *n >= 2)
                    .unwrap_or_else(|| usage());
            }
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

/// u is binned in steps of U_BIN_WIDTH from 0 to U_BINS * U_BIN_WIDTH.
const U_BIN_WIDTH: f64 = 0.1;
const U_BINS: usize = 100;
/// log2 of the largest prime factor is binned 0..=LPF_BINS-1; the last bin
/// of each histogram counts values that were not smooth at the top bound.
const LPF_BINS: usize = 33;

#[derive(Clone, Copy, Default)]
struct Bin {
    count: u64,
    smooth: u64,
    rho_sum: f64,
    rho_alpha_sum: f64,
    local_sum: f64,
}

#[derive(Clone, Copy, Default)]
struct RowStat {
    evaluated: u64,
    alg_smooth: u64,
    rat_smooth: u64,
    both: u64,
    ln_f_sum: f64,
    ln_g_sum: f64,
}

/// Counts for one slice of the region. Outer vectors are indexed like `bounds`.
struct Tally {
    evaluated: u64,
    skipped_zero: u64,
    alg_smooth: Vec<u64>,
    rat_smooth: Vec<u64>,
    both: Vec<u64>,
    alg_pred: Vec<f64>,
    rat_pred: Vec<f64>,
    alg_pred_alpha: Vec<f64>,
    rat_pred_alpha: Vec<f64>,
    alg_pred_local: Vec<f64>,
    rat_pred_local: Vec<f64>,
    alg_u: Vec<f64>,
    rat_u: Vec<f64>,
    alg_bins: Vec<Vec<Bin>>,
    rat_bins: Vec<Vec<Bin>>,
    rows: std::collections::BTreeMap<i64, RowStat>,
    alg_lpf_hist: Vec<u64>,
    rat_lpf_hist: Vec<u64>,
    residues: String,
    dump: String,
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
            alg_pred_alpha: vec![0.0; k],
            rat_pred_alpha: vec![0.0; k],
            alg_pred_local: vec![0.0; k],
            rat_pred_local: vec![0.0; k],
            alg_u: vec![0.0; k],
            rat_u: vec![0.0; k],
            alg_bins: vec![vec![Bin::default(); U_BINS]; k],
            rat_bins: vec![vec![Bin::default(); U_BINS]; k],
            rows: std::collections::BTreeMap::new(),
            alg_lpf_hist: vec![0; LPF_BINS],
            rat_lpf_hist: vec![0; LPF_BINS],
            residues: String::new(),
            dump: String::new(),
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
            self.alg_pred_alpha[i] += other.alg_pred_alpha[i];
            self.rat_pred_alpha[i] += other.rat_pred_alpha[i];
            self.alg_pred_local[i] += other.alg_pred_local[i];
            self.rat_pred_local[i] += other.rat_pred_local[i];
            self.alg_u[i] += other.alg_u[i];
            self.rat_u[i] += other.rat_u[i];
            for j in 0..U_BINS {
                self.alg_bins[i][j].count += other.alg_bins[i][j].count;
                self.alg_bins[i][j].smooth += other.alg_bins[i][j].smooth;
                self.alg_bins[i][j].rho_sum += other.alg_bins[i][j].rho_sum;
                self.alg_bins[i][j].rho_alpha_sum += other.alg_bins[i][j].rho_alpha_sum;
                self.alg_bins[i][j].local_sum += other.alg_bins[i][j].local_sum;
                self.rat_bins[i][j].count += other.rat_bins[i][j].count;
                self.rat_bins[i][j].smooth += other.rat_bins[i][j].smooth;
                self.rat_bins[i][j].rho_sum += other.rat_bins[i][j].rho_sum;
                self.rat_bins[i][j].rho_alpha_sum += other.rat_bins[i][j].rho_alpha_sum;
                self.rat_bins[i][j].local_sum += other.rat_bins[i][j].local_sum;
            }
        }
        for (b, row) in &other.rows {
            let slot = self.rows.entry(*b).or_default();
            slot.evaluated += row.evaluated;
            slot.alg_smooth += row.alg_smooth;
            slot.rat_smooth += row.rat_smooth;
            slot.both += row.both;
            slot.ln_f_sum += row.ln_f_sum;
            slot.ln_g_sum += row.ln_g_sum;
        }
        for j in 0..LPF_BINS {
            self.alg_lpf_hist[j] += other.alg_lpf_hist[j];
            self.rat_lpf_hist[j] += other.rat_lpf_hist[j];
        }
        self.residues.push_str(&other.residues);
        if self.residues.len() > 64 {
            let drop = self.residues.len() - 64;
            self.residues.drain(..drop);
        }
    }
}

fn u_bin(u: f64) -> usize {
    ((u / U_BIN_WIDTH).floor().max(0.0) as usize).min(U_BINS - 1)
}

fn lpf_bin(lpf: Option<u64>) -> usize {
    match lpf {
        Some(l) => ((64 - l.max(1).leading_zeros()) as usize - 1).min(LPF_BINS - 2),
        None => LPF_BINS - 1,
    }
}

enum Method<'a> {
    Trial { primes: &'a [u64] },
    Sieve { alg_fb: &'a FactorBase, rat_fb: &'a FactorBase, slack: f64 },
}

struct ValueRow {
    a: i64,
    ln_f: f64,
    f_lpf: Option<u64>,
    ln_g: f64,
    g_lpf: Option<u64>,
}

/// Every coprime, non-zero position of row b with its logs and largest
/// prime factors (None when not smooth at the top bound).
fn row_values(
    method: &Method,
    b: i64,
    a_max: i64,
    alg: &Homogeneous,
    rat: &Homogeneous,
    b_top: u64,
    skipped_zero: &mut u64,
) -> Vec<ValueRow> {
    let alg_pows = alg.powers_of_b(b);
    let rat_pows = rat.powers_of_b(b);
    match method {
        Method::Trial { primes } => {
            let mut out = Vec::new();
            for a in -a_max..=a_max {
                if !coprime(a, b) {
                    continue;
                }
                let f = alg.eval_with_powers(a, &alg_pows);
                let g = rat.eval_with_powers(a, &rat_pows);
                if f.is_zero() || g.is_zero() {
                    *skipped_zero += 1;
                    continue;
                }
                let (f_mag, g_mag) = (f.magnitude(), g.magnitude());
                out.push(ValueRow {
                    a,
                    ln_f: dickman::ln_biguint(f_mag),
                    f_lpf: largest_prime_factor_if_smooth(f_mag, primes, b_top),
                    ln_g: dickman::ln_biguint(g_mag),
                    g_lpf: largest_prime_factor_if_smooth(g_mag, primes, b_top),
                });
            }
            out
        }
        Method::Sieve { alg_fb, rat_fb, slack } => {
            let len = (2 * a_max + 1) as usize;
            // (index, a, |F|, |G|, ln|F|, ln|G|) for the positions that count.
            let mut vals: Vec<(usize, i64, BigUint, BigUint, f64, f64)> = Vec::with_capacity(len);
            for j in 0..len {
                let a = j as i64 - a_max;
                if !coprime(a, b) {
                    continue;
                }
                let f = alg.eval_with_powers(a, &alg_pows);
                let g = rat.eval_with_powers(a, &rat_pows);
                if f.is_zero() || g.is_zero() {
                    *skipped_zero += 1;
                    continue;
                }
                let (f_mag, g_mag) = (f.magnitude().clone(), g.magnitude().clone());
                let (ln_f, ln_g) = (dickman::ln_biguint(&f_mag), dickman::ln_biguint(&g_mag));
                vals.push((j, a, f_mag, g_mag, ln_f, ln_g));
            }
            let mut acc_f = vec![0u16; len];
            let mut acc_g = vec![0u16; len];
            linesieve::sieve_row(alg_fb, b, a_max, &mut acc_f);
            linesieve::sieve_row(rat_fb, b, a_max, &mut acc_g);
            let mut cand_f = vec![false; len];
            let mut cand_g = vec![false; len];
            for (j, _, _, _, ln_f, ln_g) in &vals {
                cand_f[*j] = acc_f[*j] as f64 / linesieve::LOG_SCALE >= ln_f - slack;
                cand_g[*j] = acc_g[*j] as f64 / linesieve::LOG_SCALE >= ln_g - slack;
            }
            let hits_f = linesieve::resieve_row(alg_fb, b, a_max, &cand_f);
            let hits_g = linesieve::resieve_row(rat_fb, b, a_max, &cand_g);
            let (mut kf, mut kg) = (0usize, 0usize);
            let mut ps = Vec::new();
            let mut out = Vec::with_capacity(vals.len());
            for (j, a, f_mag, g_mag, ln_f, ln_g) in vals {
                let f_lpf = if cand_f[j] {
                    ps.clear();
                    while kf < hits_f.len() && (hits_f[kf].0 as usize) < j {
                        kf += 1;
                    }
                    while kf < hits_f.len() && hits_f[kf].0 as usize == j {
                        ps.push(hits_f[kf].1);
                        kf += 1;
                    }
                    linesieve::verify(&f_mag, &ps)
                } else {
                    None
                };
                let g_lpf = if cand_g[j] {
                    ps.clear();
                    while kg < hits_g.len() && (hits_g[kg].0 as usize) < j {
                        kg += 1;
                    }
                    while kg < hits_g.len() && hits_g[kg].0 as usize == j {
                        ps.push(hits_g[kg].1);
                        kg += 1;
                    }
                    linesieve::verify(&g_mag, &ps)
                } else {
                    None
                };
                out.push(ValueRow { a, ln_f, f_lpf, ln_g, g_lpf });
            }
            out
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn measure_rows(
    rows: &[i64],
    a_max: i64,
    alg: &Homogeneous,
    rat: &Homogeneous,
    method: &Method,
    dickman: &Dickman,
    bounds: &[u64],
    alphas: (f64, f64),
    dump: bool,
) -> Tally {
    let b_top = *bounds.last().expect("bounds are non-empty");
    let top = bounds.len() - 1;
    let ln_bounds: Vec<f64> = bounds.iter().map(|&b| (b as f64).ln()).collect();
    let mut tally = Tally::new(bounds.len());
    for &b in rows {
        let mut row = RowStat::default();
        let values = row_values(method, b, a_max, alg, rat, b_top, &mut tally.skipped_zero);
        for v in values {
            let (a, ln_f, ln_g, f_lpf, g_lpf) = (v.a, v.ln_f, v.ln_g, v.f_lpf, v.g_lpf);
            tally.evaluated += 1;
            row.evaluated += 1;
            row.ln_f_sum += ln_f;
            row.ln_g_sum += ln_g;
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
                let (rho_f, rho_g) = (dickman.rho(u_f), dickman.rho(u_g));
                let u_f_alpha = ((ln_f + alphas.0) / ln_bounds[i]).max(0.0);
                let u_g_alpha = ((ln_g + alphas.1) / ln_bounds[i]).max(0.0);
                let rho_f_alpha = dickman.rho(u_f_alpha);
                let rho_g_alpha = dickman.rho(u_g_alpha);
                let local_f = dickman.local_density(u_f_alpha, ln_bounds[i]);
                let local_g = dickman.local_density(u_g_alpha, ln_bounds[i]);
                tally.alg_pred[i] += rho_f;
                tally.rat_pred[i] += rho_g;
                tally.alg_pred_alpha[i] += rho_f_alpha;
                tally.rat_pred_alpha[i] += rho_g_alpha;
                tally.alg_pred_local[i] += local_f;
                tally.rat_pred_local[i] += local_g;
                tally.alg_u[i] += u_f;
                tally.rat_u[i] += u_g;
                let bin = &mut tally.alg_bins[i][u_bin(u_f)];
                bin.count += 1;
                bin.smooth += u64::from(f_smooth);
                bin.rho_sum += rho_f;
                bin.rho_alpha_sum += rho_f_alpha;
                bin.local_sum += local_f;
                let bin = &mut tally.rat_bins[i][u_bin(u_g)];
                bin.count += 1;
                bin.smooth += u64::from(g_smooth);
                bin.rho_sum += rho_g;
                bin.rho_alpha_sum += rho_g_alpha;
                bin.local_sum += local_g;
                if i == top {
                    row.alg_smooth += u64::from(f_smooth);
                    row.rat_smooth += u64::from(g_smooth);
                    row.both += u64::from(f_smooth && g_smooth);
                }
            }
            tally.alg_lpf_hist[lpf_bin(f_lpf)] += 1;
            tally.rat_lpf_hist[lpf_bin(g_lpf)] += 1;
            let f_top = f_lpf.is_some_and(|l| l <= b_top);
            tally.residues.push(if f_top { '1' } else { '0' });
            if tally.residues.len() > 64 {
                tally.residues.drain(..1);
            }
            if dump {
                use std::fmt::Write as _;
                let _ = writeln!(
                    tally.dump,
                    "{a},{b},{ln_f:.6},{},{ln_g:.6},{}",
                    f_lpf.unwrap_or(0),
                    g_lpf.unwrap_or(0)
                );
            }
        }
        tally.rows.insert(b, row);
    }
    tally
}

/// The size shift α such that the mean of ρ((ln|F| + α) / ln B) over the
/// binned values equals the observed density. It is the shift a
/// root-property correction would have to supply; it is not a Murphy α
/// computation, which needs the prime-by-prime definition.
fn implied_alpha(bins: &[Bin], ln_b: f64, observed: f64, dickman: &Dickman) -> Option<f64> {
    let total: u64 = bins.iter().map(|bin| bin.count).sum();
    if total == 0 || observed <= 0.0 {
        return None;
    }
    let mean_rho = |alpha: f64| -> f64 {
        bins.iter()
            .enumerate()
            .filter(|(_, bin)| bin.count > 0)
            .map(|(j, bin)| {
                let u_center = (j as f64 + 0.5) * U_BIN_WIDTH;
                // A shift below size 1 means certainly smooth, so u is floored at 0.
                bin.count as f64 * dickman.rho((u_center + alpha / ln_b).max(0.0))
            })
            .sum::<f64>()
            / total as f64
    };
    // mean_rho decreases as alpha increases.
    let (mut lo, mut hi) = (-40.0f64, 40.0f64);
    if mean_rho(lo) < observed || mean_rho(hi) > observed {
        return None;
    }
    for _ in 0..80 {
        let mid = 0.5 * (lo + hi);
        if mean_rho(mid) > observed {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Some(0.5 * (lo + hi))
}

fn write_text(path: &Path, text: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("{}: {err}", parent.display()))?;
    }
    fs::write(path, text).map_err(|err| format!("{}: {err}", path.display()))
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
    let dickman = Dickman::new();
    let alg = Homogeneous::new(&poly.algebraic);
    let rat = Homogeneous::new(&poly.rational);
    let threads = opts.threads.max(1);
    let mut primes: Vec<u64> = Vec::new();
    let mut factor_bases: Option<(FactorBase, FactorBase)> = None;
    let smoothness = if opts.sieve {
        let alg_fb = FactorBase::build(alg.coefficients(), b_top, threads);
        let rat_fb = FactorBase::build(rat.coefficients(), b_top, threads);
        let slack = linesieve::default_slack(b_top);
        println!(
            "line sieve: {} primes up to {b_top}, {} algebraic roots, {} rational roots, candidate slack {slack:.2} in ln (factor bases built in {} ms; docs/line_sieve.md)",
            alg_fb.len(),
            alg_fb.total_roots(),
            rat_fb.total_roots(),
            started.elapsed().as_millis()
        );
        let record = json!({"method": "line-sieve", "slack_ln": slack, "factor_base_primes": alg_fb.len()});
        factor_bases = Some((alg_fb, rat_fb));
        record
    } else {
        primes = primes_up_to(b_top);
        println!(
            "trial division: {} primes up to {b_top} (sieved in {} ms)",
            primes.len(),
            started.elapsed().as_millis()
        );
        json!({"method": "trial-division", "slack_ln": 0.0, "factor_base_primes": primes.len()})
    };
    let method = match &factor_bases {
        Some((alg_fb, rat_fb)) => Method::Sieve { alg_fb, rat_fb, slack: linesieve::default_slack(b_top) },
        None => Method::Trial { primes: &primes },
    };
    let alpha_f = alpha::alpha(alg.coefficients(), opts.alpha_bound);
    let alpha_g = alpha::alpha(rat.coefficients(), opts.alpha_bound);
    println!(
        "Murphy alpha over primes <= {}: algebraic {alpha_f:+.4}, rational {alpha_g:+.4} (docs/murphy_alpha.md; computed here, not read from the instrument)",
        opts.alpha_bound
    );
    let alphas = (alpha_f, alpha_g);

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
    let mut dump_file = match &opts.dump {
        Some(path) => {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|err| format!("{}: {err}", parent.display()))?;
            }
            let mut file = fs::File::create(path).map_err(|err| format!("{}: {err}", path.display()))?;
            use std::io::Write as _;
            writeln!(file, "a,b,ln_f,lpf_f,ln_g,lpf_g").map_err(|err| err.to_string())?;
            println!("per-value dump: {} (lpf 0 means not {b_top}-smooth)", path.display());
            Some(file)
        }
        None => None,
    };

    let mut total = Tally::new(bounds.len());
    let rows: Vec<i64> = (1..=b_max).collect();
    for batch in rows.chunks(threads * 4) {
        let batch_started = Instant::now();
        let per_thread = batch.len().div_ceil(threads).max(1);
        let dump = dump_file.is_some();
        let tallies: Vec<Tally> = std::thread::scope(|scope| {
            let handles: Vec<_> = batch
                .chunks(per_thread)
                .map(|rows| {
                    let (alg, rat, method, dickman, bounds) = (&alg, &rat, &method, &dickman, &bounds);
                    scope.spawn(move || {
                        measure_rows(rows, a_max, alg, rat, method, dickman, bounds, alphas, dump)
                    })
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
            if let Some(file) = dump_file.as_mut() {
                use std::io::Write as _;
                file.write_all(tally.dump.as_bytes())
                    .map_err(|err| format!("dump: {err}"))?;
            }
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
    struct SideStats {
        observed: f64,
        stderr: f64,
        predicted: f64,
        gap: f64,
        predicted_alpha: f64,
        gap_alpha: f64,
        predicted_local: f64,
        gap_local: f64,
        mean_u: f64,
    }
    let side = |smooth: u64, pred: f64, pred_alpha: f64, pred_local: f64, u: f64| -> SideStats {
        if n == 0.0 {
            return SideStats {
                observed: 0.0,
                stderr: 0.0,
                predicted: 0.0,
                gap: 0.0,
                predicted_alpha: 0.0,
                gap_alpha: 0.0,
                predicted_local: 0.0,
                gap_local: 0.0,
                mean_u: 0.0,
            };
        }
        let observed = smooth as f64 / n;
        let predicted = pred / n;
        let predicted_alpha = pred_alpha / n;
        let predicted_local = pred_local / n;
        SideStats {
            observed,
            stderr: (observed * (1.0 - observed) / n).sqrt(),
            predicted,
            gap: observed - predicted,
            predicted_alpha,
            gap_alpha: observed - predicted_alpha,
            predicted_local,
            gap_local: observed - predicted_local,
            mean_u: u / n,
        }
    };
    let out = opts
        .out
        .clone()
        .unwrap_or_else(|| PathBuf::from("data/results").join(format!("{instance}-{run_id}.json")));
    let stem = out.with_extension("").display().to_string();
    let table_path = |name: &str| PathBuf::from(format!("{stem}.{name}.csv"));
    let units = |gap: f64, err: f64| if err > 0.0 { gap / err } else { 0.0 };

    println!(
        "{:>9} | {:>8} {:>8} {:>8} {:>8} {:>8} {:>8} {:>6} | {:>8} {:>8} {:>8} {:>8} {:>8} {:>8} {:>6} | {:>8}",
        "B", "alg_sm", "alg_obs", "raw_prd", "raw_gap", "alf_prd", "alf_gap", "u_alg",
        "rat_sm", "rat_obs", "raw_prd", "raw_gap", "alf_prd", "alf_gap", "u_rat", "both_sm"
    );
    let mut results = Vec::new();
    let mut bounds_csv = String::from(
        "b,side,smooth,observed,observed_stderr,predicted,gap,gap_stderr_units,predicted_alpha,gap_alpha,gap_alpha_stderr_units,predicted_local,gap_local,gap_local_stderr_units,mean_u,alpha,implied_alpha\n",
    );
    let mut bins_csv = String::from(
        "b,side,u_lo,u_hi,count,smooth,observed,predicted,ratio,predicted_alpha,ratio_alpha,predicted_local,ratio_local\n",
    );
    for (i, &b) in bounds.iter().enumerate() {
        let ln_b = (b as f64).ln();
        let a = side(total.alg_smooth[i], total.alg_pred[i], total.alg_pred_alpha[i], total.alg_pred_local[i], total.alg_u[i]);
        let r = side(total.rat_smooth[i], total.rat_pred[i], total.rat_pred_alpha[i], total.rat_pred_local[i], total.rat_u[i]);
        let a_implied = implied_alpha(&total.alg_bins[i], ln_b, a.observed, &dickman);
        let r_implied = implied_alpha(&total.rat_bins[i], ln_b, r.observed, &dickman);
        println!(
            "{b:>9} | {:>8} {:>8.5} {:>8.5} {:>+8.5} {:>8.5} {:>+8.5} {:>6.2} | {:>8} {:>8.5} {:>8.5} {:>+8.5} {:>8.5} {:>+8.5} {:>6.2} | {:>8}",
            total.alg_smooth[i], a.observed, a.predicted, a.gap, a.predicted_alpha, a.gap_alpha, a.mean_u,
            total.rat_smooth[i], r.observed, r.predicted, r.gap, r.predicted_alpha, r.gap_alpha, r.mean_u,
            total.both[i]
        );
        println!(
            "{:>9} | stderr {:.5}; raw gap {:+.1} se; alpha gap {:+.1} se; local gap {:+.1} se (pred {:.5}); implied alpha {}  | stderr {:.5}; raw gap {:+.1} se; alpha gap {:+.1} se; local gap {:+.1} se (pred {:.5}); implied alpha {}",
            "",
            a.stderr, units(a.gap, a.stderr), units(a.gap_alpha, a.stderr), units(a.gap_local, a.stderr), a.predicted_local,
            a_implied.map(|x| format!("{x:+.2}")).unwrap_or_else(|| "n/a".into()),
            r.stderr, units(r.gap, r.stderr), units(r.gap_alpha, r.stderr), units(r.gap_local, r.stderr), r.predicted_local,
            r_implied.map(|x| format!("{x:+.2}")).unwrap_or_else(|| "n/a".into()),
        );
        let side_json = |s: &SideStats, smooth: u64, alpha_value: f64, implied: Option<f64>| {
            let mut v = json!({
                "smooth": smooth, "observed": s.observed, "observed_stderr": s.stderr,
                "predicted": s.predicted, "gap": s.gap,
                "predicted_alpha": s.predicted_alpha, "gap_alpha": s.gap_alpha,
                "predicted_local": s.predicted_local, "gap_local": s.gap_local,
                "mean_u": s.mean_u, "alpha": alpha_value,
            });
            if let Some(x) = implied {
                v["implied_alpha"] = json!(x);
            }
            v
        };
        let alg_json = side_json(&a, total.alg_smooth[i], alpha_f, a_implied);
        let rat_json = side_json(&r, total.rat_smooth[i], alpha_g, r_implied);
        results.push(json!({"b": b, "algebraic": alg_json, "rational": rat_json, "both_smooth": total.both[i]}));
        use std::fmt::Write as _;
        for (name, s, smooth, alpha_value, implied) in [
            ("algebraic", &a, total.alg_smooth[i], alpha_f, a_implied),
            ("rational", &r, total.rat_smooth[i], alpha_g, r_implied),
        ] {
            let _ = writeln!(
                bounds_csv,
                "{b},{name},{smooth},{:.7},{:.7},{:.7},{:+.7},{:.2},{:.7},{:+.7},{:.2},{:.7},{:+.7},{:.2},{:.4},{:.4},{}",
                s.observed, s.stderr, s.predicted, s.gap, units(s.gap, s.stderr),
                s.predicted_alpha, s.gap_alpha, units(s.gap_alpha, s.stderr),
                s.predicted_local, s.gap_local, units(s.gap_local, s.stderr), s.mean_u, alpha_value,
                implied.map(|x| format!("{x:.4}")).unwrap_or_default()
            );
        }
        for (name, bins) in [("algebraic", &total.alg_bins[i]), ("rational", &total.rat_bins[i])] {
            for (j, bin) in bins.iter().enumerate() {
                if bin.count == 0 {
                    continue;
                }
                let observed = bin.smooth as f64 / bin.count as f64;
                let predicted = bin.rho_sum / bin.count as f64;
                let predicted_alpha = bin.rho_alpha_sum / bin.count as f64;
                let predicted_local = bin.local_sum / bin.count as f64;
                let ratio = |p: f64| if p > 0.0 { format!("{:.4}", observed / p) } else { String::new() };
                let _ = writeln!(
                    bins_csv,
                    "{b},{name},{:.1},{:.1},{},{},{observed:.7},{predicted:.7},{},{predicted_alpha:.7},{},{predicted_local:.7},{}",
                    j as f64 * U_BIN_WIDTH,
                    (j + 1) as f64 * U_BIN_WIDTH,
                    bin.count,
                    bin.smooth,
                    ratio(predicted),
                    ratio(predicted_alpha),
                    ratio(predicted_local)
                );
            }
        }
    }
    println!(
        "alpha_applied: true (Murphy alpha from docs/murphy_alpha.md over primes <= {}); implied alpha is the size shift that would reconcile raw Dickman with the count, for comparison with the computed alpha",
        opts.alpha_bound
    );

    let mut rows_csv = String::from(
        "b,evaluated,alg_smooth,rat_smooth,both_smooth,alg_density,rat_density,mean_ln_f,mean_ln_g\n",
    );
    for (b, row) in &total.rows {
        use std::fmt::Write as _;
        let e = row.evaluated.max(1) as f64;
        let _ = writeln!(
            rows_csv,
            "{b},{},{},{},{},{:.6},{:.6},{:.4},{:.4}",
            row.evaluated,
            row.alg_smooth,
            row.rat_smooth,
            row.both,
            row.alg_smooth as f64 / e,
            row.rat_smooth as f64 / e,
            row.ln_f_sum / e,
            row.ln_g_sum / e
        );
    }
    let mut lpf_csv = String::from("side,log2_lpf_bin,count,share\n");
    for (name, hist) in [("algebraic", &total.alg_lpf_hist), ("rational", &total.rat_lpf_hist)] {
        use std::fmt::Write as _;
        for (j, count) in hist.iter().enumerate() {
            if *count == 0 {
                continue;
            }
            let label = if j == LPF_BINS - 1 { "not_smooth".to_string() } else { j.to_string() };
            let _ = writeln!(lpf_csv, "{name},{label},{count},{:.7}", *count as f64 / n.max(1.0));
        }
    }

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

    let tables = [
        ("bounds", bounds_csv, "per bound and side: counts, observed density, raw, alpha-adjusted and local-density Dickman predictions, gaps in absolute and standard-error units, mean u, computed alpha, implied alpha"),
        ("u_bins", bins_csv, "per bound, side and u bin of width 0.1: count, smooth, observed, raw, alpha-adjusted and local-density predictions, their ratios to observed"),
        ("rows", rows_csv, "per b row at the top bound: counts, densities, mean ln|F| and ln|G|"),
        ("lpf_hist", lpf_csv, "per side: values by floor(log2(largest prime factor)) at the top bound, plus not_smooth"),
    ];
    let mut table_records = Vec::new();
    for (name, csv, description) in &tables {
        let path = table_path(name);
        write_text(&path, csv)?;
        let header = csv.lines().next().unwrap_or("");
        table_records.push(json!({
            "name": name,
            "path": path.to_string_lossy(),
            "columns": header.split(',').collect::<Vec<_>>(),
            "rows": csv.lines().count().saturating_sub(1),
            "description": description,
        }));
        println!("wrote {}", path.display());
    }

    let mut result = json!({
        "schema_version": RESULT_V2,
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
        "alpha_applied": true,
        "smoothness": smoothness,
        "alpha": {"method": alpha::METHOD, "bound": opts.alpha_bound, "algebraic": alpha_f, "rational": alpha_g, "definition": "docs/murphy_alpha.md"},
        "results": results,
        "tables": table_records,
    });
    if let Some(hardware) = hardware {
        result["hardware"] = hardware;
    }
    if let Some(path) = &opts.dump {
        result["dump"] = json!(path.to_string_lossy());
    }
    registry.validate_value(RESULT_V2, &result)?;
    let text = serde_json::to_string_pretty(&result).map_err(|err| err.to_string())?;
    write_text(&out, &(text + "\n"))?;
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
