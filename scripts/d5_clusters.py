#!/usr/bin/env python3
"""D5: are the B-smooth values clustered in the sieve region beyond their prediction?

    scripts/d5_clusters.py [--dumps data/dumps] [--only d1-01,d1-02] [--out data/results/d5_clusters]

docs/d5_falsifiable_prediction.md fixes the protocol and the rejection
criterion before measuring; docs/d5_smooth_clusters.md records the result.

Per instance it reads the per-value dump of d1-measure at the bounds 2^20,
2^24, 2^28 (<dumps>/<id>.values.csv: a, b, ln|F|, the largest prime factor
of F when F is smooth at the top bound or 0, the same for G, and the logs of
the parts of F and G made of the primes up to 2000). Every value's smooth
probability at every bound is the one scripts/d1_conditional.py sums for a
whole instance: the conditional baseline Q_B at the value's own ln c plus the
first-order root-count term of its cofactor bin. The script sums, per side,
bound, region of each partition and half, the values' smooth count O, the
prediction P and its Bernoulli variance V, and from those the dispersion,
the split-half correlation and the held-out lift.

Regions: the 50 x 50 grid of equal ranges of a and of b, the coarser n x n
tilings as unions of its tiles, and 64 sectors of equal angle. Halves: a
fixed 64-bit hash of (a, b). Writes <out>.csv and <out>.md. Standard
library only.
"""
import csv
import math
import sys
from pathlib import Path

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent))
import d1_conditional as d1  # noqa: E402

GRID = 50
TILINGS = (2, 5, 10, 25, 50)
SECTORS = 64
PROTOCOL = (1 << 20, 1 << 24, 1 << 28)
SIDES = ("algebraic", "rational")
MASK = (1 << 64) - 1


def half_of(a, b):
    """Half 0 or 1 from the splitmix64 finalizer of (a, b): a function of the
    position that ignores the arithmetic of the values."""
    z = ((((a & 0xFFFFFFFF) << 32) | (b & 0xFFFFFFFF)) + 0x9E3779B97F4A7C15) & MASK
    z = ((z ^ (z >> 30)) * 0xBF58476D1CE4E5B9) & MASK
    z = ((z ^ (z >> 27)) * 0x94D049BB133111EB) & MASK
    return (z ^ (z >> 31)) & 1


def prediction_grid(qb, b, width, s_by_bin, top):
    """Smooth probability and its Bernoulli variance on cells of ln c of an
    eighth of a cofactor bin, read at each cell's middle: cell edges include
    every bin edge and ln B, so no cell straddles the jump at c = B."""
    h = width / 8
    q = d1.q_function(qb, b, width, h)
    ps, vs = [], []
    for i in range(int(top / h) + 2):
        z = (i + 0.5) * h
        qz = q(z)
        if qz is None:
            ps.append(math.nan)
            vs.append(math.nan)
            continue
        k = 0 if z < 0.5 else int(z / width)
        p = min(1.0, max(0.0, qz + s_by_bin.get(k, 0.0)))
        ps.append(p)
        vs.append(p * (1.0 - p))
    return ps, vs


def accumulate(path, grids, bounds, a_max, b_max, width):
    """Sums per side, bound, half and region (fine tiles then sectors)."""
    regions = GRID * GRID + SECTORS
    nb = len(bounds)
    size = 2 * nb * 2 * regions
    O, P, V = [0] * size, [0.0] * size, [0.0] * size
    inv_h = 8 / width
    na, nbv = 2 * a_max + 1, b_max
    values = 0
    with open(path) as fh:
        header = fh.readline().strip().split(",")
        if header[:8] != ["a", "b", "ln_f", "lpf_f", "ln_g", "lpf_g", "ln_sf", "ln_sg"]:
            sys.exit(f"{path}: not a d1-measure dump with the small-part columns (rerun with the current d1-measure)")
        for line in fh:
            a, b, ln_f, lpf_f, ln_g, lpf_g, ln_sf, ln_sg = line.split(",")
            a, b = int(a), int(b)
            values += 1
            hh = half_of(a, b)
            cell = ((a + a_max) * GRID // na) * GRID + (b - 1) * GRID // nbv
            sector = GRID * GRID + min(SECTORS - 1, int(math.atan2(b, a) * SECTORS / math.pi))
            for s, ln_v, lpf, ln_s in ((0, ln_f, lpf_f, ln_sf), (1, ln_g, lpf_g, ln_sg)):
                ln_c = float(ln_v) - float(ln_s)
                idx = int(ln_c * inv_h) if ln_c > 0 else 0
                lpf = int(lpf)
                gp, gv = grids[s]
                for j in range(nb):
                    p, v = gp[j][idx], gv[j][idx]
                    smooth = 1 if 0 < lpf <= bounds[j] else 0
                    base = ((s * nb + j) * 2 + hh) * regions
                    i1, i2 = base + cell, base + sector
                    O[i1] += smooth
                    P[i1] += p
                    V[i1] += v
                    O[i2] += smooth
                    P[i2] += p
                    V[i2] += v
    return O, P, V, values


def partitions(O, P, V, s, j, nb):
    """{name: [(O0, P0, V0, O1, P1, V1) per region]} for one side and bound."""
    regions = GRID * GRID + SECTORS
    halves = []
    for hh in (0, 1):
        base = ((s * nb + j) * 2 + hh) * regions
        halves.append((O[base:base + regions], P[base:base + regions], V[base:base + regions]))
    out = {}
    for n in TILINGS:
        f = GRID // n
        acc = [[0, 0.0, 0.0, 0, 0.0, 0.0] for _ in range(n * n)]
        for cell in range(GRID * GRID):
            r = ((cell // GRID) // f) * n + (cell % GRID) // f
            for hh in (0, 1):
                o, p, v = halves[hh]
                acc[r][3 * hh] += o[cell]
                acc[r][3 * hh + 1] += p[cell]
                acc[r][3 * hh + 2] += v[cell]
        out[f"{n}x{n} tiles"] = [tuple(x) for x in acc]
    out[f"{SECTORS} sectors"] = [
        (halves[0][0][i], halves[0][1][i], halves[0][2][i], halves[1][0][i], halves[1][1][i], halves[1][2][i])
        for i in range(GRID * GRID, regions)]
    return out


def pearson(xs, ys):
    n = len(xs)
    if n < 3:
        return math.nan
    mx, my = sum(xs) / n, sum(ys) / n
    sxy = sum((x - mx) * (y - my) for x, y in zip(xs, ys))
    sxx, syy = sum((x - mx) ** 2 for x in xs), sum((y - my) ** 2 for y in ys)
    return sxy / math.sqrt(sxx * syy) if sxx > 0 and syy > 0 else math.nan


def lift_one(sel_z, O, P, V):
    """Pick the tenth of the regions with the largest sel_z; on (O, P, V),
    the ratio O/P there over the ratio over all regions, minus 1, and its
    error with the picked and the other regions independent."""
    r = len(sel_z)
    m = max(1, round(r / 10))
    pick = set(sorted(range(r), key=lambda i: -sel_z[i])[:m])
    os_, ps, vs = (sum(x[i] for i in pick) for x in (O, P, V))
    orr, pr, vr = (sum(x[i] for i in range(r) if i not in pick) for x in (O, P, V))
    if ps <= 0 or pr <= 0 or os_ + orr == 0:
        return math.nan, math.nan
    w = ps / (ps + pr)
    lift = (os_ / ps) / ((os_ + orr) / (ps + pr)) - 1.0
    err = (1.0 - w) * math.sqrt(vs / ps ** 2 + vr / pr ** 2)
    return lift, err


def region_stats(regs):
    use = [r for r in regs if r[1] > 0 and r[2] > 0 and r[4] > 0 and r[5] > 0]
    out = dict(regions=len(use), observed=sum(r[0] + r[3] for r in regs), predicted=sum(r[1] + r[4] for r in regs))
    if len(use) < 2:
        return dict(out, chi2=math.nan, corr=math.nan, lift=math.nan, lift_err=math.nan)
    out["chi2"] = sum((r[0] + r[3] - r[1] - r[4]) ** 2 / (r[2] + r[5]) for r in use)
    z0 = [(r[0] - r[1]) / math.sqrt(r[2]) for r in use]
    z1 = [(r[3] - r[4]) / math.sqrt(r[5]) for r in use]
    out["corr"] = pearson(z0, z1)
    l01, e01 = lift_one(z0, [r[3] for r in use], [r[4] for r in use], [r[5] for r in use])
    l10, e10 = lift_one(z1, [r[0] for r in use], [r[1] for r in use], [r[2] for r in use])
    out["lift"] = 0.5 * (l01 + l10)
    out["lift_err"] = 0.5 * math.sqrt(e01 ** 2 + e10 ** 2)
    return out


def main(argv):
    dumps, only, out = Path("data/dumps"), None, "data/results/d5_clusters"
    i = 1
    while i < len(argv):
        if argv[i] == "--dumps":
            dumps = Path(argv[i + 1]); i += 2
        elif argv[i] == "--only":
            only = set(argv[i + 1].split(",")); i += 2
        elif argv[i] == "--out":
            out = argv[i + 1]; i += 2
        else:
            sys.exit("usage: d5_clusters.py [--dumps DIR] [--only IDS] [--out PATH]")
    base, q = d1.load_baseline("data/results/baseline.consecutive.json")
    rough = base["cofactor"]["rough_bound"]
    width = base["cofactor"]["bin_width"]
    tails, cover = d1.load_root_tail("data/results/root_tail.json", rough)
    if not tails:
        sys.exit("data/results/root_tail.json is missing: run d1-tail first")
    omega = d1.buchstab_omega()
    bounds = sorted(b for b in q if b <= cover)
    top = (max(max(qb) for qb in q.values()) + 2) * width

    rows = []
    for dump in sorted(dumps.glob("d1-*.values.csv")):
        inst = dump.name[: -len(".values.csv")]
        if "." in inst or (only and inst not in only):
            continue  # <id>.values.csv only: older dumps under other names lack the small-part columns
        hist, region = {}, None
        for f in sorted(Path("data/results").glob(f"{inst}.scaled-down*.json")):
            result, h = d1.load_instance(f)
            if result is None:
                continue
            region = region or result["region"]
            for key, bins in h.items():
                hist.setdefault(key, bins)
        if region is None:
            sys.exit(f"{inst}: no d1-measurement/4 result")
        grids = []
        for side in SIDES:
            gp, gv = [], []
            for b in bounds:
                s_by_bin = d1.first_order_by_bin(q[b], hist[(side, b)], tails[(inst, side)], b, width, rough, omega,
                                                 region["a_max"], region["b_max"])
                p, v = prediction_grid(q[b], b, width, s_by_bin, top)
                gp.append(p)
                gv.append(v)
            grids.append((gp, gv))
        O, P, V, values = accumulate(dump, grids, bounds, region["a_max"], region["b_max"], width)
        print(f"{inst}: {values:,} values", flush=True)
        for s, side in enumerate(SIDES):
            for j, b in enumerate(bounds):
                for name, regs in partitions(O, P, V, s, j, len(bounds)).items():
                    st = region_stats(regs)
                    if math.isnan(st["predicted"]):
                        sys.exit(f"{inst} {side} {b}: a value lies outside the baseline table")
                    rows.append(dict(instance=inst, side=side, b=b, partition=name, **st))
    if not rows:
        sys.exit(f"no dumps under {dumps}: run d1-measure --dump (docs/d5_smooth_clusters.md)")

    with open(out + ".csv", "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=list(rows[0].keys()))
        w.writeheader()
        for r in rows:
            w.writerow({k: (f"{v:.6g}" if isinstance(v, float) else v) for k, v in r.items()})
    write_summary(rows, out)
    print(f"wrote {out}.csv ({len(rows)} rows) and {out}.md")


def mean_and_spread(xs):
    n = len(xs)
    m = sum(xs) / n
    return m, (math.sqrt(sum((x - m) ** 2 for x in xs) / (n - 1)) / math.sqrt(n) if n > 1 else math.nan)


def aggregate(rows):
    """Over instances: the mean lift, split-half correlation and dispersion per
    region, each with the larger of its error under counting noise and the
    spread across instances over sqrt(n). The spread matters where the counts
    are small and one region can carry an instance's chi-square."""
    n = len(rows)
    lift, lift_sd = mean_and_spread([r["lift"] for r in rows])
    lift_err = max(lift_sd, sum(r["lift_err"] for r in rows) / n / math.sqrt(n))
    corr, corr_sd = mean_and_spread([r["corr"] for r in rows])
    corr_err = max(corr_sd, math.sqrt(sum(1.0 / max(r["regions"] - 1, 1) for r in rows)) / n)
    disp, disp_sd = mean_and_spread([r["chi2"] / r["regions"] for r in rows])
    disp_err = max(disp_sd, math.sqrt(sum(2.0 / r["regions"] for r in rows)) / n)
    return dict(lift=lift, lift_err=lift_err, corr=corr, corr_err=corr_err, disp=disp, disp_err=disp_err, n=n)


def write_summary(rows, out):
    label = lambda b: f"2^{b.bit_length() - 1}"
    pct = lambda x: f"{100 * x:+.2f}%"
    bounds = sorted({r["b"] for r in rows})
    names = list(dict.fromkeys(r["partition"] for r in rows))
    instances = sorted({r["instance"] for r in rows})
    agg = {}
    for side in SIDES:
        for b in bounds:
            for name in names:
                rs = [r for r in rows if r["side"] == side and r["b"] == b and r["partition"] == name]
                agg[(side, b, name)] = aggregate(rs)
    L = ["# D5: smooth values region by region", ""]
    L.append(f"{len(instances)} instances, the D1 region. Each value's smooth probability is the conditional baseline Q_B at its cofactor plus the first-order root-count term (`scripts/d1_conditional.py`); the protocol and the criterion are `docs/d5_falsifiable_prediction.md`, the reading `docs/d5_smooth_clusters.md`. Rows per instance, side, bound and partition are in `{out}.csv`.")
    L.append("")
    for side in SIDES:
        L.append(f"## {side.capitalize()} side")
        L.append("")
        L.append("**Held-out lift**: the tenth of the regions with the largest excess in one half, read on the other half against that half as a whole, mean of both directions and of the ten instances, ± the larger of the spread and the noise over √10.")
        L.append("")
        L.append("| partition | " + " | ".join(label(b) for b in bounds) + " |")
        L.append("|---|" + "---:|" * len(bounds))
        for name in names:
            L.append(f"| {name} | " + " | ".join(f"{pct(agg[(side, b, name)]['lift'])} ± {100 * agg[(side, b, name)]['lift_err']:.2f}%" for b in bounds) + " |")
        L.append("")
        L.append("**Split-half correlation** of the regions' excess, mean over instances ± the larger of its error with no correlation and the spread over √10:")
        L.append("")
        L.append("| partition | " + " | ".join(label(b) for b in bounds) + " |")
        L.append("|---|" + "---:|" * len(bounds))
        for name in names:
            L.append(f"| {name} | " + " | ".join(f"{agg[(side, b, name)]['corr']:+.3f} ± {agg[(side, b, name)]['corr_err']:.3f}" for b in bounds) + " |")
        L.append("")
        L.append("**Dispersion**: χ² per region, mean over instances (1 for counting noise) ± the larger of its error under counting noise and the spread over √10:")
        L.append("")
        L.append("| partition | " + " | ".join(label(b) for b in bounds) + " |")
        L.append("|---|" + "---:|" * len(bounds))
        for name in names:
            L.append(f"| {name} | " + " | ".join(f"{agg[(side, b, name)]['disp']:.2f} ± {agg[(side, b, name)]['disp_err']:.2f}" for b in bounds) + " |")
        L.append("")
    L.append("## The criterion")
    L.append("")
    L.append("At each protocol bound and partition: the algebraic mean lift over its error, and the algebraic minus the rational lift over their combined error. The claim is supported only where both exceed 3.")
    L.append("")
    L.append("| partition | " + " | ".join(f"{label(b)} algebraic | {label(b)} minus control" for b in PROTOCOL) + " |")
    L.append("|---|" + "---:|---:|" * len(PROTOCOL))
    support = []
    for name in names:
        cells = []
        for b in PROTOCOL:
            a, r = agg[("algebraic", b, name)], agg[("rational", b, name)]
            za = a["lift"] / a["lift_err"]
            zd = (a["lift"] - r["lift"]) / math.hypot(a["lift_err"], r["lift_err"])
            cells.append(f"{za:+.1f} | {zd:+.1f}")
            if za > 3 and zd > 3:
                support.append((name, b, a["lift"]))
        L.append(f"| {name} | " + " | ".join(cells) + " |")
    L.append("")
    if support:
        L.append("**Supported** at " + ", ".join(f"{n} at {label(b)} (lift {pct(x)})" for n, b, x in support) + ".")
    else:
        L.append("**Rejected**: at no protocol bound and partition does the algebraic lift exceed three standard errors and the control by three combined standard errors.")
    Path(out + ".md").write_text("\n".join(L) + "\n")


if __name__ == "__main__":
    main(sys.argv)
