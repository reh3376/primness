//! h6-matrix: a synthetic sparse matrix over GF(2) shaped like a number field
//! sieve matrix, written in CADO-NFS's binary format, for H6
//! (docs/h6_falsifiable_prediction.md).
//!
//! `h6-matrix --rows N --density D --out DIR/NAME.bin [--seed S] [--threads T]
//!            [--sd-ratio R] [--alpha A] [--offset O]`
//!
//! The matrix is N × N. Each row's weight follows a gamma distribution with
//! mean D and standard deviation R · D (default 0.785, RSA-130's measured
//! ratio), rounded, at least 1. Each row's column positions are drawn without
//! repetition from P(i) ∝ (i + O)^-A (default A = 0.94, O = 32), the NFS-like
//! profile in CADO-NFS's `random_matrix` source, by inverse transform sampling
//! of its continuous approximation. The output is what `bwc` reads:
//!   NAME.bin     per row a little-endian u32 count, then that many u32 columns;
//!   NAME.rw.bin  u32 weight per row;
//!   NAME.cw.bin  u32 weight per column.
//! Rows are generated in chunks of 2^16 with a generator seeded by (S, chunk),
//! so the file does not depend on the thread count. A summary goes to stdout.

use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

const CHUNK: usize = 1 << 16;

fn usage() -> ! {
    eprintln!(
        "Usage: h6-matrix --rows N --density D --out DIR/NAME.bin [--seed S] [--threads T] \
         [--sd-ratio R] [--alpha A] [--offset O]"
    );
    std::process::exit(2);
}

#[derive(Clone, Copy, Debug)]
pub struct Shape {
    pub rows: usize,
    pub density: f64,
    pub sd_ratio: f64,
    pub alpha: f64,
    pub offset: f64,
}

/// xoshiro256** seeded through splitmix64.
pub struct Rng {
    s: [u64; 4],
}

fn splitmix64(z: &mut u64) -> u64 {
    *z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut x = *z;
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

impl Rng {
    pub fn new(seed: u64, stream: u64) -> Rng {
        let mut z = seed ^ stream.wrapping_mul(0xD1B5_4A32_D192_ED03);
        Rng { s: [splitmix64(&mut z), splitmix64(&mut z), splitmix64(&mut z), splitmix64(&mut z)] }
    }
    pub fn next_u64(&mut self) -> u64 {
        let r = self.s[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = self.s[1] << 17;
        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];
        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_left(45);
        r
    }
    /// Uniform in (0, 1).
    pub fn uniform(&mut self) -> f64 {
        ((self.next_u64() >> 11) as f64 + 0.5) * (1.0 / (1u64 << 53) as f64)
    }
    pub fn normal(&mut self) -> f64 {
        let (u, v) = (self.uniform(), self.uniform());
        (-2.0 * u.ln()).sqrt() * (2.0 * std::f64::consts::PI * v).cos()
    }
    /// Gamma(k, theta) for k >= 1 (Marsaglia and Tsang).
    pub fn gamma(&mut self, k: f64, theta: f64) -> f64 {
        let d = k - 1.0 / 3.0;
        let c = 1.0 / (9.0 * d).sqrt();
        loop {
            let z = self.normal();
            let v = (1.0 + c * z).powi(3);
            if v <= 0.0 {
                continue;
            }
            if self.uniform().ln() < 0.5 * z * z + d - d * v + d * v.ln() {
                return d * v * theta;
            }
        }
    }
}

/// Inverse transform for P(i) ∝ (i + offset)^-alpha on [0, n).
pub struct Columns {
    n: usize,
    offset: f64,
    beta: f64,
    lo: f64,
    span: f64,
}

impl Columns {
    pub fn new(n: usize, alpha: f64, offset: f64) -> Columns {
        let beta = 1.0 - alpha;
        let lo = offset.powf(beta);
        let span = (n as f64 + offset).powf(beta) - lo;
        Columns { n, offset, beta, lo, span }
    }
    pub fn sample(&self, rng: &mut Rng) -> u32 {
        let x = (rng.uniform() * self.span + self.lo).powf(1.0 / self.beta) - self.offset;
        (x.max(0.0) as usize).min(self.n - 1) as u32
    }
    /// The share of all nonzeros the law puts in columns [0, k).
    pub fn share_below(&self, k: usize) -> f64 {
        ((k as f64 + self.offset).powf(self.beta) - self.lo) / self.span
    }
}

/// One chunk of rows: the concatenated rows (count then columns) as u32.
pub fn chunk_rows(shape: &Shape, cols: &Columns, seed: u64, chunk: usize) -> Vec<u32> {
    let mut rng = Rng::new(seed, chunk as u64);
    let k = 1.0 / (shape.sd_ratio * shape.sd_ratio);
    let theta = shape.density / k;
    let first = chunk * CHUNK;
    let last = (first + CHUNK).min(shape.rows);
    let mut out = Vec::with_capacity((last - first) * (shape.density as usize + 1));
    let mut row: Vec<u32> = Vec::new();
    for _ in first..last {
        let w = (rng.gamma(k, theta).round() as usize).clamp(1, shape.rows);
        row.clear();
        while row.len() < w {
            while row.len() < w {
                row.push(cols.sample(&mut rng));
            }
            row.sort_unstable();
            row.dedup();
        }
        // CADO-NFS's own matrices do not keep a row's columns sorted; order is irrelevant to bwc.
        out.push(w as u32);
        out.extend_from_slice(&row);
    }
    out
}

pub fn weight_paths(out: &Path) -> (PathBuf, PathBuf) {
    let s = out.to_string_lossy();
    let stem = s.strip_suffix(".bin").unwrap_or(&s).to_string();
    (PathBuf::from(format!("{stem}.rw.bin")), PathBuf::from(format!("{stem}.cw.bin")))
}

pub struct Summary {
    pub nonzeros: u64,
    pub row_mean: f64,
    pub row_sd: f64,
    pub top1pct_share: f64,
    pub first_cols: Vec<u32>,
}

pub fn generate(shape: &Shape, seed: u64, threads: usize, out: &Path) -> std::io::Result<Summary> {
    let cols = Columns::new(shape.rows, shape.alpha, shape.offset);
    let chunks = shape.rows.div_ceil(CHUNK);
    let (rw_path, cw_path) = weight_paths(out);
    let mut mat = BufWriter::with_capacity(1 << 24, File::create(out)?);
    let mut rw = BufWriter::with_capacity(1 << 20, File::create(&rw_path)?);
    let mut cw = vec![0u32; shape.rows];
    let (mut nz, mut s1, mut s2) = (0u64, 0f64, 0f64);
    let mut next = 0;
    while next < chunks {
        let wave: Vec<usize> = (next..(next + threads).min(chunks)).collect();
        let bufs: Vec<Vec<u32>> = std::thread::scope(|s| {
            let hs: Vec<_> = wave.iter().map(|&c| { let cols = &cols; s.spawn(move || chunk_rows(shape, cols, seed, c)) }).collect();
            hs.into_iter().map(|h| h.join().expect("generator thread")).collect()
        });
        for buf in &bufs {
            let mut i = 0;
            while i < buf.len() {
                let w = buf[i] as usize;
                rw.write_all(&(w as u32).to_le_bytes())?;
                for &c in &buf[i + 1..i + 1 + w] {
                    cw[c as usize] += 1;
                }
                nz += w as u64;
                s1 += w as f64;
                s2 += (w * w) as f64;
                i += 1 + w;
            }
            let bytes: Vec<u8> = buf.iter().flat_map(|x| x.to_le_bytes()).collect();
            mat.write_all(&bytes)?;
        }
        next += wave.len();
    }
    mat.flush()?;
    rw.flush()?;
    let mut cwf = BufWriter::with_capacity(1 << 20, File::create(&cw_path)?);
    for &c in &cw {
        cwf.write_all(&c.to_le_bytes())?;
    }
    cwf.flush()?;
    let n = shape.rows as f64;
    let mean = s1 / n;
    let mut sorted = cw.clone();
    sorted.sort_unstable_by(|a, b| b.cmp(a));
    let top: u64 = sorted[..(shape.rows / 100).max(1)].iter().map(|&x| x as u64).sum();
    Ok(Summary {
        nonzeros: nz,
        row_mean: mean,
        row_sd: (s2 / n - mean * mean).max(0.0).sqrt(),
        top1pct_share: top as f64 / nz as f64,
        first_cols: cw[..8.min(shape.rows)].to_vec(),
    })
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut shape = Shape { rows: 0, density: 0.0, sd_ratio: 0.785, alpha: 0.94, offset: 32.0 };
    let mut out: Option<PathBuf> = None;
    let mut seed = 1u64;
    let mut threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
    let mut i = 1;
    while i < args.len() {
        let v = args.get(i + 1).cloned().unwrap_or_else(|| usage());
        match args[i].as_str() {
            "--rows" => shape.rows = v.parse().unwrap_or_else(|_| usage()),
            "--density" => shape.density = v.parse().unwrap_or_else(|_| usage()),
            "--sd-ratio" => shape.sd_ratio = v.parse().unwrap_or_else(|_| usage()),
            "--alpha" => shape.alpha = v.parse().unwrap_or_else(|_| usage()),
            "--offset" => shape.offset = v.parse().unwrap_or_else(|_| usage()),
            "--seed" => seed = v.parse().unwrap_or_else(|_| usage()),
            "--threads" => threads = v.parse().unwrap_or_else(|_| usage()),
            "--out" => out = Some(PathBuf::from(v)),
            _ => usage(),
        }
        i += 2;
    }
    let out = out.unwrap_or_else(|| usage());
    if shape.rows < 2 || shape.density < 1.0 || shape.sd_ratio <= 0.0 || shape.sd_ratio > 1.0
        || !(0.0..1.0).contains(&shape.alpha) || shape.rows > u32::MAX as usize
    {
        usage();
    }
    let t0 = Instant::now();
    let s = generate(&shape, seed, threads.max(1), &out).unwrap_or_else(|e| {
        eprintln!("h6-matrix: {}: {e}", out.display());
        std::process::exit(1);
    });
    let cols = Columns::new(shape.rows, shape.alpha, shape.offset);
    println!(
        "rows {} density {} seed {} nonzeros {} row_mean {:.2} row_sd {:.2} top1pct_share {:.4} (law {:.4}) first_cols {:?} seconds {:.1}",
        shape.rows,
        shape.density,
        seed,
        s.nonzeros,
        s.row_mean,
        s.row_sd,
        s.top1pct_share,
        cols.share_below((shape.rows / 100).max(1)),
        s.first_cols,
        t0.elapsed().as_secs_f64()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read_u32s(p: &Path) -> Vec<u32> {
        let b = std::fs::read(p).unwrap();
        b.chunks_exact(4).map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect()
    }

    #[test]
    fn writes_cado_binary_format_with_consistent_weights() {
        let dir = std::env::temp_dir().join(format!("h6-matrix-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let out = dir.join("m.bin");
        let shape = Shape { rows: 20_000, density: 60.0, sd_ratio: 0.785, alpha: 0.94, offset: 32.0 };
        let s = generate(&shape, 7, 3, &out).unwrap();
        let m = read_u32s(&out);
        let (rwp, cwp) = weight_paths(&out);
        let rw = read_u32s(&rwp);
        let cw = read_u32s(&cwp);
        assert_eq!(rw.len(), shape.rows);
        assert_eq!(cw.len(), shape.rows);
        let mut i = 0;
        let mut r = 0;
        let mut counted = vec![0u32; shape.rows];
        while i < m.len() {
            let w = m[i] as usize;
            assert_eq!(w as u32, rw[r]);
            let mut row = m[i + 1..i + 1 + w].to_vec();
            row.sort_unstable();
            let before = row.len();
            row.dedup();
            assert_eq!(row.len(), before, "row {r} repeats a column");
            for &c in &row {
                assert!((c as usize) < shape.rows);
                counted[c as usize] += 1;
            }
            i += 1 + w;
            r += 1;
        }
        assert_eq!(r, shape.rows);
        assert_eq!(counted, cw);
        assert_eq!(cw.iter().map(|&x| x as u64).sum::<u64>(), s.nonzeros);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn rows_and_columns_follow_the_requested_shape() {
        let dir = std::env::temp_dir().join(format!("h6-matrix-shape-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let out = dir.join("m.bin");
        let shape = Shape { rows: 100_000, density: 100.0, sd_ratio: 0.785, alpha: 0.94, offset: 32.0 };
        let s = generate(&shape, 1, 4, &out).unwrap();
        assert!((s.row_mean / 100.0 - 1.0).abs() < 0.02, "row mean {}", s.row_mean);
        assert!((s.row_sd / s.row_mean / 0.785 - 1.0).abs() < 0.05, "sd ratio {}", s.row_sd / s.row_mean);
        let law = Columns::new(shape.rows, shape.alpha, shape.offset).share_below(shape.rows / 100);
        assert!((s.top1pct_share - law).abs() < 0.03, "top 1% share {} against {}", s.top1pct_share, law);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn output_does_not_depend_on_threads() {
        let shape = Shape { rows: 3 * CHUNK + 17, density: 20.0, sd_ratio: 0.785, alpha: 0.94, offset: 32.0 };
        let cols = Columns::new(shape.rows, shape.alpha, shape.offset);
        let a: Vec<u32> = (0..shape.rows.div_ceil(CHUNK)).flat_map(|c| chunk_rows(&shape, &cols, 5, c)).collect();
        let dir = std::env::temp_dir().join(format!("h6-matrix-threads-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let out = dir.join("m.bin");
        generate(&shape, 5, 2, &out).unwrap();
        assert_eq!(read_u32s(&out), a);
        std::fs::remove_dir_all(&dir).ok();
    }
}
