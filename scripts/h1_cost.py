#!/usr/bin/env python3
"""H1: integrate the sampled sieving cost of RSA-200 on this machine to CADO-NFS's
relation target (docs/h1_falsifiable_prediction.md).

    scripts/h1_cost.py [--dir data/results/h1] [--out data/results/h1_cost]

Reads the statistics lines `las -random-sample ... -T` wrote for every sampled range
(<dir>/las.<q>.txt) and the hardware traces (data/hardware/h1-c200-q<q>.jsonl, when
present). Per range: special-q's, relations, wall seconds at the run's threads, CPU
use, the survivor counts and the categorized timings. Then, with one special-q per
root of f mod q and one root per prime on average, R(Q) = ∫ y dq / ln q and
T(Q) = ∫ t dq / ln q from qmin, ln y and ln t linear in ln q between the ranges and
on the last segment's slope beyond. Writes <out>.csv and <out>.md. Standard library only.
"""
import csv
import json
import math
import re
import sys
from pathlib import Path

QMIN = 5e7
LPB = (32, 33)
TARGET = 0.81 * sum(2 ** l / math.log(2 ** l) for l in LPB)  # cadofactor's rels_wanted rule
CLAIM, LOW, HIGH = 6300.0, 3150.0, 12600.0
CATEGORIES = {
    "norms": ("4", "260"),
    "sieving": ("2", "258", "3"),
    "survivor search": ("1",),
    "cofactorization": ("5", "261", "6"),
    "bookkeeping": ("0", "-1"),
}


def parse(path):
    text = path.read_text()
    out = {}
    m = re.search(r"# Total (\d+) reports \[([\d.e+-]+)s/r, ([\d.e+-]+)r/sq\] in ([\d.]+) elapsed s \[([\d.]+)% CPU\]", text)
    if not m:
        sys.exit(f"{path}: no closing statistics (did las finish?)")
    out["relations"] = int(m.group(1))
    out["elapsed"] = float(m.group(4))
    out["cpu_pct"] = float(m.group(5))
    nq = re.findall(r"out of (\d+) pushed", text) or re.findall(r"for (\d+) special-q's", text)
    out["special_q"] = int(nq[-1])
    disc = re.findall(r"# Discarded (\d+) special-q's", text)
    out["discarded"] = int(disc[-1]) if disc else 0
    cpu = re.search(r"# Total cpu time ([\d.]+)s", text)
    out["cpu"] = float(cpu.group(1)) if cpu else math.nan
    mem = re.search(r"PeakMemusage \(MB\) = (\d+)", text)
    out["peak_mb"] = int(mem.group(1)) if mem else 0
    cats = {}
    block = text.split("# Categorized timings", 1)
    if len(block) == 2:
        for line in block[1].splitlines()[1:]:
            mm = re.match(r"# (-?\d+) [^:]*: ([\d.]+)", line)
            if mm:
                cats[mm.group(1)] = float(mm.group(2))
            elif line.startswith("# total counted time"):
                break
    out["categories"] = cats
    out["survivors"] = {k: int(v) for k, v in re.findall(r"# survivors (\S+): (\d+)", text)}
    return out


def hardware(q):
    p = Path(f"data/hardware/h1-c200-q{q}.jsonl")
    if not p.exists():
        return None
    for line in reversed(p.read_text().splitlines()):
        r = json.loads(line)
        if r.get("kind") == "finish":
            return r["summary"]
    return None


def interp(points, x):
    """ln y at ln q = x, linear between points (sorted by x), the end slopes beyond."""
    if x <= points[0][0]:
        (x0, y0), (x1, y1) = points[0], points[1]
    elif x >= points[-1][0]:
        (x0, y0), (x1, y1) = points[-2], points[-1]
    else:
        i = max(k for k in range(len(points)) if points[k][0] <= x)
        (x0, y0), (x1, y1) = points[i], points[min(i + 1, len(points) - 1)]
    return y0 if x1 == x0 else y0 + (y1 - y0) * (x - x0) / (x1 - x0)


def main(argv):
    d, out = Path("data/results/h1"), "data/results/h1_cost"
    i = 1
    while i < len(argv):
        if argv[i] == "--dir":
            d = Path(argv[i + 1]); i += 2
        elif argv[i] == "--out":
            out = argv[i + 1]; i += 2
        else:
            sys.exit("usage: h1_cost.py [--dir DIR] [--out PATH]")
    runs = []
    for f in sorted(d.glob("las.*.txt"), key=lambda p: int(p.stem.split(".")[1])):
        q = int(f.stem.split(".")[1])
        r = parse(f)
        r["q"] = q
        r["q_mid"] = q * 1.025
        r["y"] = r["relations"] / r["special_q"]
        r["t"] = r["elapsed"] / r["special_q"]
        r["hw"] = hardware(q)
        runs.append(r)
    if len(runs) < 2:
        sys.exit(f"{d}: need at least two sampled ranges")
    ly = [(math.log(r["q_mid"]), math.log(r["y"])) for r in runs]
    lt = [(math.log(r["q_mid"]), math.log(r["t"])) for r in runs]

    # integrate in ln q: dq / ln q = q d(ln q) / ln q
    step = 1e-4
    x, R, T, NQ = math.log(QMIN), 0.0, 0.0, 0.0
    q_max = None
    x_end = math.log(1e12)
    while x < x_end:
        xm = x + step / 2
        dq = math.exp(xm) * step / xm
        R += math.exp(interp(ly, xm)) * dq
        T += math.exp(interp(lt, xm)) * dq
        NQ += dq
        x += step
        if R >= TARGET:
            q_max = math.exp(x)
            break
    hours = T / 3600.0
    sampled_to = max(r["q"] * 1.05 for r in runs)
    extrapolated = q_max is None or q_max > sampled_to
    verdict = "holds" if LOW <= hours <= HIGH else "rejected"

    # breakdown over all ranges, weighted by each range's share of T
    share = {}
    for r in runs:
        tot = sum(r["categories"].values()) or 1.0
        for name, ids in CATEGORIES.items():
            share.setdefault(name, []).append(sum(r["categories"].get(c, 0.0) for c in ids) / tot)

    rows = []
    for r in runs:
        s = r["survivors"]
        hw = r["hw"] or {}
        rows.append(dict(
            q=r["q"], special_q=r["special_q"], relations=r["relations"], rels_per_q=r["y"], seconds_per_q=r["t"],
            cpu_pct=r["cpu_pct"], cpu_s=r["cpu"], peak_mb=r["peak_mb"],
            before_sieve=s.get("before_sieve", 0), after_sieve=s.get("after_sieve", 0),
            enter_cofactoring=s.get("enter_cofactoring", 0), smooth=s.get("smooth", 0),
            **{f"share_{k.replace(' ', '_')}": share[k][runs.index(r)] for k in CATEGORIES},
            power_w_mean=(hw.get("power_all_w") or {}).get("mean", ""),
            cpu_temp_max=(hw.get("cpu_temp_c") or {}).get("max", ""),
            cpu_active_mean=(hw.get("cpu_active_ratio") or {}).get("mean", "")))
    with open(out + ".csv", "w", newline="") as fh:
        w = csv.DictWriter(fh, fieldnames=list(rows[0].keys()))
        w.writeheader()
        for r in rows:
            w.writerow({k: (f"{v:.6g}" if isinstance(v, float) else v) for k, v in r.items()})

    md = ["# H1: the sieving cost of RSA-200 on this machine\n",
          "Generated by `scripts/h1_cost.py` from `scripts/h1_test_sieve.sh`. The prediction and criterion are"
          " `docs/h1_falsifiable_prediction.md`. CADO-NFS `params.c200` with its `parameters/polynomials/c200.poly`"
          f" (sha256 in `data/results/h1/c200.poly.sha256`); `las -t` threads as recorded per range.\n",
          "## Sampled ranges\n",
          "| q range | special-q's | relations | relations per q | wall s per q | CPU | peak MB | mean power W | max CPU °C |",
          "|---|---:|---:|---:|---:|---:|---:|---:|---:|"]
    for r in runs:
        hw = r["hw"] or {}
        pw = (hw.get("power_all_w") or {}).get("mean")
        tc = (hw.get("cpu_temp_c") or {}).get("max")
        md.append(f"| [{r['q']:.3g}, {1.05 * r['q']:.3g}) | {r['special_q']} | {r['relations']:,} | {r['y']:.1f} | {r['t']:.2f} |"
                  f" {r['cpu_pct']:.0f}% | {r['peak_mb']:,} | {pw:.1f} | {tc:.1f} |" if pw is not None else
                  f"| [{r['q']:.3g}, {1.05 * r['q']:.3g}) | {r['special_q']} | {r['relations']:,} | {r['y']:.1f} | {r['t']:.2f} |"
                  f" {r['cpu_pct']:.0f}% | {r['peak_mb']:,} | n/a | n/a |")
    md.append("\n## The integral to the target\n")
    md.append(f"- Target: {TARGET:,.0f} relations (0.81 × Σ 2^lpb / ln 2^lpb, lpb = 32 and 33).")
    if q_max is None:
        md.append("- The target is not reached below 10^12 on the extrapolated slopes.")
    else:
        md.append(f"- Reached at Q_max = {q_max:.3g} after {NQ:,.0f} special-q's"
                  + (" (beyond the sampled ranges: extrapolated on the last slope)." if extrapolated else "."))
    md.append(f"- Wall time at the sampled thread count: {T:,.0f} s = **{hours:,.0f} laptop-hours** ({hours / 24 / 365:.2f} laptop-years).")
    md.append(f"- Relations per laptop-hour on average: {TARGET / hours:,.0f}; special-q's per laptop-hour: {NQ / hours:,.0f}.")
    md.append(f"- One hour at 200 digits needs {hours:,.0f}× this machine's relation collection.")
    md.append(f"\n**H1 {verdict}**: the measured {hours:,.0f} laptop-hours "
              + ("lies" if verdict == "holds" else "does not lie") + f" within [{LOW:,.0f}, {HIGH:,.0f}], a factor of 2 around {CLAIM:,.0f}.\n")
    md.append("## Where the time goes\n")
    md.append("Share of `las`'s counted time per category, per range:\n")
    md.append("| q | " + " | ".join(CATEGORIES) + " |")
    md.append("|---|" + "---:|" * len(CATEGORIES))
    for k, r in enumerate(runs):
        md.append(f"| {r['q']:.3g} | " + " | ".join(f"{100 * share[c][k]:.0f}%" for c in CATEGORIES) + " |")
    md.append("\nSurvivors per special-q: the sieve region, what passes the sieve, what enters cofactorization, what is smooth:\n")
    md.append("| q | region | after the sieve | enter cofactorization | relations |")
    md.append("|---|---:|---:|---:|---:|")
    for r in runs:
        s = r["survivors"]
        n = r["special_q"]
        md.append(f"| {r['q']:.3g} | {s.get('before_sieve', 0) / n:,.0f} | {s.get('after_sieve', 0) / n:,.0f} |"
                  f" {s.get('enter_cofactoring', 0) / n:,.0f} | {r['relations'] / n:.1f} |")
    Path(out + ".md").write_text("\n".join(md) + "\n")
    print(f"target {TARGET:,.0f} relations; Q_max {q_max if q_max else float('nan'):.4g}; {hours:,.1f} laptop-hours; H1 {verdict}")
    print(f"wrote {out}.md and {out}.csv")


if __name__ == "__main__":
    main(sys.argv)
