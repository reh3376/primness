//! h2-cpu: the CPU side of H2 (docs/h2_falsifiable_prediction.md), and the
//! inputs both sides share.
//!
//! `h2-cpu --prepare DIR` writes `DIR/primes.bin` (every prime p in
//! (2^15, 2^27] with a root r = splitmix64(p) mod p, as little-endian u32
//! pairs) and `DIR/lanes.bin` (per lane: an odd 96-bit modulus n with its top
//! bit set, x0 and y below n, −n⁻¹ mod 2^32 and mod 2^64, and R² mod n for
//! R = 2^96 and R = 2^128; 112 bytes, little-endian).
//!
//! `h2-cpu --run DIR` times, on every thread of the machine (or `--threads`):
//!   sieve   one special-q's bucket sieve on one side: a line of 2^29
//!           positions, every r + k p receives ⌊log2 p⌋; fill into buckets
//!           of 2^16 positions, then apply as byte additions. The checksum is
//!           the sum over positions of the byte (mod 256 per position).
//!   mulmod  dependent chains x ← x · y mod n in Montgomery form (two 64-bit
//!           limbs, R = 2^128), K = 2^16 steps per lane; the results
//!           x0 · y^K mod n go to `DIR/cpu_lanes.bin` for the comparison with
//!           the GPU, and the first lanes are checked against num-bigint.
//!   copy    a 1 GiB copy, for the memory bandwidth reached.
//! Each kernel runs once to warm up and then `--repeats` times (default 5).
//! CSV lines on stdout: kernel,device,run,seconds,ops,ops_per_s,check.

use num_bigint::BigUint;
use num_traits::One;
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

pub const LINE_BITS: u32 = 29;
pub const P_MIN_BITS: u32 = 15;
pub const P_MAX_BITS: u32 = 27;
const REGION_BITS: u32 = 16;
pub const LANES: usize = 1 << 17;
pub const CPU_LANES_PER_THREAD: usize = 1024;
pub const K: u32 = 1 << 16;
const LANE_BYTES: usize = 112;

fn usage() -> ! {
    eprintln!("Usage: h2-cpu (--prepare DIR | --run DIR) [--threads N] [--repeats N]");
    std::process::exit(2);
}

fn splitmix64(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Every prime in (2^lo, 2^hi] with its root.
fn primes_with_roots(lo: u32, hi: u32) -> Vec<(u32, u32)> {
    let n = 1usize << hi;
    let mut composite = vec![false; n + 1];
    let mut i = 2;
    while i * i <= n {
        if !composite[i] {
            let mut j = i * i;
            while j <= n {
                composite[j] = true;
                j += i;
            }
        }
        i += 1;
    }
    ((1usize << lo) + 1..=n).filter(|&p| !composite[p]).map(|p| (p as u32, (splitmix64(p as u64) % p as u64) as u32)).collect()
}

fn inv_mod_2_64(n: u64) -> u64 {
    let mut x = n; // correct to 3 bits for odd n
    for _ in 0..6 {
        x = x.wrapping_mul(2u64.wrapping_sub(n.wrapping_mul(x)));
    }
    x
}

#[derive(Clone, Copy)]
pub struct Lane {
    pub n: u128,
    pub x0: u128,
    pub y: u128,
    pub np32: u32,
    pub np64: u64,
    pub r96sq: u128,
    pub r128sq: u128,
}

fn make_lane(i: usize) -> Lane {
    let h = |k: u64| splitmix64((i as u64) << 8 | k);
    let mask96 = (1u128 << 96) - 1;
    let n = (((h(1) as u128) << 64 | h(2) as u128) & mask96) | 1 | (1u128 << 95);
    let x0 = ((h(3) as u128) << 64 | h(4) as u128) % n;
    let y = ((h(5) as u128) << 64 | h(6) as u128) % n;
    let inv = inv_mod_2_64(n as u64);
    let nb = BigUint::from(n);
    let r96sq = ((BigUint::one() << 192u32) % &nb).to_u64_digits();
    let r128sq = ((BigUint::one() << 256u32) % &nb).to_u64_digits();
    let to_u128 = |d: Vec<u64>| d.iter().rev().fold(0u128, |acc, &w| acc << 64 | w as u128);
    Lane { n, x0, y, np32: (inv as u32).wrapping_neg(), np64: inv.wrapping_neg(), r96sq: to_u128(r96sq), r128sq: to_u128(r128sq) }
}

fn write_lanes(path: &Path, lanes: &[Lane]) -> std::io::Result<()> {
    let mut buf = Vec::with_capacity(lanes.len() * LANE_BYTES);
    for l in lanes {
        buf.extend_from_slice(&l.n.to_le_bytes());
        buf.extend_from_slice(&l.x0.to_le_bytes());
        buf.extend_from_slice(&l.y.to_le_bytes());
        buf.extend_from_slice(&l.np32.to_le_bytes());
        buf.extend_from_slice(&[0u8; 4]);
        buf.extend_from_slice(&l.np64.to_le_bytes());
        buf.extend_from_slice(&l.r96sq.to_le_bytes());
        buf.extend_from_slice(&l.r128sq.to_le_bytes());
        buf.extend_from_slice(&[0u8; 16]);
    }
    fs::write(path, buf)
}

fn read_lanes(path: &Path) -> Result<Vec<Lane>, String> {
    let b = fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if b.len() % LANE_BYTES != 0 {
        return Err(format!("{}: not a lane file", path.display()));
    }
    let u128_at = |o: usize| u128::from_le_bytes(b[o..o + 16].try_into().unwrap());
    Ok(b.chunks(LANE_BYTES)
        .enumerate()
        .map(|(i, _)| {
            let o = i * LANE_BYTES;
            Lane {
                n: u128_at(o),
                x0: u128_at(o + 16),
                y: u128_at(o + 32),
                np32: u32::from_le_bytes(b[o + 48..o + 52].try_into().unwrap()),
                np64: u64::from_le_bytes(b[o + 56..o + 64].try_into().unwrap()),
                r96sq: u128_at(o + 64),
                r128sq: u128_at(o + 80),
            }
        })
        .collect())
}

/// Montgomery product a b R⁻¹ mod n with R = 2^128, two 64-bit limbs; a, b < n < 2^127.
#[inline(always)]
pub fn mont_mul(a: u128, b: u128, n: u128, np: u64) -> u128 {
    let (a0, a1) = (a as u64 as u128, (a >> 64) as u64 as u128);
    let (n0, n1) = (n as u64 as u128, (n >> 64) as u64 as u128);
    let (mut t0, mut t1, mut t2) = (0u64, 0u64, 0u64);
    for bi in [b as u64 as u128, (b >> 64) as u64 as u128] {
        let p = a0 * bi + t0 as u128;
        t0 = p as u64;
        let p = a1 * bi + t1 as u128 + (p >> 64);
        t1 = p as u64;
        let s = t2 as u128 + (p >> 64);
        t2 = s as u64;
        let t3 = (s >> 64) as u64;
        let m = t0.wrapping_mul(np) as u128;
        let p = m * n0 + t0 as u128;
        let p = m * n1 + t1 as u128 + (p >> 64);
        t0 = p as u64;
        let s = t2 as u128 + (p >> 64);
        t1 = s as u64;
        t2 = t3 + (s >> 64) as u64;
    }
    let t = (t1 as u128) << 64 | t0 as u128;
    if t2 != 0 || t >= n {
        t.wrapping_sub(n)
    } else {
        t
    }
}

/// x0 · y^k mod n through Montgomery form.
pub fn chain(l: &Lane, k: u32) -> u128 {
    let mut x = mont_mul(l.x0, l.r128sq, l.n, l.np64);
    let y = mont_mul(l.y, l.r128sq, l.n, l.np64);
    for _ in 0..k {
        x = mont_mul(x, y, l.n, l.np64);
    }
    mont_mul(x, 1, l.n, l.np64)
}

/// Eight interleaved chains, for the throughput run.
fn chains8(ls: &[Lane], k: u32, out: &mut [u128]) {
    let mut x = [0u128; 8];
    let mut y = [0u128; 8];
    for j in 0..8 {
        x[j] = mont_mul(ls[j].x0, ls[j].r128sq, ls[j].n, ls[j].np64);
        y[j] = mont_mul(ls[j].y, ls[j].r128sq, ls[j].n, ls[j].np64);
    }
    for _ in 0..k {
        for j in 0..8 {
            x[j] = mont_mul(x[j], y[j], ls[j].n, ls[j].np64);
        }
    }
    for j in 0..8 {
        out[j] = mont_mul(x[j], 1, ls[j].n, ls[j].np64);
    }
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

fn sieve_once(primes: &[(u32, u32)], chunks: &[std::ops::Range<usize>], buckets: &mut [Vec<Vec<u32>>]) -> (f64, f64, u64, u64) {
    let line = 1u64 << LINE_BITS;
    let t0 = Instant::now();
    std::thread::scope(|s| {
        for (range, bk) in chunks.iter().zip(buckets.iter_mut()) {
            s.spawn(move || {
                for b in bk.iter_mut() {
                    b.clear();
                }
                for &(p, r) in &primes[range.clone()] {
                    let logp = 31 - p.leading_zeros();
                    let (p, mut x) = (p as u64, r as u64);
                    while x < line {
                        bk[(x >> REGION_BITS) as usize].push((x as u32 & 0xffff) | logp << 16);
                        x += p;
                    }
                }
            });
        }
    });
    let fill = t0.elapsed().as_secs_f64();
    let regions = 1usize << (LINE_BITS - REGION_BITS);
    let threads = buckets.len();
    let buckets = &*buckets;
    let t1 = Instant::now();
    let parts: Vec<(u64, u64)> = std::thread::scope(|s| {
        let hs: Vec<_> = (0..threads)
            .map(|t| {
                s.spawn(move || {
                    let mut arr = vec![0u8; 1 << REGION_BITS];
                    let (mut sum, mut count) = (0u64, 0u64);
                    for region in (t..regions).step_by(threads) {
                        arr.fill(0);
                        for bk in buckets {
                            for &u in &bk[region] {
                                let off = (u & 0xffff) as usize;
                                arr[off] = arr[off].wrapping_add((u >> 16) as u8);
                            }
                            count += bk[region].len() as u64;
                        }
                        sum += arr.iter().map(|&x| x as u64).sum::<u64>();
                    }
                    (sum, count)
                })
            })
            .collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let apply = t1.elapsed().as_secs_f64();
    (fill, apply, parts.iter().map(|x| x.1).sum(), parts.iter().map(|x| x.0).sum())
}

fn run(dir: &Path, threads: usize, repeats: usize) -> Result<(), String> {
    let pb = fs::read(dir.join("primes.bin")).map_err(|e| format!("primes.bin: {e} (run --prepare first)"))?;
    let primes: Vec<(u32, u32)> =
        pb.chunks(8).map(|c| (u32::from_le_bytes(c[0..4].try_into().unwrap()), u32::from_le_bytes(c[4..8].try_into().unwrap()))).collect();
    // chunks of equal work Σ 2^29 / p
    let line = (1u64 << LINE_BITS) as f64;
    let total: f64 = primes.iter().map(|&(p, _)| line / p as f64).sum();
    let mut chunks = Vec::new();
    let (mut start, mut acc) = (0usize, 0.0);
    for (i, &(p, _)) in primes.iter().enumerate() {
        acc += line / p as f64;
        if acc >= total * (chunks.len() + 1) as f64 / threads as f64 && chunks.len() + 1 < threads {
            chunks.push(start..i + 1);
            start = i + 1;
        }
    }
    chunks.push(start..primes.len());
    let regions = 1usize << (LINE_BITS - REGION_BITS);
    let per = (total / regions as f64 / threads as f64 * 1.3) as usize + 256;
    let mut buckets: Vec<Vec<Vec<u32>>> = (0..chunks.len()).map(|_| (0..regions).map(|_| Vec::with_capacity(per)).collect()).collect();
    let out = std::io::stdout();
    let mut out = out.lock();
    let mut times = Vec::new();
    for run in 0..=repeats {
        let (fill, apply, count, sum) = sieve_once(&primes, &chunks, &mut buckets);
        let secs = fill + apply;
        if run > 0 {
            times.push(secs);
        }
        writeln!(out, "sieve,cpu,{run},{secs:.6},{count},{:.6e},{sum}", count as f64 / secs).unwrap();
        writeln!(out, "sieve-fill,cpu,{run},{fill:.6},{count},{:.6e},{sum}", count as f64 / fill).unwrap();
        writeln!(out, "sieve-apply,cpu,{run},{apply:.6},{count},{:.6e},{sum}", count as f64 / apply).unwrap();
    }
    eprintln!("sieve: median {:.3} s over {} runs", median(times), repeats);
    drop(buckets);

    let lanes = read_lanes(&dir.join("lanes.bin"))?;
    // correctness against num-bigint on the first lanes, with a short chain
    for l in lanes.iter().take(8) {
        let kk = 1000u32;
        let want = (BigUint::from(l.x0) * BigUint::from(l.y).modpow(&BigUint::from(kk), &BigUint::from(l.n))) % BigUint::from(l.n);
        let got = BigUint::from(chain(l, kk));
        if got != want {
            return Err(format!("Montgomery chain disagrees with num-bigint: {got} != {want}"));
        }
    }
    let nl = threads * CPU_LANES_PER_THREAD;
    let mut results = vec![0u128; nl];
    let mut times = Vec::new();
    for run in 0..=repeats {
        let t0 = Instant::now();
        std::thread::scope(|s| {
            for (t, res) in results.chunks_mut(CPU_LANES_PER_THREAD).enumerate() {
                let ls = &lanes[t * CPU_LANES_PER_THREAD..(t + 1) * CPU_LANES_PER_THREAD];
                s.spawn(move || {
                    for (g, r) in ls.chunks(8).zip(res.chunks_mut(8)) {
                        chains8(g, K, r);
                    }
                });
            }
        });
        let secs = t0.elapsed().as_secs_f64();
        let ops = nl as u64 * K as u64;
        let check = results.iter().fold(0u128, |a, &x| a ^ x.rotate_left((x % 127) as u32)) as u64;
        if run > 0 {
            times.push(secs);
        }
        writeln!(out, "mulmod,cpu,{run},{secs:.6},{ops},{:.6e},{check}", ops as f64 / secs).unwrap();
    }
    eprintln!("mulmod: median {:.3} s over {} runs", median(times), repeats);
    let mut buf = Vec::with_capacity(nl * 16);
    for r in &results {
        buf.extend_from_slice(&r.to_le_bytes());
    }
    fs::write(dir.join("cpu_lanes.bin"), buf).map_err(|e| e.to_string())?;

    // memory bandwidth: copy 1 GiB
    let n = 1usize << 27;
    let src: Vec<u64> = (0..n as u64).collect();
    let mut dst = vec![0u64; n];
    for run in 0..=repeats {
        let t0 = Instant::now();
        std::thread::scope(|s| {
            let per = n / threads + 1;
            for (d, sc) in dst.chunks_mut(per).zip(src.chunks(per)) {
                s.spawn(move || d.copy_from_slice(sc));
            }
        });
        let secs = t0.elapsed().as_secs_f64();
        let bytes = 2 * n as u64 * 8;
        writeln!(out, "copy,cpu,{run},{secs:.6},{bytes},{:.6e},{}", bytes as f64 / secs, dst[n - 1]).unwrap();
    }
    Ok(())
}

fn prepare(dir: &Path) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let primes = primes_with_roots(P_MIN_BITS, P_MAX_BITS);
    let mut buf = Vec::with_capacity(primes.len() * 8);
    for &(p, r) in &primes {
        buf.extend_from_slice(&p.to_le_bytes());
        buf.extend_from_slice(&r.to_le_bytes());
    }
    fs::write(dir.join("primes.bin"), buf).map_err(|e| e.to_string())?;
    let lanes: Vec<Lane> = (0..LANES).map(make_lane).collect();
    write_lanes(&dir.join("lanes.bin"), &lanes).map_err(|e| e.to_string())?;
    let updates: u64 = primes.iter().map(|&(p, r)| ((1u64 << LINE_BITS) - r as u64).div_ceil(p as u64)).sum();
    eprintln!("{} primes in (2^{P_MIN_BITS}, 2^{P_MAX_BITS}], {updates} updates per line of 2^{LINE_BITS}; {LANES} lanes", primes.len());
    Ok(())
}

fn main() {
    let mut mode = None;
    let mut threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
    let mut repeats = 5;
    let mut args = env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--prepare" => mode = Some(("prepare", args.next().map(PathBuf::from).unwrap_or_else(|| usage()))),
            "--run" => mode = Some(("run", args.next().map(PathBuf::from).unwrap_or_else(|| usage()))),
            "--threads" => threads = args.next().and_then(|v| v.parse().ok()).filter(|&n| n >= 1).unwrap_or_else(|| usage()),
            "--repeats" => repeats = args.next().and_then(|v| v.parse().ok()).filter(|&n| n >= 1).unwrap_or_else(|| usage()),
            _ => usage(),
        }
    }
    let Some((m, dir)) = mode else { usage() };
    let r = if m == "prepare" { prepare(&dir) } else { run(&dir, threads, repeats) };
    if let Err(e) = r {
        eprintln!("h2-cpu: {e}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_traits::Zero;

    #[test]
    fn montgomery_chain_matches_bigint() {
        for i in 0..16 {
            let l = make_lane(i);
            let k = 300 + i as u32;
            let n = BigUint::from(l.n);
            let want = (BigUint::from(l.x0) * BigUint::from(l.y).modpow(&BigUint::from(k), &n)) % &n;
            assert_eq!(BigUint::from(chain(&l, k)), want);
        }
    }

    #[test]
    fn lanes_round_trip_and_constants() {
        let lanes: Vec<Lane> = (0..4).map(make_lane).collect();
        let dir = std::env::temp_dir().join(format!("h2-cpu-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        write_lanes(&dir.join("lanes.bin"), &lanes).unwrap();
        let back = read_lanes(&dir.join("lanes.bin")).unwrap();
        fs::remove_dir_all(&dir).ok();
        for (a, b) in lanes.iter().zip(&back) {
            assert_eq!((a.n, a.x0, a.y, a.np32, a.np64, a.r96sq, a.r128sq), (b.n, b.x0, b.y, b.np32, b.np64, b.r96sq, b.r128sq));
            assert_eq!((a.n as u64).wrapping_mul(a.np64.wrapping_neg()), 1);
            assert_eq!((a.n as u32).wrapping_mul(a.np32.wrapping_neg()), 1);
            assert!(a.n >> 95 == 1 && a.n & 1 == 1 && a.x0 < a.n && a.y < a.n);
            let n = BigUint::from(a.n);
            assert_eq!(BigUint::from(a.r96sq), (BigUint::one() << 192u32) % &n);
            assert!(!BigUint::from(a.r128sq).is_zero());
        }
    }

    #[test]
    fn primes_and_roots() {
        let ps = primes_with_roots(4, 8);
        assert_eq!(ps.iter().map(|x| x.0).collect::<Vec<_>>()[..5], [17, 19, 23, 29, 31]);
        assert!(ps.iter().all(|&(p, r)| r < p));
        assert_eq!(ps.last().unwrap().0, 251);
    }
}
