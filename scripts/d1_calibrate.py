#!/usr/bin/env python3
"""Apply the empirical Dickman baseline (d1-baseline/1 or /2) to the instance results.

    scripts/d1_calibrate.py [--baseline data/results/baseline.consecutive.json]
                            [--out data/results/summary.calibrated]

Reads the baseline points table (consecutive integers near X, per bound B:
observed density beside rho(u), the derivative form rho + rho'/ln B and the
de Bruijn local form rho + gamma rho'/ln B), reports which form the random
integers follow, then recomputes every instance's prediction on both the
de Bruijn form and the empirical curve: per u bin, the alpha-adjusted
Dickman sum times the baseline's ratio to rho at the alpha-shifted u. The
criterion from docs/d1_falsifiable_prediction.md is applied per bound with
both weightings, absolute and relative. When data/results/alpha_moments.json
(alpha-moments/1, from d1-alpha) is present, the second-order term is added:
the prediction is multiplied by 1 + alpha2 rho''(u)/(2 rho(u) ln^2 B), the
convexity correction for the spread of the small-prime part that alpha's
single shift ignores, and the implied extra variance (the gap each instance
would need) is tabulated beside the computed alpha2. Writes <out>.csv,
<out>.baseline.csv and <out>.md. Standard library only.
"""
import csv
import glob
import json
import math
import statistics
import sys
from bisect import bisect_left
from pathlib import Path

MIN_SMOOTH_FOR_EMPIRICAL = 50
CHECK_MIN_SMOOTH = 1000
CHECK_U = (1.5, 6.0)


def load_baseline(path):
    result = json.loads(Path(path).read_text())
    if result.get("schema_version") not in ("d1-baseline/1", "d1-baseline/2"):
        sys.exit(f"{path}: not a d1-baseline/1 or d1-baseline/2 result")
    points = {}
    for size in result["results"]:
        for p in size["bounds"]:
            points.setdefault(p["b"], []).append(p)
    for b in points:
        points[b].sort(key=lambda p: p["u"])
    return result, points


def interpolate(xs, ys, x):
    """Linear interpolation, clamped to the end points."""
    if not xs:
        return float("nan")
    if x <= xs[0]:
        return ys[0]
    if x >= xs[-1]:
        return ys[-1]
    i = bisect_left(xs, x)
    x0, x1, y0, y1 = xs[i - 1], xs[i], ys[i - 1], ys[i]
    return y0 + (y1 - y0) * (x - x0) / (x1 - x0)


class Baseline:
    def __init__(self, points):
        self.debruijn = {}
        self.empirical = {}
        for b, ps in points.items():
            good = [p for p in ps if p["rho"] > 0]
            self.debruijn[b] = ([p["u"] for p in good], [p["debruijn_local"] / p["rho"] for p in good])
            emp = [p for p in good if p["smooth"] >= MIN_SMOOTH_FOR_EMPIRICAL]
            self.empirical[b] = ([p["u"] for p in emp], [p["observed"] / p["rho"] for p in emp])

    def ratio_debruijn(self, b, u):
        xs, ys = self.debruijn[b]
        return interpolate(xs, ys, u)

    def ratio_empirical(self, b, u):
        xs, ys = self.empirical[b]
        if not xs or u > xs[-1]:
            # Beyond the sizes with enough smooth values the closed form stands in.
            return self.ratio_debruijn(b, u)
        return interpolate(xs, ys, u)


class Rho:
    """rho(u) from the baseline table (it depends on u only): exact on [0, 2],
    log-linear interpolation of the tabulated values above 2. rho'' from
    rho'(u) = -rho(u-1)/u: rho''(u) = rho(u-2)/(u(u-1)) + rho(u-1)/u^2."""

    def __init__(self, points):
        grid = sorted({(round(p["u"], 9), p["rho"]) for ps in points.values() for p in ps if p["u"] > 2 and p["rho"] > 0})
        self.us = [g[0] for g in grid]
        self.lns = [math.log(g[1]) for g in grid]

    def rho(self, u):
        if u <= 1:
            return 1.0
        if u <= 2:
            return 1.0 - math.log(u)
        if u >= self.us[-1]:
            return math.exp(self.lns[-1])
        i = bisect_left(self.us, u)
        u0, u1, l0, l1 = self.us[i - 1], self.us[i], self.lns[i - 1], self.lns[i]
        return math.exp(l0 + (l1 - l0) * (u - u0) / (u1 - u0))

    def rho2(self, u):
        if u <= 1:
            return 0.0
        if u <= 2:
            return 1.0 / (u * u)
        return self.rho(u - 2) / (u * (u - 1)) + self.rho(u - 1) / (u * u)


def load_alpha2(path="data/results/alpha_moments.json"):
    p = Path(path)
    if not p.is_file():
        return None
    doc = json.loads(p.read_text())
    if doc.get("schema_version") != "alpha-moments/1":
        return None
    return {(e["instance"], side): e[side]["alpha2"] for e in doc["polynomials"] for side in ("algebraic", "rational")}


def baseline_check(points):
    """Per bound: mean and max |observed/form − 1| over the well-measured points."""
    rows = []
    for b, ps in sorted(points.items()):
        sel = [p for p in ps if p["smooth"] >= CHECK_MIN_SMOOTH and CHECK_U[0] <= p["u"] <= CHECK_U[1]]
        if not sel:
            continue
        row = {"b": b, "points": len(sel), "u_min": min(p["u"] for p in sel), "u_max": max(p["u"] for p in sel)}
        for form in ("rho", "local", "debruijn_local"):
            devs = [p["observed"] / p[form] - 1 for p in sel if p[form] > 0]
            sig = [(p["observed"] - p[form]) / p["observed_stderr"] for p in sel if p["observed_stderr"] > 0]
            row[f"{form}_mean_dev"] = statistics.mean(devs)
            row[f"{form}_max_abs_dev"] = max(abs(d) for d in devs)
            row[f"{form}_mean_abs_sigma"] = statistics.mean(abs(s) for s in sig)
        rows.append(row)
    return rows


def load_bins(stem):
    path = Path(stem + ".u_bins.csv")
    out = {}
    with path.open() as fh:
        for r in csv.DictReader(fh):
            key = (int(r["b"]), r["side"])
            out.setdefault(key, []).append(r)
    return out


def main(argv):
    baseline_path, out = "data/results/baseline.consecutive.json", "data/results/summary.calibrated"
    i = 1
    while i < len(argv):
        if argv[i] == "--baseline":
            baseline_path = argv[i + 1]; i += 2
        elif argv[i] == "--out":
            out = argv[i + 1]; i += 2
        else:
            sys.exit("usage: d1_calibrate.py [--baseline FILE] [--out PATH]")
    base_result, points = load_baseline(baseline_path)
    baseline = Baseline(points)
    check = baseline_check(points)
    rho = Rho(points)
    alpha2 = load_alpha2()

    rows = []
    instances = set()
    for f in sorted(glob.glob("data/results/d1-*.scaled-down*.json")):
        r = json.loads(Path(f).read_text())
        if r.get("schema_version") not in ("d1-measurement/2", "d1-measurement/3", "d1-measurement/4"):
            continue
        inst = Path(f).name.split(".")[0]
        instances.add(inst)
        stem = f[: -len(".json")]
        bins = load_bins(stem)
        for entry in r["results"]:
            b = entry["b"]
            if b not in points:
                continue
            ln_b = math.log(b)
            for side in ("algebraic", "rational"):
                s = entry[side]
                alpha = s["alpha"]
                shift = alpha / ln_b
                a2 = alpha2.get((inst, side)) if alpha2 else None
                pred_alpha = pred_db = pred_emp = pred_a2 = 0.0
                curv_sum = 0.0
                for row in bins.get((b, side), []):
                    count = int(row["count"])
                    if count == 0:
                        continue
                    u_center = 0.5 * (float(row["u_lo"]) + float(row["u_hi"]))
                    u_alpha = max(u_center + shift, 0.0)
                    part = float(row["predicted_alpha"]) * count
                    pred_alpha += part
                    r_db = baseline.ratio_debruijn(b, u_alpha)
                    pred_db += part * r_db
                    pred_emp += part * baseline.ratio_empirical(b, u_alpha)
                    # Second-order term: the convexity of rho times the excess variance of the size shift.
                    curv = rho.rho2(u_alpha) / rho.rho(u_alpha) / (2 * ln_b * ln_b) if rho.rho(u_alpha) > 0 else 0.0
                    curv_sum += part * r_db * curv
                    if a2 is not None:
                        pred_a2 += part * r_db * (1 + a2 * curv)
                n = r["evaluated"]
                pred_alpha /= n; pred_db /= n; pred_emp /= n; pred_a2 /= n; curv_sum /= n
                # The extra variance that would close the gap against the de Bruijn baseline.
                implied_var = (s["observed"] - pred_db) / curv_sum if curv_sum > 0 else float("nan")
                rows.append({
                    "instance": inst, "digits": r["n_digits"], "b": b, "side": side, "evaluated": n,
                    "smooth": s["smooth"], "observed": s["observed"], "observed_stderr": s["observed_stderr"],
                    "predicted_alpha": s["predicted_alpha"], "predicted_alpha_from_bins": pred_alpha,
                    "predicted_debruijn": pred_db, "predicted_empirical": pred_emp,
                    "gap_alpha": s["observed"] - s["predicted_alpha"],
                    "gap_debruijn": s["observed"] - pred_db,
                    "gap_empirical": s["observed"] - pred_emp,
                    "rel_gap_alpha": s["observed"] / s["predicted_alpha"] - 1 if s["predicted_alpha"] else float("nan"),
                    "rel_gap_debruijn": s["observed"] / pred_db - 1 if pred_db else float("nan"),
                    "rel_gap_empirical": s["observed"] / pred_emp - 1 if pred_emp else float("nan"),
                    "alpha2": a2 if a2 is not None else "",
                    "implied_variance": implied_var,
                    "predicted_alpha2": pred_a2 if a2 is not None else "",
                    "gap_alpha2": s["observed"] - pred_a2 if a2 is not None else "",
                    "rel_gap_alpha2": s["observed"] / pred_a2 - 1 if a2 is not None and pred_a2 else "",
                    "mean_u": s["mean_u"], "alpha": alpha,
                })
    if not rows:
        sys.exit("no instance results found")
    # Two files cover the same bound 2^20 (the 2^14–2^20 run and the 2^20–2^28 run); keep one row per instance, bound, side.
    seen = set()
    unique = []
    for row in rows:
        key = (row["instance"], row["b"], row["side"])
        if key in seen:
            continue
        seen.add(key)
        unique.append(row)
    rows = sorted(unique, key=lambda r: (r["b"], r["side"], r["instance"]))

    out_csv = Path(out + ".csv")
    out_csv.parent.mkdir(parents=True, exist_ok=True)
    with out_csv.open("w", newline="") as fh:
        w = csv.DictWriter(fh, fieldnames=list(rows[0].keys()))
        w.writeheader(); w.writerows(rows)
    check_csv = Path(out + ".baseline.csv")
    with check_csv.open("w", newline="") as fh:
        w = csv.DictWriter(fh, fieldnames=list(check[0].keys()))
        w.writeheader(); w.writerows(check)

    def exp(b):
        return f"2^{int(b).bit_length() - 1}"

    L = []
    L.append("# D1 summary against the calibrated baseline")
    L.append("")
    L.append(f"Baseline run `{base_result['run_id']}`: {base_result['sizes']['count']} sizes, ln X from {base_result['sizes']['ln_x_min']:g} to {base_result['sizes']['ln_x_max']:g} in steps of {base_result['sizes']['ln_x_step']:g}, {base_result['interval_length']:,} consecutive integers each, {base_result['elapsed_ms']/1000:.0f} s. Definitions in `docs/baseline_calibration.md`. Per instance, bound and side the rows are in `{out_csv}`; the baseline check is in `{check_csv}`.")
    L.append("")
    L.append("## Which Dickman form do integers without structure follow?")
    L.append("")
    L.append(f"Over the baseline points with at least {CHECK_MIN_SMOOTH:,} smooth values and u in [{CHECK_U[0]}, {CHECK_U[1]}]: mean of observed/form − 1, the largest |observed/form − 1|, and the mean |observed − form| in Poisson standard errors.")
    L.append("")
    L.append("| B | points | u range | ρ(u): mean dev | max | mean σ | ρ + ρ′/ln B: mean dev | max | mean σ | ρ + γρ′/ln B: mean dev | max | mean σ |")
    L.append("|---|---|---|---|---|---|---|---|---|---|---|---|")
    for c in check:
        L.append(
            f"| {exp(c['b'])} | {c['points']} | {c['u_min']:.2f}–{c['u_max']:.2f} "
            f"| {c['rho_mean_dev']*100:+.1f}% | {c['rho_max_abs_dev']*100:.1f}% | {c['rho_mean_abs_sigma']:.1f} "
            f"| {c['local_mean_dev']*100:+.1f}% | {c['local_max_abs_dev']*100:.1f}% | {c['local_mean_abs_sigma']:.1f} "
            f"| {c['debruijn_local_mean_dev']*100:+.1f}% | {c['debruijn_local_max_abs_dev']*100:.1f}% | {c['debruijn_local_mean_abs_sigma']:.1f} |"
        )
    L.append("")
    L.append("## Rejection criterion, algebraic side, against each baseline")
    L.append("")
    L.append("Each prediction is the α-adjusted Dickman sum of the run, per u bin, times the baseline's ratio to ρ at the α-shifted u. `α only` is the prediction the earlier summaries used. The gap is observed − predicted. The noise is the Poisson noise of the counts (√Σse² / n) and, beside it, the spread of the gaps across instances (std / √n). Absolute gaps weight the dense small instances most; relative gaps weight every instance alike.")
    L.append("")
    bounds = sorted({row["b"] for row in rows})
    baselines = [("α only", "gap_alpha", "rel_gap_alpha", "predicted_alpha"), ("de Bruijn local form", "gap_debruijn", "rel_gap_debruijn", "predicted_debruijn"), ("empirical curve", "gap_empirical", "rel_gap_empirical", "predicted_empirical")]
    if alpha2:
        baselines.append(("de Bruijn local form with the second-order term α₂", "gap_alpha2", "rel_gap_alpha2", "predicted_alpha2"))
    for label, key, rel, pred_key in baselines:
        L.append(f"### Baseline: {label}")
        L.append("")
        L.append("| B | n | mean abs gap | Poisson noise | gap / noise | spread | gap / spread | verdict (absolute) | mean rel gap | rel Poisson noise | rel spread | verdict (relative) |")
        L.append("|---|---|---|---|---|---|---|---|---|---|---|---|")
        for b in bounds:
            rs = [row for row in rows if row["b"] == b and row["side"] == "algebraic"]
            n = len(rs)
            gaps = [row[key] for row in rs]
            rels = [row[rel] for row in rs]
            mean = statistics.mean(gaps)
            poisson = math.sqrt(sum(row["observed_stderr"] ** 2 for row in rs)) / n
            spread = statistics.stdev(gaps) / math.sqrt(n) if n > 1 else float("nan")
            rel_mean = statistics.mean(rels)
            rel_poisson = math.sqrt(sum((row["observed_stderr"] / row[pred_key]) ** 2 for row in rs)) / n
            rel_spread = statistics.stdev(rels) / math.sqrt(n) if n > 1 else float("nan")
            v_abs = "rejected" if abs(mean) < 2 * poisson else "not rejected"
            v_rel = "rejected" if abs(rel_mean) < 2 * rel_poisson else "not rejected"
            L.append(
                f"| {exp(b)} | {n} | {mean:+.6f} | {poisson:.6f} | {mean/poisson if poisson else 0:+.1f} | {spread:.6f} | {mean/spread if spread else 0:+.1f} | {v_abs} "
                f"| {rel_mean*100:+.2f}% | {rel_poisson*100:.2f}% | {rel_spread*100:.2f}% | {v_rel} |"
            )
        L.append("")
    L.append("## Per instance, algebraic side: relative gap against the de Bruijn baseline")
    L.append("")
    insts = sorted(instances)
    L.append("| instance | digits | " + " | ".join(exp(b) for b in bounds) + " | α |")
    L.append("|---|---|" + "---|" * len(bounds) + "---|")
    for inst in insts:
        cells = []
        digits = alpha = None
        for b in bounds:
            match = [row for row in rows if row["instance"] == inst and row["b"] == b and row["side"] == "algebraic"]
            if match:
                digits = match[0]["digits"]; alpha = match[0]["alpha"]
                cells.append(f"{match[0]['rel_gap_debruijn']*100:+.1f}%")
            else:
                cells.append("")
        L.append(f"| {inst} | {digits} | " + " | ".join(cells) + f" | {alpha:+.2f} |")
    L.append("")
    if alpha2:
        L.append("## Per instance, algebraic side: relative gap after the second-order term α₂")
        L.append("")
        L.append("The prediction is the de Bruijn baseline times 1 + α₂ ρ″(u)/(2 ρ(u) ln² B) per u bin, with α₂ computed from the polynomial (`data/results/alpha_moments.csv`). At 2^14 the shift α/ln B is about 0.4 in u and the second-order expansion is coarse; from 2^20 up it is the regime the expansion is built for.")
        L.append("")
        L.append("| instance | digits | " + " | ".join(exp(b) for b in bounds) + " | α | α₂ |")
        L.append("|---|---|" + "---|" * len(bounds) + "---|---|")
        for inst in insts:
            cells = []
            digits = alpha = a2 = None
            for b in bounds:
                match = [row for row in rows if row["instance"] == inst and row["b"] == b and row["side"] == "algebraic"]
                if match and match[0]["rel_gap_alpha2"] != "":
                    digits = match[0]["digits"]; alpha = match[0]["alpha"]; a2 = match[0]["alpha2"]
                    cells.append(f"{match[0]['rel_gap_alpha2']*100:+.1f}%")
                else:
                    cells.append("")
            L.append(f"| {inst} | {digits} | " + " | ".join(cells) + f" | {alpha:+.2f} | {a2:+.2f} |")
        L.append("")
        L.append("## Implied extra variance against computed α₂, algebraic side")
        L.append("")
        L.append("The implied value is the extra variance of the size shift (in ln² units) that would close each instance's gap against the de Bruijn baseline at that bound, (observed − predicted) / Σ predicted·ρ″/(2ρ ln² B). A residual that is a pure second-order effect gives the same number at every bound, and that number is α₂.")
        L.append("")
        L.append("| instance | " + " | ".join(exp(b) for b in bounds) + " | α₂ computed |")
        L.append("|---|" + "---|" * len(bounds) + "---|")
        for inst in insts:
            cells = []
            a2 = None
            for b in bounds:
                match = [row for row in rows if row["instance"] == inst and row["b"] == b and row["side"] == "algebraic"]
                if match:
                    a2 = match[0]["alpha2"]
                    cells.append(f"{match[0]['implied_variance']:.1f}")
                else:
                    cells.append("")
            L.append(f"| {inst} | " + " | ".join(cells) + f" | {a2:+.2f} |")
        L.append("")
    L.append("## Per instance, rational side: relative gap against the de Bruijn baseline")
    L.append("")
    L.append("| instance | " + " | ".join(exp(b) for b in bounds) + " | α |")
    L.append("|---|" + "---|" * len(bounds) + "---|")
    for inst in insts:
        cells = []
        alpha = None
        for b in bounds:
            match = [row for row in rows if row["instance"] == inst and row["b"] == b and row["side"] == "rational"]
            if match:
                alpha = match[0]["alpha"]
                cells.append(f"{match[0]['rel_gap_debruijn']*100:+.1f}%")
            else:
                cells.append("")
        L.append(f"| {inst} | " + " | ".join(cells) + f" | {alpha:+.2f} |")
    Path(out + ".md").write_text("\n".join(L) + "\n")
    print(f"wrote {out_csv} ({len(rows)} rows), {check_csv} and {out}.md")


if __name__ == "__main__":
    main(sys.argv)
