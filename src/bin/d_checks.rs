//! d-checks: per-value checks of the prediction D1 ended with, on the
//! d1-measure dumps (docs/obstacle_d_verdict.md).
//!
//! The manifest (written by scripts/d_checks.py) names the columns, each one
//! side of one dump with its smooth probability per bound and cell of ln c
//! (scripts/d5_clusters.py prediction_grid). Per column, bound and bin the run
//! sums the values, the smooth ones O, the prediction P = Σπ and the Bernoulli
//! variance V = Σπ(1 − π), grouped by
//!
//!   all     every value
//!   certain the values smooth for certain (π = 1: the cofactor is below B)
//!
//! and, over the other values only (π < 1, the ones that carry information),
//!
//!   u       u = ln c / ln B, bins of 0.25
//!   lns     ln of the value's part made of the primes up to the rough bound, bins of 1
//!   olns    the same for the other side at the same (a, b)
//!   oc      ln of the other side's cofactor, bins of 2
//!   parity  (a mod 2, b mod 2)
//!   mod3    (a mod 3, b mod 3)
//!
//! and the neighbouring values, (a, b) with (a + 1, b) (`pair-a`) and with
//! (a, b + 1) (`pair-b`): the pairs where both are smooth, their expectation
//! Σ π π′ for independent values, and the variance of that count, with the
//! covariance of consecutive pairs that share a value. CSV out.

use serde_json::Value;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

const HEADER: &str = "a,b,ln_f,lpf_f,ln_g,lpf_g,ln_sf,ln_sg";
const MANIFEST_V1: &str = "d-checks-manifest/1";
const GROUPS: [(&str, usize); 8] =
    [("all", 1), ("certain", 1), ("u", 64), ("lns", 64), ("olns", 64), ("oc", 64), ("parity", 4), ("mod3", 9)];
const U_WIDTH: f64 = 0.25;
const MISSING: u8 = u8::MAX;

fn usage() -> ! {
    eprintln!("Usage: d-checks --manifest FILE --out FILE [--threads N]");
    std::process::exit(2);
}

struct Column {
    name: String,
    dump: PathBuf,
    side: usize,
    pi: Vec<Vec<f64>>,
}

struct Manifest {
    bounds: Vec<u64>,
    inv_h: f64,
    a_max: i64,
    b_max: i64,
    columns: Vec<Column>,
}

fn load(path: &Path) -> Result<Manifest, String> {
    let text = std::fs::read_to_string(path).map_err(|err| format!("{}: {err}", path.display()))?;
    let v: Value = serde_json::from_str(&text).map_err(|err| format!("{}: {err}", path.display()))?;
    let bad = |what: &str| format!("{}: {what}", path.display());
    if v["schema_version"].as_str() != Some(MANIFEST_V1) {
        return Err(bad("not a d-checks-manifest/1 manifest"));
    }
    let bounds: Vec<u64> = v["bounds"].as_array().ok_or_else(|| bad("bounds"))?.iter().filter_map(Value::as_u64).collect();
    if bounds.is_empty() || bounds.len() >= MISSING as usize {
        return Err(bad("bounds"));
    }
    let width = v["cofactor_bin_width"].as_f64().ok_or_else(|| bad("cofactor_bin_width"))?;
    let a_max = v["region"]["a_max"].as_i64().filter(|&x| x > 0).ok_or_else(|| bad("region.a_max"))?;
    let b_max = v["region"]["b_max"].as_i64().filter(|&x| x > 0).ok_or_else(|| bad("region.b_max"))?;
    let mut columns = Vec::new();
    for c in v["columns"].as_array().ok_or_else(|| bad("columns"))? {
        let pi: Vec<Vec<f64>> = c["pi"]
            .as_array()
            .ok_or_else(|| bad("pi"))?
            .iter()
            .map(|row| row.as_array().map(|r| r.iter().map(|x| x.as_f64().unwrap_or(f64::NAN)).collect()).unwrap_or_default())
            .collect();
        if pi.len() != bounds.len() {
            return Err(bad("pi needs one row per bound"));
        }
        columns.push(Column {
            name: c["name"].as_str().ok_or_else(|| bad("name"))?.to_string(),
            dump: PathBuf::from(c["dump"].as_str().ok_or_else(|| bad("dump"))?),
            side: match c["side"].as_str() {
                Some("algebraic") => 0,
                Some("rational") => 1,
                _ => return Err(bad("side")),
            },
            pi,
        });
    }
    Ok(Manifest { bounds, inv_h: 8.0 / width, a_max, b_max, columns })
}

/// Pairs of consecutive entries of one line of the region (a row or a
/// column); `None` where the line has no value (gcd(a, b) > 1). Returns the
/// pairs where both are smooth, Σ p p′ and the variance of the count for
/// independent values: Σ e(1 − e) over the pairs, e = p p′, plus twice
/// p_x p_y (1 − p_y) p_z for consecutive pairs (x, y), (y, z).
fn line_pairs(line: &[Option<(f64, bool)>]) -> (f64, f64, f64) {
    let (mut o, mut e, mut v) = (0.0, 0.0, 0.0);
    for w in line.windows(2) {
        if let (Some((p, s)), Some((q, t))) = (w[0], w[1]) {
            let pq = p * q;
            e += pq;
            v += pq * (1.0 - pq);
            if s && t {
                o += 1.0;
            }
        }
    }
    for w in line.windows(3) {
        if let (Some((px, _)), Some((py, _)), Some((pz, _))) = (w[0], w[1], w[2]) {
            v += 2.0 * px * pz * py * (1.0 - py);
        }
    }
    (o, e, v)
}

/// Bin of (a mod 2, b mod 2) and of (a mod 3, b mod 3), for negative a too.
fn residue_bins(a: i64, b: i64) -> (usize, usize) {
    ((a.rem_euclid(2) * 2 + b.rem_euclid(2)) as usize, (a.rem_euclid(3) * 3 + b.rem_euclid(3)) as usize)
}

/// Sums per bound and bin of every group: [values, O, P, V].
struct Sums {
    offsets: Vec<usize>,
    total: usize,
    cells: Vec<[f64; 4]>,
    pairs: Vec<[f64; 3]>,
}

impl Sums {
    fn new(nb: usize) -> Self {
        let mut offsets = Vec::new();
        let mut total = 0;
        for (_, n) in GROUPS {
            offsets.push(total);
            total += n;
        }
        Self { offsets, total, cells: vec![[0.0; 4]; total * nb], pairs: vec![[0.0; 3]; 2 * nb] }
    }

    fn add(&mut self, j: usize, group: usize, bin: usize, smooth: bool, p: f64) {
        let c = &mut self.cells[j * self.total + self.offsets[group] + bin.min(GROUPS[group].1 - 1)];
        c[0] += 1.0;
        if smooth {
            c[1] += 1.0;
        }
        c[2] += p;
        c[3] += p * (1.0 - p);
    }
}

fn run_column(m: &Manifest, col: &Column) -> Result<Sums, String> {
    let nb = m.bounds.len();
    let ln_b: Vec<f64> = m.bounds.iter().map(|&b| (b as f64).ln()).collect();
    let na = (2 * m.a_max + 1) as usize;
    let rows = m.b_max as usize;
    let mut grid_cell = vec![0u32; na * rows];
    let mut grid_level = vec![MISSING; na * rows];
    let mut sums = Sums::new(nb);
    let file = File::open(&col.dump).map_err(|err| format!("{}: {err}", col.dump.display()))?;
    let mut lines = BufReader::with_capacity(1 << 22, file).lines();
    let header = lines.next().transpose().map_err(|err| format!("{}: {err}", col.dump.display()))?;
    if header.as_deref().map(str::trim) != Some(HEADER) {
        return Err(format!("{}: not a d1-measure dump with the small-part columns", col.dump.display()));
    }
    let (own, other) = if col.side == 0 { ((2, 3, 6), (4, 7)) } else { ((4, 5, 7), (2, 6)) };
    for line in lines {
        let line = line.map_err(|err| format!("{}: {err}", col.dump.display()))?;
        let f: Vec<&str> = line.split(',').collect();
        if f.len() != 8 {
            return Err(format!("{}: bad line {line}", col.dump.display()));
        }
        let num = |s: &str| s.parse::<f64>().map_err(|err| format!("{}: {err} in {line}", col.dump.display()));
        let int = |s: &str| s.parse::<i64>().map_err(|err| format!("{}: {err} in {line}", col.dump.display()));
        let (a, b) = (int(f[0])?, int(f[1])?);
        if a.abs() > m.a_max || b < 1 || b > m.b_max {
            return Err(format!("{}: ({a}, {b}) lies outside the manifest's region", col.dump.display()));
        }
        let (ln_v, lpf, ln_s) = (num(f[own.0])?, int(f[own.1])? as u64, num(f[own.2])?);
        let (o_ln_v, o_ln_s) = (num(f[other.0])?, num(f[other.1])?);
        let ln_c = ln_v - ln_s;
        let cell = if ln_c > 0.0 { (ln_c * m.inv_h) as usize } else { 0 };
        let level = m.bounds.iter().position(|&bd| lpf > 0 && lpf <= bd).unwrap_or(nb);
        let lns = ln_s.max(0.0) as usize;
        let olns = o_ln_s.max(0.0) as usize;
        let oc = ((o_ln_v - o_ln_s).max(0.0) / 2.0) as usize;
        let (par, m3) = residue_bins(a, b);
        for (j, lb) in ln_b.iter().enumerate() {
            let p = *col.pi[j].get(cell).ok_or_else(|| format!("{}: a value lies beyond the table", col.name))?;
            if !p.is_finite() {
                return Err(format!("{}: a value lies outside the baseline table at {}", col.name, m.bounds[j]));
            }
            let smooth = level <= j;
            sums.add(j, 0, 0, smooth, p);
            if p >= 1.0 {
                sums.add(j, 1, 0, smooth, p);
                continue;
            }
            let u = if ln_c > 0.0 { (ln_c / lb / U_WIDTH) as usize } else { 0 };
            for (g, bin) in [(2, u), (3, lns), (4, olns), (5, oc), (6, par), (7, m3)] {
                sums.add(j, g, bin, smooth, p);
            }
        }
        let k = (b - 1) as usize * na + (a + m.a_max) as usize;
        grid_cell[k] = cell as u32;
        grid_level[k] = level as u8;
    }
    for j in 0..nb {
        let at = |k: usize| -> Option<(f64, bool)> {
            (grid_level[k] != MISSING).then(|| (col.pi[j][grid_cell[k] as usize], (grid_level[k] as usize) <= j))
        };
        for (dir, count, len, index) in [
            (0usize, rows, na, Box::new(|r: usize, i: usize| r * na + i) as Box<dyn Fn(usize, usize) -> usize>),
            (1usize, na, rows, Box::new(|r: usize, i: usize| i * na + r) as Box<dyn Fn(usize, usize) -> usize>),
        ] {
            let mut line = Vec::with_capacity(len);
            for r in 0..count {
                line.clear();
                line.extend((0..len).map(|i| at(index(r, i))));
                let (o, e, v) = line_pairs(&line);
                let s = &mut sums.pairs[j * 2 + dir];
                s[0] += o;
                s[1] += e;
                s[2] += v;
            }
        }
    }
    Ok(sums)
}

fn main() {
    let mut manifest = None;
    let mut out = None;
    let mut threads = std::thread::available_parallelism().map(|n| n.get().min(8)).unwrap_or(1);
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--manifest" => manifest = args.next().map(PathBuf::from),
            "--out" => out = args.next().map(PathBuf::from),
            "--threads" => threads = args.next().and_then(|v| v.parse().ok()).filter(|n| *n >= 1).unwrap_or_else(|| usage()),
            _ => usage(),
        }
    }
    let (Some(manifest), Some(out)) = (manifest, out) else { usage() };
    if let Err(err) = run(&manifest, &out, threads) {
        eprintln!("d-checks: {err}");
        std::process::exit(1);
    }
}

fn run(manifest: &Path, out: &Path, threads: usize) -> Result<(), String> {
    let m = load(manifest)?;
    let next = AtomicUsize::new(0);
    let results: Mutex<Vec<Option<Result<Sums, String>>>> = Mutex::new((0..m.columns.len()).map(|_| None).collect());
    std::thread::scope(|scope| {
        for _ in 0..threads.min(m.columns.len()) {
            scope.spawn(|| loop {
                let i = next.fetch_add(1, Ordering::Relaxed);
                if i >= m.columns.len() {
                    break;
                }
                let r = run_column(&m, &m.columns[i]);
                results.lock().expect("results lock")[i] = Some(r);
            });
        }
    });
    let results = results.into_inner().expect("results lock");
    let mut w = std::io::BufWriter::new(File::create(out).map_err(|err| format!("{}: {err}", out.display()))?);
    let io = |err: std::io::Error| format!("{}: {err}", out.display());
    writeln!(w, "column,group,bin,b,values,observed,expected,variance").map_err(io)?;
    for (col, r) in m.columns.iter().zip(results) {
        let s = r.expect("every column ran")?;
        for (j, b) in m.bounds.iter().enumerate() {
            for (g, (name, n)) in GROUPS.iter().enumerate() {
                for bin in 0..*n {
                    let c = s.cells[j * s.total + s.offsets[g] + bin];
                    if c[0] > 0.0 {
                        writeln!(w, "{},{name},{bin},{b},{},{},{:.6},{:.6}", col.name, c[0], c[1], c[2], c[3]).map_err(io)?;
                    }
                }
            }
            for (dir, name) in ["pair-a", "pair-b"].iter().enumerate() {
                let p = s.pairs[j * 2 + dir];
                writeln!(w, "{},{name},0,{b},,{},{:.6},{:.6}", col.name, p[0], p[1], p[2]).map_err(io)?;
            }
        }
        eprintln!("{}: done", col.name);
    }
    w.flush().map_err(io)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mean and variance of Σ X_i X_{i+1} over every outcome of independent X_i.
    fn brute(p: &[Option<f64>]) -> (f64, f64) {
        let idx: Vec<usize> = (0..p.len()).filter(|&i| p[i].is_some()).collect();
        let (mut m1, mut m2) = (0.0, 0.0);
        for mask in 0u32..(1 << idx.len()) {
            let mut x = vec![false; p.len()];
            let mut w = 1.0;
            for (k, &i) in idx.iter().enumerate() {
                let on = mask >> k & 1 == 1;
                x[i] = on;
                let pi = p[i].unwrap();
                w *= if on { pi } else { 1.0 - pi };
            }
            let s = (0..p.len() - 1).filter(|&i| p[i].is_some() && p[i + 1].is_some() && x[i] && x[i + 1]).count() as f64;
            m1 += w * s;
            m2 += w * s * s;
        }
        (m1, m2 - m1 * m1)
    }

    #[test]
    fn neighbour_variance_matches_enumeration() {
        let line = [Some(0.3), Some(0.9), None, Some(0.5), Some(0.2), Some(1.0), Some(0.7), Some(0.05)];
        let with_flags: Vec<Option<(f64, bool)>> = line.iter().map(|p| p.map(|p| (p, false))).collect();
        let (_, e, v) = line_pairs(&with_flags);
        let (be, bv) = brute(&line);
        assert!((e - be).abs() < 1e-12, "{e} {be}");
        assert!((v - bv).abs() < 1e-12, "{v} {bv}");
    }

    #[test]
    fn neighbour_counts_both_smooth() {
        let line = [Some((0.5, true)), Some((0.5, true)), Some((0.5, false)), None, Some((0.5, true)), Some((0.5, true))];
        assert_eq!(line_pairs(&line).0, 2.0);
    }

    #[test]
    fn residues_of_negative_a() {
        assert_eq!(residue_bins(-3000, 1), (1, 1));
        assert_eq!(residue_bins(-1, 2), (2, 2 * 3 + 2));
        assert_eq!(residue_bins(4, 3), (1, 3));
        assert_eq!(residue_bins(-6, 4), (0, 1));
    }
}
