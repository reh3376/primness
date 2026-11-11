#!/usr/bin/env python3
"""Obstacle D, the closing checks: is anything left in what D1, D2 part 1 and D5 measured?

    scripts/d_checks.py [--out data/results/d_checks] [--skip-run]

docs/obstacle_d_verdict.md records the reading. Four parts:

1. Pooled statistics. Every per-instance statistic of D1 (summary.conditional.csv,
   after the first-order term), D2 part 1 (data/results/d2/joint: each column against
   its own prediction, each class and the union) and D5 (d5_clusters.csv: held-out
   lift and the regions' dispersion) as z = deviation / error, per family and bound:
   mean, mean square, Kolmogorov-Smirnov against N(0, 1), counts beyond 2 and 3. A
   weak excess spread over many tests shifts the mean; a real one in a few fattens
   the tails.
2. Per-value checks (binary d-checks over the dumps, values smooth for certain left
   out): calibration of the prediction in u = ln c / ln B, and smoothness at fixed
   cofactor against the part made of the primes up to 2000, the other side's small
   part and cofactor, and (a, b) mod 2 and mod 3; neighbouring values smooth together.
3. Each polynomial's gap against the digits of N, extrapolated to 200 and 250 digits.
4. The five toy quartics of the structure ladder (Galois groups A4, S4, C4, D4, V4)
   against the same model, with and without the root-count term.

Writes <out>.md, <out>.csv (one row per statistic) and <out>.sums.csv (d-checks'
sums). The manifest goes to data/dumps/d_checks/. Standard library only.
"""
import csv
import json
import math
import re
import subprocess
import sys
from collections import defaultdict
from pathlib import Path

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent))
import d1_conditional as d1  # noqa: E402
import d5_clusters as d5  # noqa: E402

CHECKS = "target/release/d-checks"
BOUNDS = (1 << 14, 1 << 16, 1 << 18, 1 << 20, 1 << 24, 1 << 28)
GALOIS = {"a4": ("A4", 1), "s4": ("S4", 1), "d4": ("D4", 2), "c4": ("C4", 3), "v4": ("V4", 3)}
U_WIDTH = 0.25


def lab(b):
    return f"2^{b.bit_length() - 1}"


def pct(x, e=None, digits=2):
    if x is None or (isinstance(x, float) and math.isnan(x)):
        return "n/a"
    s = f"{100 * x:+.{digits}f}%"
    return s if e is None else f"{s} ± {100 * e:.{digits}f}%"


# ---------------------------------------------------------------- statistics


def phi(x):
    return 0.5 * math.erfc(-x / math.sqrt(2.0))


def ks_pvalue(zs):
    """Kolmogorov-Smirnov test of zs against N(0, 1): the asymptotic p-value
    with Stephens' correction."""
    xs = sorted(zs)
    n = len(xs)
    d = max(max((i + 1) / n - phi(x), phi(x) - i / n) for i, x in enumerate(xs))
    lam = (math.sqrt(n) + 0.12 + 0.11 / math.sqrt(n)) * d
    p = 2.0 * sum((-1) ** (k - 1) * math.exp(-2.0 * k * k * lam * lam) for k in range(1, 101))
    return min(1.0, max(0.0, p))


def gamma_q(a, x):
    """Regularized upper incomplete gamma Q(a, x)."""
    if x <= 0:
        return 1.0
    gln = math.lgamma(a)
    if x < a + 1.0:
        term = total = 1.0 / a
        ap = a
        for _ in range(1000):
            ap += 1.0
            term *= x / ap
            total += term
            if abs(term) < abs(total) * 1e-15:
                break
        return 1.0 - total * math.exp(-x + a * math.log(x) - gln)
    b = x + 1.0 - a
    c = 1.0 / 1e-300
    d = 1.0 / b
    h = d
    for i in range(1, 1000):
        an = -i * (i - a)
        b += 2.0
        d = an * d + b
        d = 1e-300 if abs(d) < 1e-300 else d
        c = b + an / c
        c = 1e-300 if abs(c) < 1e-300 else c
        d = 1.0 / d
        delta = d * c
        h *= delta
        if abs(delta - 1.0) < 1e-15:
            break
    return math.exp(-x + a * math.log(x) - gln) * h


def chi2_sf(chi2, dof):
    return gamma_q(dof / 2.0, chi2 / 2.0)


def wls(xs, ys, es):
    """Weighted straight line: the value at the weighted mean of x, the slope,
    their errors, chi^2 and the degrees of freedom."""
    w = [1.0 / e ** 2 for e in es]
    sw = sum(w)
    xm = sum(wi * x for wi, x in zip(w, xs)) / sw
    ym = sum(wi * y for wi, y in zip(w, ys)) / sw
    sxx = sum(wi * (x - xm) ** 2 for wi, x in zip(w, xs))
    slope = sum(wi * (x - xm) * (y - ym) for wi, x, y in zip(w, xs, ys)) / sxx if sxx > 0 else 0.0
    chi2 = sum(wi * (y - ym - slope * (x - xm)) ** 2 for wi, x, y in zip(w, xs, ys))
    return dict(xm=xm, at_xm=ym, slope=slope, se_at_xm=1.0 / math.sqrt(sw),
                se_slope=1.0 / math.sqrt(sxx) if sxx > 0 else math.nan, chi2=chi2, dof=len(xs) - 2)


def weighted_mean(rs, es):
    w = [1.0 / e ** 2 for e in es]
    m = sum(wi * r for wi, r in zip(w, rs)) / sum(w)
    chi2 = sum(wi * (r - m) ** 2 for wi, r in zip(w, rs))
    return m, 1.0 / math.sqrt(sum(w)), chi2


# ---------------------------------------------------------------- part 1


def pooled():
    """family -> bound -> [(z, label)]"""
    fam = defaultdict(lambda: defaultdict(list))
    with open("data/results/summary.conditional.csv") as fh:
        for r in csv.DictReader(fh):
            fam[f"D1 {r['side']} side, after the root counts"][int(r["b"])].append((float(r["z_after_first"]), r["instance"]))
    for f in sorted(Path("data/results/d2/joint").glob("d1-*.csv")):
        inst = f.stem
        with open(f) as fh:
            for r in csv.DictReader(fh):
                b, o, e, v = int(r["b"]), float(r["observed"]), float(r["expected"]), float(r["variance"])
                if v <= 0:
                    continue
                z = (o - e) / math.sqrt(v)
                if r["kind"] == "column":
                    if not r["name"].startswith(inst):
                        continue  # the next instance's columns, the control here; counted in their own file
                    kind = "rational sides" if r["name"].endswith("/rational") else (
                        "rotated fields" if ".r" in r["name"] else "instruments")
                    fam[f"D2 single column: {kind}"][b].append((z, r["name"]))
                elif r["kind"] == "class":
                    fam[f"D2 joint: {r['name']}"][b].append((z, inst))
                elif r["kind"] == "union":
                    fam["D2 joint: union"][b].append((z, inst))
    with open("data/results/d5_clusters.csv") as fh:
        for r in csv.DictReader(fh):
            b = int(r["b"])
            lift, err, chi2, regions = float(r["lift"]), float(r["lift_err"]), float(r["chi2"]), int(r["regions"])
            if err > 0 and not math.isnan(lift):
                fam[f"D5 held-out lift: {r['side']}"][b].append((lift / err, f"{r['instance']} {r['partition']}"))
            if regions > 1 and not math.isnan(chi2):
                fam[f"D5 dispersion: {r['side']}"][b].append(((chi2 - regions) / math.sqrt(2 * regions), f"{r['instance']} {r['partition']}"))
    return fam


# ---------------------------------------------------------------- part 2: manifest and run


class Predictions:
    def __init__(self):
        self.base, self.q = d1.load_baseline("data/results/baseline.consecutive.json")
        self.rough = self.base["cofactor"]["rough_bound"]
        self.width = self.base["cofactor"]["bin_width"]
        self.omega = d1.buchstab_omega()
        self.tails = {}
        cover = None
        for path in ("data/results/root_tail.json", "data/results/d2/root_tail.json", "data/results/galois/root_tail.json"):
            t, c = d1.load_root_tail(path, self.rough)
            if not t:
                sys.exit(f"{path} is missing: run d1-tail (scripts/d_checks_run_all.sh)")
            self.tails[path] = t
            cover = c if cover is None else min(cover, c)
        self.bounds = sorted(b for b in self.q if b <= cover)
        if tuple(self.bounds) != BOUNDS:
            sys.exit(f"bounds {self.bounds} are not the protocol's six")
        self.top = (max(max(qb) for qb in self.q.values()) + 2) * self.width

    def rows(self, results, tail_path, key, side, with_tail=True):
        hist, region = {}, None
        for f in results:
            result, h = d1.load_instance(f)
            if result is None:
                continue
            region = region or result["region"]
            for k, bins in h.items():
                hist.setdefault(k, bins)
        if region is None:
            sys.exit(f"{results}: no d1-measurement/4 result")
        out = []
        for b in self.bounds:
            s_by_bin = {}
            if with_tail:
                s_by_bin = d1.first_order_by_bin(self.q[b], hist[(side, b)], self.tails[tail_path][key], b, self.width,
                                                 self.rough, self.omega, region["a_max"], region["b_max"])
            ps, _ = d5.prediction_grid(self.q[b], b, self.width, s_by_bin, self.top)
            out.append([None if math.isnan(p) else p for p in ps])
        return out, region


def columns():
    """(name, family, dump, side, results, tail path, tail key, with the root-count term)"""
    cols = []
    for i in range(1, 11):
        inst = f"d1-{i:02d}"
        res = sorted(Path("data/results").glob(f"{inst}.scaled-down*.json"))
        dump = f"data/dumps/{inst}.values.csv"
        cols.append((inst, "instruments", dump, "algebraic", res, "data/results/root_tail.json", (inst, "algebraic"), True))
        cols.append((f"{inst}/rational", "rational sides", dump, "rational", res, "data/results/root_tail.json", (inst, "rational"), True))
    with open("data/results/d2/fields.csv") as fh:
        fields = [r["field"] for r in csv.DictReader(fh) if r["rank"]]
    for f in fields:
        cols.append((f, "rotated fields", f"data/dumps/d2/{f}.values.csv", "algebraic", [Path(f"data/results/d2/{f}.json")],
                     "data/results/d2/root_tail.json", (f, "algebraic"), True))
    for k in GALOIS:
        res = [Path(f"data/results/galois/{k}.json")]
        dump = f"data/dumps/galois/{k}.values.csv"
        cols.append((f"galois/{k}", "quartics", dump, "algebraic", res, "data/results/galois/root_tail.json", (k, "algebraic"), True))
        cols.append((f"galois/{k}/no-tail", "quartics without the root-count term", dump, "algebraic", res,
                     "data/results/galois/root_tail.json", (k, "algebraic"), False))
    return cols


def run_checks(out_sums):
    pred = Predictions()
    cols = columns()
    region = None
    entries = []
    for name, _, dump, side, res, tpath, key, with_tail in cols:
        if not Path(dump).exists():
            sys.exit(f"{dump} is missing: run scripts/d5_run_all.sh, scripts/d2_run_all.sh, scripts/d_checks_run_all.sh")
        pi, reg = pred.rows(res, tpath, key, side, with_tail)
        if region is None:
            region = reg
        elif (reg["a_max"], reg["b_max"]) != (region["a_max"], region["b_max"]):
            sys.exit(f"{name}: region differs")
        entries.append(dict(name=name, dump=dump, side=side, pi=pi))
    manifest = dict(schema_version="d-checks-manifest/1", bounds=pred.bounds, cofactor_bin_width=pred.width,
                    region=dict(a_max=region["a_max"], b_max=region["b_max"]), columns=entries)
    mpath = Path("data/dumps/d_checks/manifest.json")
    mpath.parent.mkdir(parents=True, exist_ok=True)
    mpath.write_text(json.dumps(manifest))
    subprocess.run([CHECKS, "--manifest", str(mpath), "--out", out_sums], check=True)


def load_sums(path):
    s = {}
    with open(path) as fh:
        for r in csv.DictReader(fh):
            s[(r["column"], r["group"], int(r["bin"]), int(r["b"]))] = (
                float(r["values"] or 0), float(r["observed"]), float(r["expected"]), float(r["variance"]))
    return s


def family_of():
    return {name: fam for name, fam, *_ in columns()}


def pooled_bins(s, cols, group, b, nbins):
    out = []
    for k in range(nbins):
        n = o = p = v = 0.0
        for c in cols:
            x = s.get((c, group, k, b))
            if x:
                n += x[0]
                o += x[1]
                p += x[2]
                v += x[3]
        out.append((n, o, p, v))
    return out


def uncertain(s, c, b):
    """O, P, V over the values not smooth for certain."""
    a = s[(c, "all", 0, b)]
    cert = s.get((c, "certain", 0, b), (0.0, 0.0, 0.0, 0.0))
    return a[1] - cert[1], a[2] - cert[2], a[3]


def digits(inst):
    text = Path(f"data/instruments/{inst}/cado.poly").read_text()
    return len(re.search(r"^n:\s*(\d+)", text, re.M).group(1))


# ---------------------------------------------------------------- main


def main(argv):
    out, skip = "data/results/d_checks", False
    i = 1
    while i < len(argv):
        if argv[i] == "--out":
            out = argv[i + 1]
            i += 2
        elif argv[i] == "--skip-run":
            skip = True
            i += 1
        else:
            sys.exit("usage: d_checks.py [--out PATH] [--skip-run]")
    sums_path = out + ".sums.csv"
    if not skip:
        run_checks(sums_path)
    s = load_sums(sums_path)
    fam = family_of()
    by = defaultdict(list)
    for c, f in fam.items():
        by[f].append(c)
    rows = []  # one per statistic
    md = []

    # part 1
    md.append("## 1. Pooled statistics of D1, D2 part 1 and D5\n")
    md.append("Every per-instance statistic as z = deviation / its error. Under pure noise the mean is 0, the mean square 1,"
              " and the Kolmogorov-Smirnov p-value uniform. The bounds of one family are nested, so its rows are not"
              " independent; the last column pools them only to show the tails.\n")
    md.append("| family | n per bound | mean z per bound (2^14 … 2^28) | mean z² per bound | all bounds: KS p, beyond 2 / 3 (expected) |")
    md.append("|---|---:|---|---|---|")
    fz = pooled()
    every = []
    for name in sorted(fz):
        means, squares, allz = [], [], []
        for b in BOUNDS:
            zs = [z for z, _ in fz[name][b]]
            if not zs:
                continue
            allz += zs
            means.append(sum(zs) / len(zs))
            squares.append(sum(z * z for z in zs) / len(zs))
            every += [(abs(z), z, name, b, lab_) for z, lab_ in fz[name][b]]
            rows.append(dict(part="pooled", family=name, test="mean z", b=b, n=len(zs), value=means[-1],
                             error=1 / math.sqrt(len(zs)), stat=squares[-1], p=ks_pvalue(zs)))
        n = len(allz)
        e2, e3 = 2 * n * (1 - phi(2)), 2 * n * (1 - phi(3))
        t2, t3 = sum(abs(z) > 2 for z in allz), sum(abs(z) > 3 for z in allz)
        md.append(f"| {name} | {n // len(means)} | {' '.join(f'{m:+.2f}' for m in means)} | {' '.join(f'{q:.2f}' for q in squares)}"
                  f" | {ks_pvalue(allz):.3f}; {t2} / {t3} ({e2:.1f} / {e3:.2f}) |")
    every.sort(reverse=True)
    md.append(f"\n{len(every)} statistics. The ten largest |z|:\n")
    md.append("| z | family | bound | instance and region |")
    md.append("|---:|---|---|---|")
    for _, z, name, b, lab_ in every[:10]:
        md.append(f"| {z:+.2f} | {name} | {lab(b)} | {lab_} |")

    # part 2
    md.append("\n## 2. Per-value checks\n")
    md.append("Values smooth for certain (cofactor below B) are left out. r = O/P − 1 with the counting error √V/P.\n")
    md.append("**Calibration in u = ln c / ln B.** Per bound, the weighted straight line of r against u over the bins of 0.25"
              " that hold P ≥ 25, and its value extrapolated to u = 6:\n")
    md.append("| columns | bound | u range | slope per unit u | χ² / dof | r at u = 6 |")
    md.append("|---|---|---|---:|---:|---:|")
    for f in ("instruments", "rotated fields", "rational sides"):
        for b in BOUNDS:
            bins = pooled_bins(s, by[f], "u", b, 64)
            xs, ys, es = [], [], []
            for k, (n, o, p, v) in enumerate(bins):
                if p >= 25 and v > 0:
                    xs.append((k + 0.5) * U_WIDTH)
                    ys.append(o / p - 1)
                    es.append(math.sqrt(v) / p)
            if len(xs) < 3:
                continue
            w = wls(xs, ys, es)
            at6 = w["at_xm"] + w["slope"] * (6 - w["xm"])
            e6 = math.hypot(w["se_at_xm"], w["se_slope"] * (6 - w["xm"]))
            rows.append(dict(part="u", family=f, test="slope in u", b=b, n=len(xs), value=w["slope"], error=w["se_slope"],
                             stat=w["chi2"], p=chi2_sf(w["chi2"], w["dof"])))
            md.append(f"| {f} | {lab(b)} | {min(xs) - 0.125:.2f}–{max(xs) + 0.125:.2f} | {pct(w['slope'], w['se_slope'], 3)} |"
                      f" {w['chi2']:.1f} / {w['dof']} | {pct(at6, e6)} |")
    md.append("\n**All algebraic columns (instruments and rotated fields), every bound pooled** (a value counts once per bound):\n")
    md.append("| u | values | observed | predicted | r |")
    md.append("|---|---:|---:|---:|---:|")
    alg = by["instruments"] + by["rotated fields"]
    tot = [[0.0] * 4 for _ in range(64)]
    for b in BOUNDS:
        for k, x in enumerate(pooled_bins(s, alg, "u", b, 64)):
            for t in range(4):
                tot[k][t] += x[t]
    for k in range(4, 64, 2):
        n = tot[k][0] + tot[k + 1][0]
        o = tot[k][1] + tot[k + 1][1]
        p = tot[k][2] + tot[k + 1][2]
        v = tot[k][3] + tot[k + 1][3]
        if p < 25:
            continue
        rows.append(dict(part="u pooled", family="algebraic", test=f"u {k * U_WIDTH:.1f}-{(k + 2) * U_WIDTH:.1f}", b=0, n=int(n),
                         value=o / p - 1, error=math.sqrt(v) / p, stat=o, p=math.nan))
        md.append(f"| {k * U_WIDTH:.1f}–{(k + 2) * U_WIDTH:.1f} | {int(n):,} | {int(o):,} | {p:,.1f} | {pct(o / p - 1, math.sqrt(v) / p, 3)} |")

    md.append("\n**Smoothness at fixed cofactor against what the model says does not matter.** Per family and bound, χ² of the"
              " bins (P ≥ 25) about the family's own O/P, its p-value, and the weighted slope of r across the bins:\n")
    md.append("| grouping | family | χ² / dof per bound (2^14 … 2^28) | smallest p | largest |slope| / error |")
    md.append("|---|---|---|---:|---:|")
    groups = (("lns", 64, "ln of the part made of the primes up to 2000, bins of 1"),
              ("olns", 64, "the other side's small part, bins of 1"),
              ("oc", 64, "ln of the other side's cofactor, bins of 2"),
              ("parity", 4, "(a mod 2, b mod 2)"),
              ("mod3", 9, "(a mod 3, b mod 3)"))
    for g, nb, desc in groups:
        for f in ("instruments", "rotated fields", "rational sides"):
            cells, pmin, smax = [], 1.0, 0.0
            for b in BOUNDS:
                bins = [(k, x) for k, x in enumerate(pooled_bins(s, by[f], g, b, nb)) if x[2] >= 25 and x[3] > 0]
                if len(bins) < 2:
                    continue
                kap = sum(x[1] for _, x in bins) / sum(x[2] for _, x in bins)
                chi2 = sum((x[1] - kap * x[2]) ** 2 / x[3] for _, x in bins)
                dof = len(bins) - 1
                p = chi2_sf(chi2, dof)
                pmin = min(pmin, p)
                cells.append(f"{chi2:.1f}/{dof}")
                if len(bins) >= 3:
                    w = wls([float(k) for k, _ in bins], [x[1] / x[2] - 1 for _, x in bins], [math.sqrt(x[3]) / x[2] for _, x in bins])
                    smax = max(smax, abs(w["slope"]) / w["se_slope"])
                    rows.append(dict(part="grouping", family=f, test=f"{g} slope", b=b, n=len(bins), value=w["slope"],
                                     error=w["se_slope"], stat=chi2, p=p))
                else:
                    rows.append(dict(part="grouping", family=f, test=f"{g} chi2", b=b, n=len(bins), value=math.nan,
                                     error=math.nan, stat=chi2, p=p))
            md.append(f"| {desc} | {f} | {' '.join(cells)} | {pmin:.3f} | {smax:.1f} |")

    md.append("\n**Neighbouring values smooth together**, R = O/E − 1 against independence, E = Σ π π′:\n")
    md.append("| columns | pair | " + " | ".join(lab(b) for b in BOUNDS) + " |")
    md.append("|---|---|" + "---:|" * len(BOUNDS))
    for f, cols in (("algebraic (40)", alg), ("rational sides", by["rational sides"])):
        for g, desc in (("pair-a", "(a, b), (a + 1, b)"), ("pair-b", "(a, b), (a, b + 1)")):
            cells = []
            for b in BOUNDS:
                o = sum(s[(c, g, 0, b)][1] for c in cols)
                e = sum(s[(c, g, 0, b)][2] for c in cols)
                v = sum(s[(c, g, 0, b)][3] for c in cols)
                rows.append(dict(part="neighbours", family=f, test=g, b=b, n=len(cols), value=o / e - 1, error=math.sqrt(v) / e,
                                 stat=o, p=math.nan))
                cells.append(pct(o / e - 1, math.sqrt(v) / e))
            md.append(f"| {f} | {desc} | " + " | ".join(cells) + " |")

    # part 3
    md.append("\n## 3. The gap against the digits of N\n")
    md.append("κ − 1 = O/P − 1 over the values not smooth for certain, per polynomial; the weighted straight line in the"
              " digits of N (errors widened by √(χ²/dof) where that exceeds 1), and its extrapolation with the one-sided"
              " 95% upper bound:\n")
    md.append("| columns | bound | slope per digit | χ² / dof | 200 digits (95% upper) | 250 digits (95% upper) |")
    md.append("|---|---|---:|---:|---:|---:|")
    for f, cols in (("algebraic (40)", alg), ("rational sides", by["rational sides"])):
        for b in BOUNDS:
            xs, ys, es = [], [], []
            for c in cols:
                o, p, v = uncertain(s, c, b)
                xs.append(float(digits(c.split(".")[0].split("/")[0])))
                ys.append(o / p - 1)
                es.append(math.sqrt(v) / p)
            w = wls(xs, ys, es)
            infl = max(1.0, math.sqrt(w["chi2"] / w["dof"]))
            cells = []
            for D in (200, 250):
                m = w["at_xm"] + w["slope"] * (D - w["xm"])
                e = infl * math.hypot(w["se_at_xm"], w["se_slope"] * (D - w["xm"]))
                cells.append(f"{pct(m)} ({pct(m + 1.645 * e)})")
                rows.append(dict(part="trend", family=f, test=f"at {D} digits", b=b, n=len(xs), value=m, error=e,
                                 stat=w["chi2"], p=math.nan))
            md.append(f"| {f} | {lab(b)} | {pct(w['slope'], infl * w['se_slope'], 4)} | {w['chi2']:.1f} / {w['dof']} | {cells[0]} | {cells[1]} |")

    # part 4
    md.append("\n## 4. The toy quartics of the structure ladder\n")
    md.append("Tiny coefficients, Galois groups A4, S4, C4, D4, V4 (`data/instruments/galois/`). Var r_p is the variance of"
              " the number of roots mod p over the primes (Chebotarev: 1 for a 2-transitive group, 2 for D4, 3 for C4 and V4)."
              " κ − 1 over the values not smooth for certain, before and after the first-order root-count term:\n")
    md.append("| quartic | group | Var r_p | | " + " | ".join(lab(b) for b in BOUNDS) + " |")
    md.append("|---|---|---:|---|" + "---:|" * len(BOUNDS))
    after = defaultdict(list)
    for k, (grp, var) in GALOIS.items():
        for label, col in (("before", f"galois/{k}/no-tail"), ("after", f"galois/{k}")):
            cells = []
            for b in BOUNDS:
                o, p, v = uncertain(s, col, b)
                r, e = o / p - 1, math.sqrt(v) / p
                cells.append(pct(r, e) if label == "after" else pct(r))
                rows.append(dict(part="galois", family=grp, test=label, b=b, n=1, value=r, error=e, stat=o, p=math.nan))
                if label == "after":
                    after[b].append((var, r, e))
            md.append(f"| {k} | {grp} | {var} | {label} | " + " | ".join(cells) + " |")
    md.append("\nAfter the term, per bound: the five about their weighted mean (χ²) and the weighted slope in Var r_p:\n")
    md.append("| bound | weighted mean | χ² / 4 | slope per unit of Var r_p |")
    md.append("|---|---:|---:|---:|")
    for b in BOUNDS:
        xs = [x for x, _, _ in after[b]]
        ys = [y for _, y, _ in after[b]]
        es = [e for _, _, e in after[b]]
        m, me, chi2 = weighted_mean(ys, es)
        w = wls(xs, ys, es)
        rows.append(dict(part="galois", family="all five", test="slope in Var r_p", b=b, n=5, value=w["slope"], error=w["se_slope"],
                         stat=chi2, p=chi2_sf(chi2, 4)))
        md.append(f"| {lab(b)} | {pct(m, me)} | {chi2:.1f} | {pct(w['slope'], w['se_slope'], 3)} |")

    head = ["# Obstacle D: the closing checks\n",
            "Generated by `scripts/d_checks.py` (`scripts/d_checks_run_all.sh` end to end). The reading is in"
            " `docs/obstacle_d_verdict.md`. Per-value prediction: the conditional baseline Q_B(c) plus the first-order"
            " root-count term (`docs/baseline_calibration.md` sections 7–9), as D5 and D2 use it.\n"]
    Path(out + ".md").write_text("\n".join(head + md) + "\n")
    with open(out + ".csv", "w", newline="") as fh:
        w = csv.DictWriter(fh, fieldnames=["part", "family", "test", "b", "n", "value", "error", "stat", "p"])
        w.writeheader()
        for r in rows:
            w.writerow({k: (f"{v:.6g}" if isinstance(v, float) else v) for k, v in r.items()})
    print(f"wrote {out}.md, {out}.csv ({len(rows)} rows) and {sums_path}")


if __name__ == "__main__":
    main(sys.argv)
