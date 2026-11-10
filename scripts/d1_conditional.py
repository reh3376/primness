#!/usr/bin/env python3
"""Compare every instance with the conditional baseline Q_B(c).

    scripts/d1_conditional.py [--baseline data/results/baseline.consecutive.json]
                              [--results GLOB] [--out data/results/summary.conditional]

The baseline (d1-baseline/2) gives, per bin of ln c and bound B, the density
Q_B(c) of B-smooth integers among the integers with no prime factor up to the
rough bound. Each instance result (d1-measurement/4) gives, per side, the
values and the smooth values by the same bins of their cofactor c = |F| / s.
The prediction for a side is the sum over its values of Q_B(c); it uses
neither alpha nor an expansion. docs/baseline_calibration.md sections 5, 7, 8.

Two readings of the table are reported: Q of the bin as counted, and Q
interpolated linearly between the mean ln c of neighbouring baseline bins at
the instance's own mean ln c in the bin. Their difference measures what the
bin width costs. Noise: the binomial noise of the instance's count and the
noise the baseline's own counts put on the prediction.

The alpha tail. The split removes the primes up to the rough bound y exactly;
a polynomial's roots at the primes between y and B still move its smoothness.
Linearized, they act as the size shift alpha(B) - alpha(y), which is zero on
average over polynomials; the script reports that tail, the gap it predicts
through the table's own slope in ln c, and what remains after it. The tail
comes from the root counts of d1-tail (data/results/root_tail.json) when
present, else from d1-alpha results at larger bounds
(data/results/alpha_moments.b<e>.json for the bound 2^e).

The exact first-order term (section 9). Without the linearization: a prime p
in (y, B] divides a value with probability r_p/(p + 1) against 1/p for an
integer, and changing that one probability moves the smooth density of the
cofactors of size c by

    eta_p * R(c/p)/R(c) * (Q_B(c/p) - Q_B(c)),
    eta_p = (r_p p - p - 1)/(p^2 - 1),

where R is the density of y-rough integers near a size (Buchstab's omega).
A prime above the region's largest b never meets the root at infinity, and
there the affine roots alone count: eta_p^aff = (r_p^aff - 1)/(p - 1). A prime
above the largest |a| meets the root 0 only at (0, 1), and where 0 is a root
1/(p - 1) comes off as well.
Summed over the primes, from the per-bin sums of d1-tail, and over the
values, it is the correction to the prediction; the gap after it is what the
root counts between y and B do not explain.

Writes <out>.csv and <out>.md. Standard library only.
"""
import csv
import glob
import json
import math
import sys
from collections import defaultdict
from pathlib import Path


def load_baseline(path):
    result = json.loads(Path(path).read_text())
    if result.get("schema_version") != "d1-baseline/2":
        sys.exit(f"{path}: not a d1-baseline/2 result")
    table = next(t for t in result["tables"] if t["name"] == "cofactor")
    q = defaultdict(dict)  # b -> k -> (count, mean_ln_c, smooth)
    with open(table["path"]) as f:
        for row in csv.DictReader(f):
            k = int(row["bin"])
            q[int(row["b"])][k] = (int(row["count"]), float(row["mean_ln_c"]), int(row["smooth"]))
    return result, q


def load_instance(path):
    result = json.loads(Path(path).read_text())
    if result.get("schema_version") != "d1-measurement/4":
        return None, None
    table = next(t for t in result["tables"] if t["name"] == "cofactor")
    hist = defaultdict(list)  # (side, b) -> [(k, count, mean_ln_c, smooth)]
    with open(table["path"]) as f:
        for row in csv.DictReader(f):
            k = int(row["bin"])
            hist[(row["side"], int(row["b"]))].append((k, int(row["count"]), float(row["mean_ln_c"]), int(row["smooth"])))
    return result, hist


def interpolated(qb, k, x, jump):
    """Q at ln c = x from the baseline bin k and a neighbour: the one on x's
    side, or the other one when that side lies across the jump at c = B
    (bins below index `jump` hold cofactors below B, which are all smooth)."""
    n0, m0, s0 = qb[k]
    q0 = s0 / n0
    if k == 0 or k < jump:
        return q0
    j = k - 1 if x < m0 else k + 1
    if j < jump:
        j = k + 1
    other = qb.get(j)
    if other is None:
        return q0
    n1, m1, s1 = other
    if n1 == 0 or m1 == m0:
        return q0
    q1 = s1 / n1
    return min(1.0, max(0.0, q0 + (q1 - q0) * (x - m0) / (m1 - m0)))


def compare(qb, bins, jump):
    """Observed and predicted smooth counts of one side at one bound."""
    out = dict(values=0, observed=0, pred_bin=0.0, pred_interp=0.0, var_obs=0.0, var_table=0.0, uncovered=0)
    for k, n, mean, smooth in bins:
        out["values"] += n
        out["observed"] += smooth
        if k not in qb or qb[k][0] == 0:
            out["uncovered"] += n
            continue
        big_n, _, big_s = qb[k]
        q = big_s / big_n
        out["pred_bin"] += n * q
        out["pred_interp"] += n * interpolated(qb, k, mean, jump)
        out["var_obs"] += n * q * (1.0 - q)
        # With no smooth integer counted, the bin's density is below about 1/N; carry that as its noise.
        var_q = q * (1.0 - q) / big_n if big_s > 0 else 1.0 / big_n**2
        out["var_table"] += n * n * var_q
    return out


def sensitivity(qb, bins, shift, width):
    """d ln(prediction) / d ln c: every value moved `shift` bins down and up."""
    def pred(m):
        total = 0.0
        for k, n, _, _ in bins:
            if k == 0:
                total += n
            elif k + m in qb and qb[k + m][0] > 0:
                total += n * qb[k + m][2] / qb[k + m][0]
        return total
    lo, hi = pred(-shift), pred(shift)
    if lo <= 0 or hi <= 0:
        return float("nan")
    return (math.log(hi) - math.log(lo)) / (2 * shift * width)


def load_alphas(rough_bound):
    """bound -> {(instance, side): alpha} from every alpha-moments result under data/results."""
    out = {}
    for f in sorted(glob.glob("data/results/alpha_moments*.json")):
        r = json.loads(Path(f).read_text())
        if not str(r.get("schema_version", "")).startswith("alpha-moments/"):
            continue
        table = next(t for t in r["tables"] if "alpha" in t["columns"])
        with open(table["path"]) as fh:
            out[r["bound"]] = {(row["instance"], row["side"]): float(row["alpha"]) for row in csv.DictReader(fh)}
    return out if rough_bound in out and len(out) > 1 else {}


def load_root_tail(path, rough_bound):
    """(instance, side) -> [(mean ln p, ln p upper edge, sum eta, sum eta ln p,
    sum eta_aff, ln p lower edge, sum eta_zero)] by bin of ln p, and the bound
    covered, from a root-tail/1 result."""
    if not Path(path).exists():
        return {}, 0
    r = json.loads(Path(path).read_text())
    if r.get("schema_version") != "root-tail/1" or r["from"] != rough_bound:
        return {}, 0
    table = next(t for t in r["tables"] if t["name"] == "root_tail")
    out = defaultdict(list)
    with open(table["path"]) as fh:
        for row in csv.DictReader(fh):
            out[(row["instance"], row["side"])].append(
                (float(row["mean_ln_p"]), float(row["ln_p_hi"]), float(row["eta"]), float(row["eta_ln_p"]),
                 float(row["eta_affine"]), float(row["ln_p_lo"]), float(row["eta_zero"])))
    for bins in out.values():
        bins.sort()
    return out, r["bound"]


def root_tail_bins(tail, b):
    """The bins of ln p that lie at or below ln B (B a power of two is a bin edge)."""
    return [t for t in tail if t[1] <= math.log(b) + 1e-9]


EULER_GAMMA = 0.5772156649015329


def buchstab_omega(u_max=30.0, du=1e-3):
    """omega(u) as a function: 1/u on [1, 2], (u omega(u))' = omega(u - 1)
    beyond (trapezoid rule on a grid of du), e^-gamma past u_max, 0 below 1."""
    n, lag = int(round((u_max - 1) / du)), int(round(1 / du))
    w = [0.0] * (n + 1)
    for i in range(n + 1):
        u = 1 + i * du
        if i <= lag:
            w[i] = 1 / u
        else:
            w[i] = ((u - du) * w[i - 1] + 0.5 * du * (w[i - lag] + w[i - 1 - lag])) / u
    limit = math.exp(-EULER_GAMMA)

    def omega(u):
        if u < 1:
            return 0.0
        if u >= u_max:
            return limit
        x = (u - 1) / du
        i = int(x)
        return w[i] + (w[i + 1] - w[i]) * (x - i) if i < n else w[n]
    return omega


def q_function(qb, b, width, step):
    """Q_B at any ln c: 1 below ln B, else the table interpolated as the
    prediction reads it, sampled every `step` and interpolated between
    samples on the same side of the jump; None where the table has no data."""
    ln_b = math.log(b)
    jump = math.ceil(ln_b / width - 1e-9)
    top = max(qb) + 1
    n = int(top * width / step) + 2
    i_jump = int(round(ln_b / step))
    grid = [None] * n
    for i in range(n):
        if i < i_jump:
            grid[i] = 1.0
            continue
        z = i * step
        k = max(int(z / width), jump)
        if k in qb and qb[k][0] > 0:
            grid[i] = interpolated(qb, k, z, jump)

    def q(z):
        if z < ln_b:
            return 1.0
        i = max(int(z / step), i_jump)
        if i + 1 >= n:
            return None
        g0, g1 = grid[i], grid[i + 1]
        if g0 is None or g1 is None:
            return g0 if g1 is None else g1
        return g0 + (g1 - g0) * (z / step - i)
    return q


def region_eta(t, ln_a_max, ln_b_max):
    """A bin's sum of eta for the region: the root at infinity dropped above
    the largest b, the root 0 dropped above the largest |a|."""
    e = t[4] if t[5] >= ln_b_max else t[2]
    return e - t[6] if t[5] >= ln_a_max else e


def first_order(qb, bins, tail, b, width, rough, omega, a_max, b_max, density=True):
    """The exact first-order term of the primes in (rough, B], as an absolute
    number of smooth values: sum over values and primes of eta_p R(c/p)/R(c)
    (Q(c/p) - Q(c)), with eta_p for the region (region_eta). `density` False
    sets R(c/p)/R(c) to 1."""
    ln_y, ln_b = math.log(rough), math.log(b)
    jump = math.ceil(ln_b / width - 1e-9)
    q = q_function(qb, b, width, width / 8)
    tb = [(t[0], region_eta(t, math.log(a_max), math.log(b_max))) for t in root_tail_bins(tail, b)]
    total = 0.0
    for k, n, x, _ in bins:
        if k < jump or k not in qb or qb[k][0] == 0:
            continue  # below B every cofactor is smooth, and so is every c/p
        q0 = interpolated(qb, k, x, jump)
        w0 = omega(x / ln_y)
        s = 0.0
        for l, eta in tb:
            z = x - l
            if z < ln_y:
                break  # a rough c/p below y is 1, and c = p is not above B
            qz = q(z)
            if qz is None:
                continue
            s += eta * (omega(z / ln_y) / w0 if density else 1.0) * (qz - q0)
        total += n * s
    return total


def mean_std(xs):
    n = len(xs)
    m = sum(xs) / n
    sd = math.sqrt(sum((x - m) ** 2 for x in xs) / (n - 1)) if n > 1 else float("nan")
    return m, sd


def weighted(xs, ses):
    """Inverse-variance weighted mean, its standard error, and chi-square of
    the values about zero (as many degrees of freedom as values). The
    instances' noise differs fifty-fold, so these, not the plain spread, say
    whether the values are consistent with their noise."""
    w = [1.0 / s ** 2 for s in ses]
    m = sum(wi * x for wi, x in zip(w, xs)) / sum(w)
    return m, 1.0 / math.sqrt(sum(w)), sum((x / s) ** 2 for x, s in zip(xs, ses))


def main(argv):
    baseline_path, pattern, out = "data/results/baseline.consecutive.json", "data/results/d1-*.scaled-down*.json", "data/results/summary.conditional"
    root_tail_path = "data/results/root_tail.json"
    i = 1
    while i < len(argv):
        if argv[i] == "--baseline":
            baseline_path = argv[i + 1]; i += 2
        elif argv[i] == "--results":
            pattern = argv[i + 1]; i += 2
        elif argv[i] == "--root-tail":
            root_tail_path = argv[i + 1]; i += 2
        elif argv[i] == "--out":
            out = argv[i + 1]; i += 2
        else:
            sys.exit("usage: d1_conditional.py [--baseline FILE] [--results GLOB] [--root-tail FILE] [--out PATH]")
    base, q = load_baseline(baseline_path)
    rough = base["cofactor"]["rough_bound"]
    alphas = load_alphas(rough)
    tails, tail_cover = load_root_tail(root_tail_path, rough)
    omega = buchstab_omega()

    rows, seen = [], set()
    for f in sorted(glob.glob(pattern)):
        result, hist = load_instance(f)
        if result is None:
            continue
        cof = result["cofactor"]
        if cof["rough_bound"] != base["cofactor"]["rough_bound"] or cof["bin_width"] != base["cofactor"]["bin_width"]:
            sys.exit(f"{f}: rough bound or bin width differs from the baseline's")
        inst = result["instance"]
        for (side, b), bins in sorted(hist.items()):
            # A bound measured in two runs of an instance is counted once.
            if b not in q or (inst, side, b) in seen:
                continue
            seen.add((inst, side, b))
            # The first bin at or above ln B; exact when the width is ln 2 over an integer and B a power of two.
            jump = math.ceil(math.log(b) / cof["bin_width"] - 1e-9)
            c = compare(q[b], bins, jump)
            covered = c["values"] - c["uncovered"]
            se_obs, se_table = math.sqrt(c["var_obs"]), math.sqrt(c["var_table"])
            se = math.hypot(se_obs, se_table)
            rel = lambda pred: c["observed"] / pred - 1.0 if pred > 0 else float("nan")
            # The alpha tail over the primes in (rough bound, B]: from the root counts when they
            # reach B, else from d1-alpha at the largest computed bound not above B.
            tail = tail_bound = tail_gap = first = first_flat = None
            if (inst, side) in tails and b <= tail_cover:
                tail_bound = b
                tail = -sum(t[3] for t in root_tail_bins(tails[(inst, side)], b))
                reg = (result["region"]["a_max"], result["region"]["b_max"])
                first = first_order(q[b], bins, tails[(inst, side)], b, cof["bin_width"], rough, omega, *reg)
                first_flat = first_order(q[b], bins, tails[(inst, side)], b, cof["bin_width"], rough, omega, *reg, density=False)
            else:
                at = [x for x in alphas if rough < x <= b and (inst, side) in alphas[x]]
                if at and (inst, side) in alphas[rough]:
                    tail_bound = max(at)
                    tail = alphas[tail_bound][(inst, side)] - alphas[rough][(inst, side)]
            if tail is not None:
                slope = sensitivity(q[b], bins, 4, cof["bin_width"])
                tail_gap = math.expm1(slope * tail) if slope == slope else None
            rows.append(dict(
                instance=inst, side=side, b=b, values=c["values"], uncovered=c["uncovered"],
                observed=c["observed"], predicted_bin=c["pred_bin"], predicted=c["pred_interp"],
                se_observed=se_obs, se_table=se_table,
                rel_gap_bin=rel(c["pred_bin"]), rel_gap=rel(c["pred_interp"]),
                rel_se=se / c["pred_interp"] if c["pred_interp"] > 0 else float("nan"),
                z=(c["observed"] - c["pred_interp"]) / se if se > 0 else float("nan"),
                density=c["observed"] / covered if covered else float("nan"),
                alpha_tail="" if tail is None else tail, alpha_tail_bound="" if tail_bound is None else tail_bound,
                tail_gap="" if tail_gap is None else tail_gap,
                rel_gap_after_tail="" if tail_gap is None else (c["observed"] / c["pred_interp"] - 1.0) - tail_gap,
                first_order="" if first is None else first / c["pred_interp"],
                first_order_flat="" if first_flat is None else first_flat / c["pred_interp"],
                rel_gap_after_first="" if first is None else c["observed"] / (c["pred_interp"] + first) - 1.0,
                z_after_first="" if first is None or se == 0 else (c["observed"] - c["pred_interp"] - first) / se,
            ))
    if not rows:
        sys.exit("no d1-measurement/4 results found: rerun the instances with the current d1-measure")

    cols = list(rows[0].keys())
    with open(out + ".csv", "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=cols)
        w.writeheader()
        for r in rows:
            w.writerow({k: (f"{v:.6g}" if isinstance(v, float) else v) for k, v in r.items()})

    instances = sorted({r["instance"] for r in rows})
    bounds = sorted({r["b"] for r in rows})
    label = lambda b: f"2^{b.bit_length() - 1}" if b & (b - 1) == 0 else str(b)
    pct = lambda x: f"{100 * x:+.2f}%"
    L = ["# D1 against the conditional baseline", ""]
    L.append(f"Baseline run `{base['run_id']}`: {base['cofactor']['count']:,} consecutive integers split at the rough bound {base['cofactor']['rough_bound']}, ln c in bins of {base['cofactor']['bin_width']:.5f}. {len(instances)} instances. The prediction for a side is the sum over its values of Q_B(c), the baseline density of B-smooth integers among rough integers of the cofactor's size; no α, no α₂, no Dickman function. Rows per instance, side and bound are in `{out}.csv`. Definitions in `docs/baseline_calibration.md`.")
    L.append("")
    for side in ("algebraic", "rational"):
        L.append(f"## {side.capitalize()} side")
        L.append("")
        L.append("Relative gap observed/predicted − 1 per instance, with Q interpolated at the instance's mean ln c in each bin. The last rows: the mean over instances, the same with Q read per bin, the standard error of the mean from the spread across instances, the mean of the per-instance noise (instance count and baseline table together), the mean gap over the spread's standard error, and the mean weighted by each instance's noise with the χ² of the gaps about zero (the noise differs fifty-fold between instances, so the χ² and not the spread says whether the gaps are noise).")
        L.append("")
        L.append("| instance | " + " | ".join(label(b) for b in bounds) + " |")
        L.append("|---|" + "---:|" * len(bounds))
        cell = {(r["instance"], r["b"]): r for r in rows if r["side"] == side}
        for inst in instances:
            L.append(f"| {inst} | " + " | ".join(
                f"{pct(cell[(inst, b)]['rel_gap'])} ({cell[(inst, b)]['z']:+.1f}σ)" if (inst, b) in cell else "" for b in bounds) + " |")
        stats = {}
        for b in bounds:
            rs = [cell[(inst, b)] for inst in instances if (inst, b) in cell]
            m, sd = mean_std([r["rel_gap"] for r in rs])
            mb, _ = mean_std([r["rel_gap_bin"] for r in rs])
            sem = sd / math.sqrt(len(rs))
            noise = sum(r["rel_se"] for r in rs) / len(rs)
            wm, wse, chi0 = weighted([r["rel_gap"] for r in rs], [r["rel_se"] for r in rs])
            stats[b] = (m, mb, sem, noise, m / sem if sem > 0 else float("nan"), sum(r["uncovered"] for r in rs), sum(1 for r in rs if r["rel_gap"] > 0), len(rs), wm, wse, chi0)
        L.append("| **mean** | " + " | ".join(f"**{pct(stats[b][0])}**" for b in bounds) + " |")
        L.append("| mean, Q per bin | " + " | ".join(pct(stats[b][1]) for b in bounds) + " |")
        L.append("| spread / √n | " + " | ".join(f"{100 * stats[b][2]:.2f}%" for b in bounds) + " |")
        L.append("| mean per-instance noise | " + " | ".join(f"{100 * stats[b][3]:.2f}%" for b in bounds) + " |")
        L.append("| mean / (spread / √n) | " + " | ".join(f"{stats[b][4]:+.1f}" for b in bounds) + " |")
        L.append("| weighted mean (by 1/noise²) ± its error | " + " | ".join(f"{pct(stats[b][8])} ± {100 * stats[b][9]:.2f}%" for b in bounds) + " |")
        L.append("| χ² about zero (dof) | " + " | ".join(f"{stats[b][10]:.1f} ({stats[b][7]})" for b in bounds) + " |")
        L.append("| instances above zero | " + " | ".join(f"{stats[b][6]} of {stats[b][7]}" for b in bounds) + " |")
        L.append("| values outside the table | " + " | ".join(str(stats[b][5]) for b in bounds) + " |")
        L.append("")
        has_tail = all(r["tail_gap"] != "" for r in cell.values())
        if has_tail and max(abs(r["alpha_tail"]) for r in cell.values()) < 1e-3:
            L.append(f"**The α tail.** Below 0.001 for every instance on this side (a linear polynomial has one root at every prime), so nothing is removed.")
            L.append("")
        elif has_tail:
            L.append(f"**The α tail.** α over the primes in ({rough}, B] (computed up to the largest bound on file, at most B), the gap it predicts as exp(slope × tail) − 1 with the slope d ln(prediction)/d ln c read from the table, and the gap that remains. Correlation and regression slope are of the observed gap on the predicted one across instances.")
            L.append("")
            L.append("| | " + " | ".join(label(b) for b in bounds) + " |")
            L.append("|---|" + "---:|" * len(bounds))
            t = {}
            for b in bounds:
                rs = [cell[(inst, b)] for inst in instances if (inst, b) in cell]
                xs, ys = [r["tail_gap"] for r in rs], [r["rel_gap"] for r in rs]
                mx, my = sum(xs) / len(xs), sum(ys) / len(ys)
                cov = sum((x - mx) * (y - my) for x, y in zip(xs, ys))
                vx, vy = sum((x - mx) ** 2 for x in xs), sum((y - my) ** 2 for y in ys)
                m, sd = mean_std([r["rel_gap_after_tail"] for r in rs])
                _, sd_gap = mean_std(ys)
                _, sd_tail = mean_std([r["alpha_tail"] for r in rs])
                t[b] = (sd_tail, cov / math.sqrt(vx * vy) if vx > 0 and vy > 0 else float("nan"), cov / vx if vx > 0 else float("nan"), sd_gap, sd, m, sd / math.sqrt(len(rs)), max(r["alpha_tail_bound"] for r in rs))
            L.append("| tail computed up to | " + " | ".join(label(t[b][7]) for b in bounds) + " |")
            L.append("| spread of the tail across instances | " + " | ".join(f"{t[b][0]:.3f}" for b in bounds) + " |")
            L.append("| correlation, gap with tail prediction | " + " | ".join(f"{t[b][1]:+.2f}" for b in bounds) + " |")
            L.append("| regression slope | " + " | ".join(f"{t[b][2]:.2f}" for b in bounds) + " |")
            L.append("| spread of the gap | " + " | ".join(f"{100 * t[b][3]:.2f}%" for b in bounds) + " |")
            L.append("| spread after the tail | " + " | ".join(f"{100 * t[b][4]:.2f}%" for b in bounds) + " |")
            L.append("| **mean gap after the tail** | " + " | ".join(f"**{pct(t[b][5])}**" for b in bounds) + " |")
            L.append("| its standard error (spread / √n) | " + " | ".join(f"{100 * t[b][6]:.2f}%" for b in bounds) + " |")
            L.append("")
        has_first = bool(cell) and all(r["first_order"] != "" for r in cell.values())
        if has_first and max(abs(r["first_order"]) for r in cell.values()) < 1e-4:
            L.append(f"**The exact first-order term.** Below 0.01% for every instance on this side at every bound, so nothing is removed.")
            L.append("")
        elif has_first:
            L.append(f"**The exact first-order term.** The root counts r_p of `d1-tail` at every prime in ({rough}, B], summed over the values as η_p R(c/p)/R(c) (Q_B(c/p) − Q_B(c)), with η_p = (r_p p − p − 1)/(p² − 1), or (r_p^aff − 1)/(p − 1) over the affine roots alone at the primes above the region's largest b and less 1/(p − 1) where 0 is a root above its largest |a|, and R the density of rough integers (section 9 of the calibration document). Per instance: the term, and the gap that remains after it with its noise in σ.")
            L.append("")
            L.append("| instance | " + " | ".join(label(b) for b in bounds) + " |")
            L.append("|---|" + "---:|" * len(bounds))
            for inst in instances:
                L.append(f"| {inst} | " + " | ".join(
                    f"{pct(cell[(inst, b)]['first_order'])} → {pct(cell[(inst, b)]['rel_gap_after_first'])} ({cell[(inst, b)]['z_after_first']:+.1f}σ)"
                    if (inst, b) in cell else "" for b in bounds) + " |")
            L.append("")
            L.append("Across instances: correlation and regression slope of the observed gap on the term (a slope of 1 is a full explanation), the spread of the gap before and after, the χ² about zero before and after (consistent with noise when near the number of instances), the mean after, plain and weighted by the noise, with their errors, and the effect of the rough-integer density ratio R(c/p)/R(c).")
            L.append("")
            L.append("| | " + " | ".join(label(b) for b in bounds) + " |")
            L.append("|---|" + "---:|" * len(bounds))
            t = {}
            for b in bounds:
                rs = [cell[(inst, b)] for inst in instances if (inst, b) in cell]
                xs, ys = [r["first_order"] for r in rs], [r["rel_gap"] for r in rs]
                mx, my = sum(xs) / len(xs), sum(ys) / len(ys)
                cov = sum((x - mx) * (y - my) for x, y in zip(xs, ys))
                vx, vy = sum((x - mx) ** 2 for x in xs), sum((y - my) ** 2 for y in ys)
                after = [r["rel_gap_after_first"] for r in rs]
                m, sd = mean_std(after)
                _, sd_gap = mean_std(ys)
                ses = [r["rel_se"] for r in rs]
                _, _, chi_before = weighted(ys, ses)
                wm, wse, chi_after = weighted(after, ses)
                flat = max(abs(r["first_order"] - r["first_order_flat"]) for r in rs)
                t[b] = (cov / math.sqrt(vx * vy) if vx > 0 and vy > 0 else float("nan"), cov / vx if vx > 0 else float("nan"),
                        sd_gap, sd, sum(ses) / len(rs), m, sd / math.sqrt(len(rs)), chi_before, chi_after, len(rs),
                        sum(1 for r in rs if abs(r["z_after_first"]) > 3), flat, wm, wse)
            L.append("| correlation, gap with the term | " + " | ".join(f"{t[b][0]:+.2f}" for b in bounds) + " |")
            L.append("| regression slope | " + " | ".join(f"{t[b][1]:.2f}" for b in bounds) + " |")
            L.append("| spread of the gap | " + " | ".join(f"{100 * t[b][2]:.2f}%" for b in bounds) + " |")
            L.append("| spread after the term | " + " | ".join(f"{100 * t[b][3]:.2f}%" for b in bounds) + " |")
            L.append("| mean per-instance noise | " + " | ".join(f"{100 * t[b][4]:.2f}%" for b in bounds) + " |")
            L.append("| χ² about zero before the term (dof) | " + " | ".join(f"{t[b][7]:.1f} ({t[b][9]})" for b in bounds) + " |")
            L.append("| **χ² about zero after the term (dof)** | " + " | ".join(f"**{t[b][8]:.1f}** ({t[b][9]})" for b in bounds) + " |")
            L.append("| instances beyond 3σ after the term | " + " | ".join(str(t[b][10]) for b in bounds) + " |")
            L.append("| **mean gap after the term** | " + " | ".join(f"**{pct(t[b][5])}**" for b in bounds) + " |")
            L.append("| its standard error (spread / √n) | " + " | ".join(f"{100 * t[b][6]:.2f}%" for b in bounds) + " |")
            L.append("| **weighted mean after the term ± its error** | " + " | ".join(f"**{pct(t[b][12])}** ± {100 * t[b][13]:.2f}%" for b in bounds) + " |")
            L.append("| largest change from R(c/p)/R(c) | " + " | ".join(f"{100 * t[b][11]:.3f}%" for b in bounds) + " |")
            L.append("")
    Path(out + ".md").write_text("\n".join(L) + "\n")
    print(f"wrote {out}.csv ({len(rows)} rows) and {out}.md")


if __name__ == "__main__":
    main(sys.argv)
