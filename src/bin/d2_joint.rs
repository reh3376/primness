//! d2-joint: joint smoothness of values at the same (a, b), against the
//! product of their individual predictions (docs/d2_falsifiable_prediction.md).
//!
//! The manifest (written by scripts/d2_joint.py) names the columns, each one
//! side of one d1-measure dump with its smooth probability per bound and cell
//! of ln c, and the classes of pairs of columns. Every dump must list the same
//! (a, b) in the same order. Per column and bound the run counts the values,
//! the smooth ones and the prediction, and fits κ = O/P over the values whose
//! probability is below 1. With p = min(1, κπ) (1 where π is 1) it then sums,
//! per pair and class, the values at which both columns are smooth, their
//! expectation Σ p_s p_t, and the exact variance of that count when every
//! value's events are independent; a class's variance includes the
//! covariance of its pairs through shared columns. A union gives, for one
//! rational column and several algebraic ones, the values at which the
//! rational one and at least one other are smooth. CSV out.

use serde_json::Value;
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

const HEADER: &str = "a,b,ln_f,lpf_f,ln_g,lpf_g,ln_sf,ln_sg";
const CHUNKS: usize = 96;
const MANIFEST_V1: &str = "d2-joint-manifest/1";

fn usage() -> ! {
    eprintln!("Usage: d2-joint --manifest FILE --out FILE [--threads N]");
    std::process::exit(2);
}

struct Column {
    name: String,
    dump: PathBuf,
    side: usize,
    pi: Vec<Vec<f64>>,
}

struct Class {
    name: String,
    pairs: Vec<(usize, usize)>,
    /// For every ordered pair of pairs, the columns of their union.
    unions: Vec<Vec<usize>>,
}

impl Class {
    fn new(name: String, pairs: Vec<(usize, usize)>) -> Self {
        let mut unions = Vec::new();
        for &(s, t) in &pairs {
            for &(u, w) in &pairs {
                let mut cols = vec![s, t, u, w];
                cols.sort_unstable();
                cols.dedup();
                unions.push(cols);
            }
        }
        Self { name, pairs, unions }
    }

    /// E[(Σ_pairs X_s X_t)^2] for independent X_c with P(X_c = 1) = p[c]:
    /// every ordered pair of pairs contributes the product of p over the
    /// columns the two pairs use between them.
    fn second_moment(&self, p: &[f64]) -> f64 {
        self.unions.iter().map(|cols| cols.iter().map(|&c| p[c]).product::<f64>()).sum()
    }
}

struct Union {
    name: String,
    rational: usize,
    fields: Vec<usize>,
}

struct Manifest {
    bounds: Vec<u64>,
    inv_h: f64,
    columns: Vec<Column>,
    classes: Vec<Class>,
    unions: Vec<Union>,
}

fn load(path: &Path) -> Result<Manifest, String> {
    let text = fs::read_to_string(path).map_err(|err| format!("{}: {err}", path.display()))?;
    let v: Value = serde_json::from_str(&text).map_err(|err| format!("{}: {err}", path.display()))?;
    let bad = |what: &str| format!("{}: {what}", path.display());
    if v["schema_version"].as_str() != Some(MANIFEST_V1) {
        return Err(bad("not a d2-joint-manifest/1 manifest"));
    }
    let bounds: Vec<u64> = v["bounds"].as_array().ok_or_else(|| bad("bounds"))?.iter().filter_map(Value::as_u64).collect();
    let width = v["cofactor_bin_width"].as_f64().ok_or_else(|| bad("cofactor_bin_width"))?;
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
    let index = |x: &Value| -> Result<usize, String> {
        x.as_u64().map(|i| i as usize).filter(|&i| i < columns.len()).ok_or_else(|| bad("column index"))
    };
    let mut classes = Vec::new();
    for c in v["classes"].as_array().ok_or_else(|| bad("classes"))? {
        let mut pairs = Vec::new();
        for p in c["pairs"].as_array().ok_or_else(|| bad("pairs"))? {
            let p = p.as_array().ok_or_else(|| bad("pair"))?;
            pairs.push((index(&p[0])?, index(&p[1])?));
        }
        classes.push(Class::new(c["name"].as_str().ok_or_else(|| bad("class name"))?.to_string(), pairs));
    }
    let mut unions = Vec::new();
    for u in v["unions"].as_array().map(Vec::as_slice).unwrap_or(&[]) {
        unions.push(Union {
            name: u["name"].as_str().ok_or_else(|| bad("union name"))?.to_string(),
            rational: index(&u["rational"])?,
            fields: u["fields"].as_array().ok_or_else(|| bad("union fields"))?.iter().map(index).collect::<Result<_, _>>()?,
        });
    }
    Ok(Manifest { bounds, inv_h: 8.0 / width, columns, classes, unions })
}

/// Per column: the cell of ln c and the index of the first bound at which
/// the value is smooth (the number of bounds when it is not), and the pairs.
struct Values {
    cells: Vec<Vec<u32>>,
    levels: Vec<Vec<u8>>,
    n: usize,
}

fn read(m: &Manifest) -> Result<Values, String> {
    let mut by_dump: BTreeMap<&Path, Vec<usize>> = BTreeMap::new();
    for (i, c) in m.columns.iter().enumerate() {
        by_dump.entry(c.dump.as_path()).or_default().push(i);
    }
    let files: Vec<(&Path, Vec<usize>)> = by_dump.into_iter().collect();
    type Parsed = (Vec<(i16, i16)>, Vec<(usize, Vec<u32>, Vec<u8>)>);
    let parsed: Vec<Result<Parsed, String>> = std::thread::scope(|scope| {
        let handles: Vec<_> = files
            .iter()
            .map(|(path, cols)| {
                scope.spawn(move || -> Result<Parsed, String> {
                    let text = fs::read_to_string(path).map_err(|err| format!("{}: {err}", path.display()))?;
                    let mut lines = text.lines();
                    if lines.next().map(|h| h.trim()) != Some(HEADER) {
                        return Err(format!("{}: not a d1-measure dump with the small-part columns", path.display()));
                    }
                    let mut ab = Vec::new();
                    let mut out: Vec<(usize, Vec<u32>, Vec<u8>)> = cols.iter().map(|&c| (c, Vec::new(), Vec::new())).collect();
                    for line in lines {
                        let f: Vec<&str> = line.split(',').collect();
                        if f.len() != 8 {
                            return Err(format!("{}: bad line {line}", path.display()));
                        }
                        let num = |s: &str| s.parse::<f64>().map_err(|err| format!("{}: {err} in {line}", path.display()));
                        let int = |s: &str| s.parse::<i64>().map_err(|err| format!("{}: {err} in {line}", path.display()));
                        ab.push((int(f[0])? as i16, int(f[1])? as i16));
                        for (c, cells, levels) in out.iter_mut() {
                            let (ln_v, lpf, ln_s) = if m.columns[*c].side == 0 { (f[2], f[3], f[6]) } else { (f[4], f[5], f[7]) };
                            let ln_c = num(ln_v)? - num(ln_s)?;
                            cells.push(if ln_c > 0.0 { (ln_c * m.inv_h) as u32 } else { 0 });
                            let lpf = int(lpf)? as u64;
                            let level = m.bounds.iter().position(|&b| lpf > 0 && lpf <= b).unwrap_or(m.bounds.len());
                            levels.push(level as u8);
                        }
                    }
                    Ok((ab, out))
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().expect("reader panicked")).collect()
    });
    let mut cells = vec![Vec::new(); m.columns.len()];
    let mut levels = vec![Vec::new(); m.columns.len()];
    let mut reference: Option<Vec<(i16, i16)>> = None;
    for (result, (path, _)) in parsed.into_iter().zip(&files) {
        let (ab, cols) = result?;
        match &reference {
            None => reference = Some(ab),
            Some(r) if *r == ab => {}
            Some(_) => return Err(format!("{}: its (a, b) differ from {}", path.display(), files[0].0.display())),
        }
        for (c, cl, lv) in cols {
            cells[c] = cl;
            levels[c] = lv;
        }
    }
    let n = reference.map(|r| r.len()).unwrap_or(0);
    Ok(Values { cells, levels, n })
}

#[derive(Clone, Default)]
struct Sums {
    observed: u64,
    expected: f64,
    variance: f64,
}

impl Sums {
    fn add(&mut self, o: &Sums) {
        self.observed += o.observed;
        self.expected += o.expected;
        self.variance += o.variance;
    }
}

fn main() {
    let mut manifest = None;
    let mut out = None;
    let mut threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
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
        eprintln!("d2-joint: {err}");
        std::process::exit(1);
    }
}

fn run(manifest: &Path, out: &Path, threads: usize) -> Result<(), String> {
    let m = load(manifest)?;
    let v = read(&m)?;
    let (nb, nc) = (m.bounds.len(), m.columns.len());

    // Pass 1: per column and bound, the counts and κ over the uncertain values.
    let mut kappa = vec![vec![1.0f64; nb]; nc];
    let mut rows = String::from("kind,name,s,t,b,values,observed,expected,variance,kappa,certain\n");
    for (c, col) in m.columns.iter().enumerate() {
        for j in 0..nb {
            let (mut obs, mut certain, mut pred, mut var, mut pred_unc, mut obs_unc) = (0u64, 0u64, 0.0f64, 0.0f64, 0.0f64, 0u64);
            for i in 0..v.n {
                let cell = v.cells[c][i] as usize;
                let pi = *col.pi[j].get(cell).unwrap_or(&f64::NAN);
                if pi.is_nan() {
                    return Err(format!("{}: a value at ln c cell {cell} lies outside the prediction", col.name));
                }
                let smooth = (v.levels[c][i] as usize) <= j;
                obs += u64::from(smooth);
                pred += pi;
                var += pi * (1.0 - pi);
                if pi >= 1.0 {
                    certain += 1;
                } else {
                    pred_unc += pi;
                    obs_unc += u64::from(smooth);
                }
            }
            if pred_unc > 0.0 {
                kappa[c][j] = obs_unc as f64 / pred_unc;
            }
            rows.push_str(&format!(
                "column,{},,,{},{},{obs},{pred:.6},{var:.6},{:.9},{certain}\n",
                col.name, m.bounds[j], v.n, kappa[c][j]
            ));
        }
    }

    // Pass 2: pairs, classes and unions, on fixed chunks merged in order.
    let n_pairs: usize = m.classes.iter().map(|c| c.pairs.len()).sum();
    let next = AtomicUsize::new(0);
    let chunk = v.n.div_ceil(CHUNKS).max(1);
    type Acc = (Vec<Sums>, Vec<Sums>, Vec<Sums>);
    let mut parts: Vec<(usize, Acc)> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads.min(CHUNKS))
            .map(|_| {
                scope.spawn(|| {
                    let mut done = Vec::new();
                    loop {
                        let k = next.fetch_add(1, Ordering::Relaxed);
                        if k >= CHUNKS {
                            break;
                        }
                        let mut pair_acc = vec![Sums::default(); n_pairs * nb];
                        let mut class_acc = vec![Sums::default(); m.classes.len() * nb];
                        let mut union_acc = vec![Sums::default(); m.unions.len() * nb];
                        let mut p = vec![0.0f64; nc];
                        let mut s = vec![false; nc];
                        for i in (k * chunk)..((k + 1) * chunk).min(v.n) {
                            for j in 0..nb {
                                for c in 0..nc {
                                    let pi = m.columns[c].pi[j][v.cells[c][i] as usize];
                                    p[c] = if pi >= 1.0 { 1.0 } else { (kappa[c][j] * pi).min(1.0) };
                                    s[c] = (v.levels[c][i] as usize) <= j;
                                }
                                let mut q = 0;
                                for (ci, class) in m.classes.iter().enumerate() {
                                    let mut mean = 0.0;
                                    let mut obs = 0u64;
                                    for &(a, b) in &class.pairs {
                                        let e = p[a] * p[b];
                                        let o = u64::from(s[a] && s[b]);
                                        let acc = &mut pair_acc[q * nb + j];
                                        acc.observed += o;
                                        acc.expected += e;
                                        acc.variance += e * (1.0 - e);
                                        mean += e;
                                        obs += o;
                                        q += 1;
                                    }
                                    let second = class.second_moment(&p);
                                    let acc = &mut class_acc[ci * nb + j];
                                    acc.observed += obs;
                                    acc.expected += mean;
                                    acc.variance += second - mean * mean;
                                }
                                for (ui, u) in m.unions.iter().enumerate() {
                                    let none: f64 = u.fields.iter().map(|&c| 1.0 - p[c]).product();
                                    let e = p[u.rational] * (1.0 - none);
                                    let acc = &mut union_acc[ui * nb + j];
                                    acc.observed += u64::from(s[u.rational] && u.fields.iter().any(|&c| s[c]));
                                    acc.expected += e;
                                    acc.variance += e * (1.0 - e);
                                }
                            }
                        }
                        done.push((k, (pair_acc, class_acc, union_acc)));
                    }
                    done
                })
            })
            .collect();
        handles.into_iter().flat_map(|h| h.join().expect("joint thread panicked")).collect()
    });
    parts.sort_by_key(|(k, _)| *k);
    let mut pair_acc = vec![Sums::default(); n_pairs * nb];
    let mut class_acc = vec![Sums::default(); m.classes.len() * nb];
    let mut union_acc = vec![Sums::default(); m.unions.len() * nb];
    for (_, (pa, ca, ua)) in &parts {
        for (x, y) in pair_acc.iter_mut().zip(pa) {
            x.add(y);
        }
        for (x, y) in class_acc.iter_mut().zip(ca) {
            x.add(y);
        }
        for (x, y) in union_acc.iter_mut().zip(ua) {
            x.add(y);
        }
    }
    let mut q = 0;
    for (ci, class) in m.classes.iter().enumerate() {
        for &(a, b) in &class.pairs {
            for j in 0..nb {
                let acc = &pair_acc[q * nb + j];
                rows.push_str(&format!(
                    "pair,{},{},{},{},{},{},{:.6},{:.6},,\n",
                    class.name, m.columns[a].name, m.columns[b].name, m.bounds[j], v.n, acc.observed, acc.expected, acc.variance
                ));
            }
            q += 1;
        }
        for j in 0..nb {
            let acc = &class_acc[ci * nb + j];
            rows.push_str(&format!(
                "class,{},,,{},{},{},{:.6},{:.6},,\n",
                class.name, m.bounds[j], v.n, acc.observed, acc.expected, acc.variance
            ));
        }
    }
    for (ui, u) in m.unions.iter().enumerate() {
        for j in 0..nb {
            let acc = &union_acc[ui * nb + j];
            rows.push_str(&format!(
                "union,{},{},,{},{},{},{:.6},{:.6},,\n",
                u.name, m.columns[u.rational].name, m.bounds[j], v.n, acc.observed, acc.expected, acc.variance
            ));
        }
    }
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("{}: {err}", parent.display()))?;
    }
    fs::write(out, rows).map_err(|err| format!("{}: {err}", out.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// E[S] and E[S^2] of S = Σ_pairs X_s X_t by enumerating every outcome.
    fn brute(p: &[f64], pairs: &[(usize, usize)]) -> (f64, f64) {
        let n = p.len();
        let (mut m1, mut m2) = (0.0, 0.0);
        for bits in 0u32..(1 << n) {
            let x = |c: usize| (bits >> c) & 1 == 1;
            let prob: f64 = (0..n).map(|c| if x(c) { p[c] } else { 1.0 - p[c] }).product();
            let s = pairs.iter().filter(|&&(a, b)| x(a) && x(b)).count() as f64;
            m1 += prob * s;
            m2 += prob * s * s;
        }
        (m1, m2)
    }

    #[test]
    fn class_variance_matches_enumeration() {
        let p = [0.31, 0.07, 0.55, 0.012, 0.9, 0.2, 0.44, 0.05, 0.6, 0.33];
        let classes = [
            vec![(0, 4), (1, 4), (2, 4), (3, 4)],
            vec![(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)],
            vec![(0, 9), (1, 9), (2, 9), (3, 9)],
            vec![(0, 6), (0, 7), (0, 8), (1, 7), (1, 8), (2, 8)],
        ];
        for pairs in classes {
            let class = Class::new("t".into(), pairs.clone());
            let mean: f64 = pairs.iter().map(|&(a, b)| p[a] * p[b]).sum();
            let (m1, m2) = brute(&p, &pairs);
            assert!((mean - m1).abs() < 1e-12, "{pairs:?}");
            assert!((class.second_moment(&p) - m2).abs() < 1e-12, "{pairs:?}: {} vs {m2}", class.second_moment(&p));
        }
    }
}
