//! d2-fields: the extra number fields of D2 part 1, chosen before any
//! smoothness is measured (docs/d2_falsifiable_prediction.md).
//!
//! For each instrument polynomial f with rational side g = Y1·x + Y0, every
//! rotation f + v·g with 0 < |v| ≤ R has the same root modulo N, the same
//! leading coefficient and the same resultant with g. Each candidate gets the
//! score mean ln|F_v(a, b)| over the coprime pairs of the region plus Murphy's
//! α over the primes up to 2000; a candidate counts only when its
//! irreducibility over Q is certified by its factor degrees modulo the primes
//! below a bound (fp::certify_irreducible). The K certified candidates with
//! the lowest score, ties to the smaller |v| and then the positive v, are
//! written as `<out>/<id>.r<v>/cado.poly`, and every candidate goes to a table.

use num_bigint::BigInt;
use num_traits::ToPrimitive;
use primness::eval::{coprime, Homogeneous};
use primness::{alpha, fp, poly};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

fn usage() -> ! {
    eprintln!(
        "Usage: d2-fields [--instruments DIR] [--out DIR] [--table FILE] [--a-max N] [--b-max N] [--range R] [--count K] [--alpha-bound N] [--certify-below P] [--threads N]"
    );
    std::process::exit(2);
}

struct Options {
    instruments: PathBuf,
    out: PathBuf,
    table: PathBuf,
    a_max: i64,
    b_max: i64,
    range: i64,
    count: usize,
    alpha_bound: u64,
    certify_below: u64,
    threads: usize,
}

fn parse_args() -> Options {
    let mut o = Options {
        instruments: PathBuf::from("data/instruments"),
        out: PathBuf::from("data/instruments/d2"),
        table: PathBuf::from("data/results/d2/fields.csv"),
        a_max: 3000,
        b_max: 1500,
        range: 16,
        count: 3,
        alpha_bound: alpha::DEFAULT_BOUND,
        certify_below: 10_000,
        threads: std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1),
    };
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        let mut value = || args.next().unwrap_or_else(|| usage());
        match arg.as_str() {
            "--instruments" => o.instruments = PathBuf::from(value()),
            "--out" => o.out = PathBuf::from(value()),
            "--table" => o.table = PathBuf::from(value()),
            "--a-max" => o.a_max = value().parse().ok().filter(|n| *n >= 1).unwrap_or_else(|| usage()),
            "--b-max" => o.b_max = value().parse().ok().filter(|n| *n >= 1).unwrap_or_else(|| usage()),
            "--range" => o.range = value().parse().ok().filter(|n| *n >= 1).unwrap_or_else(|| usage()),
            "--count" => o.count = value().parse().ok().filter(|n| *n >= 1).unwrap_or_else(|| usage()),
            "--alpha-bound" => o.alpha_bound = value().parse().ok().filter(|n| *n >= 2).unwrap_or_else(|| usage()),
            "--certify-below" => o.certify_below = value().parse().ok().filter(|n| *n >= 3).unwrap_or_else(|| usage()),
            "--threads" => o.threads = value().parse().ok().filter(|n| *n >= 1).unwrap_or_else(|| usage()),
            _ => usage(),
        }
    }
    o
}

/// The coprime pairs of the region, in d1-measure's order (b, then a).
fn region(a_max: i64, b_max: i64) -> Vec<(i64, i64)> {
    let mut out = Vec::new();
    for b in 1..=b_max {
        for a in -a_max..=a_max {
            if coprime(a, b) {
                out.push((a, b));
            }
        }
    }
    out
}

/// Mean ln|F(a, b)| over the pairs, skipping zero values as d1-measure does:
/// exact in i128 where every step fits, else in BigInt.
fn mean_ln(coeffs: &[BigInt], pairs: &[(i64, i64)]) -> f64 {
    let small: Option<Vec<i128>> = coeffs.iter().map(ToPrimitive::to_i128).collect();
    let hom = Homogeneous::new(coeffs);
    let d = coeffs.len() - 1;
    let exact = |a: i64, b: i64| -> Option<i128> {
        let c = small.as_ref()?;
        let (a, b) = (a as i128, b as i128);
        let mut bp = 1i128;
        let mut acc = c[d];
        for i in (0..d).rev() {
            bp = bp.checked_mul(b)?;
            acc = acc.checked_mul(a)?.checked_add(c[i].checked_mul(bp)?)?;
        }
        Some(acc)
    };
    let (mut sum, mut n) = (0.0f64, 0u64);
    for &(a, b) in pairs {
        let ln = match exact(a, b) {
            Some(0) => continue,
            Some(v) => (v as f64).abs().ln(),
            None => {
                let v = hom.eval(a, b);
                if v == BigInt::from(0) {
                    continue;
                }
                primness::dickman::ln_biguint(v.magnitude())
            }
        };
        sum += ln;
        n += 1;
    }
    sum / n as f64
}

/// Res(f, Y1·x + Y0) up to sign: the homogeneous F(−Y0, Y1).
fn resultant_with_linear(coeffs: &[BigInt], y0: &BigInt, y1: &BigInt) -> BigInt {
    let d = coeffs.len() - 1;
    let x = -y0;
    let mut acc = coeffs[d].clone();
    let mut y_pow = BigInt::from(1);
    for i in (0..d).rev() {
        y_pow *= y1;
        acc = acc * &x + &coeffs[i] * &y_pow;
    }
    acc
}

struct Candidate {
    v: i64,
    coeffs: Vec<BigInt>,
    mean_ln: f64,
    alpha: f64,
    certificate: Option<Vec<(u64, Vec<usize>)>>,
    resultant: BigInt,
}

impl Candidate {
    fn score(&self) -> f64 {
        self.mean_ln + self.alpha
    }
}

fn certificate_text(cert: &Option<Vec<(u64, Vec<usize>)>>) -> String {
    match cert {
        None => String::new(),
        Some(list) => list
            .iter()
            .map(|(p, d)| format!("{p}:{}", d.iter().map(|k| k.to_string()).collect::<Vec<_>>().join(".")))
            .collect::<Vec<_>>()
            .join(" "),
    }
}

fn write_text(path: &Path, text: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("{}: {err}", parent.display()))?;
    }
    fs::write(path, text).map_err(|err| format!("{}: {err}", path.display()))
}

fn main() {
    if let Err(err) = run(parse_args()) {
        eprintln!("d2-fields: {err}");
        std::process::exit(1);
    }
}

fn run(o: Options) -> Result<(), String> {
    let files = poly::instrument_files(&o.instruments)?;
    if files.is_empty() {
        return Err(format!("no polynomial files under {}", o.instruments.display()));
    }
    let pairs = region(o.a_max, o.b_max);
    println!(
        "region: a in [{}, {}], b in [1, {}], {} coprime pairs; rotations 0 < |v| <= {}, {} per instrument; alpha to {}; certified below {}",
        -o.a_max, o.a_max, o.b_max, pairs.len(), o.range, o.count, o.alpha_bound, o.certify_below
    );
    let mut table = String::from("instance,v,c0,c1,mean_ln_f,alpha,score,certified,certificate,rank,field\n");
    for (id, path) in &files {
        let text = fs::read_to_string(path).map_err(|err| format!("{}: {err}", path.display()))?;
        let p = poly::parse_instrument(&text).map_err(|err| format!("{}: {err:?}", path.display()))?;
        if p.rational.len() != 2 {
            return Err(format!("{}: the rational side is not linear", path.display()));
        }
        let (y0, y1) = (p.rational[0].clone(), p.rational[1].clone());
        let vs: Vec<i64> = (-o.range..=o.range).collect();
        let next = AtomicUsize::new(0);
        let mut cands: Vec<Candidate> = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..o.threads.min(vs.len()))
                .map(|_| {
                    scope.spawn(|| {
                        let mut out = Vec::new();
                        loop {
                            let i = next.fetch_add(1, Ordering::Relaxed);
                            let Some(&v) = vs.get(i) else { break };
                            let mut coeffs = p.algebraic.clone();
                            coeffs[0] += &y0 * v;
                            coeffs[1] += &y1 * v;
                            let resultant = resultant_with_linear(&coeffs, &y0, &y1);
                            out.push(Candidate {
                                v,
                                mean_ln: mean_ln(&coeffs, &pairs),
                                alpha: alpha::alpha(&coeffs, o.alpha_bound),
                                certificate: if v == 0 { None } else { fp::certify_irreducible(&coeffs, o.certify_below) },
                                resultant,
                                coeffs,
                            });
                        }
                        out
                    })
                })
                .collect();
            handles.into_iter().flat_map(|h| h.join().expect("candidate thread panicked")).collect()
        });
        cands.sort_by_key(|c| c.v);
        let base = cands.iter().find(|c| c.v == 0).expect("v = 0 is a candidate");
        if cands.iter().any(|c| c.resultant != base.resultant) {
            return Err(format!("{id}: a rotation changed the resultant with g"));
        }
        let n_big = BigInt::from(p.n.clone());
        if &base.resultant % &n_big != BigInt::from(0) {
            return Err(format!("{id}: N does not divide the resultant of f and g"));
        }
        let mut ranked: Vec<&Candidate> = cands.iter().filter(|c| c.v != 0 && c.certificate.is_some()).collect();
        ranked.sort_by(|x, y| {
            x.score()
                .total_cmp(&y.score())
                .then(x.v.abs().cmp(&y.v.abs()))
                .then(y.v.signum().cmp(&x.v.signum()))
        });
        let chosen: Vec<i64> = ranked.iter().take(o.count).map(|c| c.v).collect();
        if chosen.len() < o.count {
            return Err(format!("{id}: only {} certified rotations", chosen.len()));
        }
        let base_score = base.score();
        println!(
            "{id}: Res(f, g)/N = {}; instrument score {base_score:.3} (mean ln|F| {:.3}, alpha {:+.3}); {} of {} rotations certified irreducible",
            &base.resultant / &n_big,
            base.mean_ln,
            base.alpha,
            cands.iter().filter(|c| c.certificate.is_some()).count(),
            cands.len() - 1
        );
        for c in &cands {
            let rank = chosen.iter().position(|&v| v == c.v).map(|r| r + 1);
            let field = rank.map(|_| format!("{id}.r{}", c.v)).unwrap_or_default();
            table.push_str(&format!(
                "{id},{},{},{},{:.6},{:.6},{:.6},{},{},{},{field}\n",
                c.v,
                c.coeffs[0],
                c.coeffs[1],
                c.mean_ln,
                c.alpha,
                c.score(),
                if c.v == 0 { "instrument".to_string() } else { c.certificate.is_some().to_string() },
                certificate_text(&c.certificate),
                rank.map(|r| r.to_string()).unwrap_or_default(),
            ));
            let Some(rank) = rank else { continue };
            println!(
                "  {field}: score {:.3} ({:+.3} from the instrument; mean ln|F| {:.3}, alpha {:+.3}); irreducible by {}",
                c.score(),
                c.score() - base_score,
                c.mean_ln,
                c.alpha,
                certificate_text(&c.certificate)
            );
            let mut text = format!(
                "# D2 field (docs/d2_falsifiable_prediction.md): the instrument polynomial of {id} rotated by {v} times its rational polynomial, f + {v}*g.\n\
                 # d2-fields rank {rank} of the certified rotations with 0 < |v| <= {range}: score {score:.6} = mean ln|F| {m:.6} over a in [-{a}, {a}], b in [1, {b}] + alpha {al:+.6} (primes <= {ab}).\n\
                 # Irreducible over Q: factor degrees modulo {cert}.\n",
                v = c.v,
                range = o.range,
                score = c.score(),
                m = c.mean_ln,
                a = o.a_max,
                b = o.b_max,
                al = c.alpha,
                ab = o.alpha_bound,
                cert = certificate_text(&c.certificate),
            );
            text.push_str(&format!("n: {}\nY0: {}\nY1: {}\n", p.n, y0, y1));
            for (i, coeff) in c.coeffs.iter().enumerate() {
                text.push_str(&format!("c{i}: {coeff}\n"));
            }
            if let Some(skew) = &p.skew {
                text.push_str(&format!("skew: {skew}\n"));
            }
            let file = o.out.join(&field).join("cado.poly");
            let parsed = poly::parse_instrument(&text).map_err(|err| format!("{field}: {err:?}"))?;
            if parsed.algebraic != c.coeffs || parsed.rational != p.rational || parsed.n != p.n {
                return Err(format!("{field}: the written polynomial does not parse back"));
            }
            write_text(&file, &text)?;
        }
    }
    write_text(&o.table, &table)?;
    println!("wrote {} and the fields under {}", o.table.display(), o.out.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn big(c: &[i128]) -> Vec<BigInt> {
        c.iter().map(|&x| BigInt::from(x)).collect()
    }

    #[test]
    fn resultant_with_a_linear_form() {
        // Res(x^2 − 2, x − 1) = f(1) = −1; Res(f, 3x + 5) = 3^2 f(−5/3) = 25 − 18 = 7.
        assert_eq!(resultant_with_linear(&big(&[-2, 0, 1]), &BigInt::from(-1), &BigInt::from(1)), BigInt::from(-1));
        assert_eq!(resultant_with_linear(&big(&[-2, 0, 1]), &BigInt::from(5), &BigInt::from(3)), BigInt::from(7));
    }

    #[test]
    fn mean_log_is_exact_on_both_paths() {
        let pairs = region(40, 20);
        // Small coefficients take the i128 path, huge ones overflow into BigInt.
        for coeffs in [big(&[7, -3, 1, 0, 2]), big(&[i128::MAX / 3, 5, -1, 3, 1 << 100])] {
            let hom = Homogeneous::new(&coeffs);
            let (mut sum, mut n) = (0.0, 0u64);
            for &(a, b) in &pairs {
                let v = hom.eval(a, b);
                if v != BigInt::from(0) {
                    sum += primness::dickman::ln_biguint(v.magnitude());
                    n += 1;
                }
            }
            let got = mean_ln(&coeffs, &pairs);
            assert!((got - sum / n as f64).abs() < 1e-9, "{got} vs {}", sum / n as f64);
        }
        assert!(pairs.iter().all(|&(a, b)| coprime(a, b)) && pairs.len() > 0);
    }
}
