#!/usr/bin/env python3
"""H6: block Wiedemann at 200 digits on this laptop (docs/h6_falsifiable_prediction.md).

    scripts/h6_summary.py [--out data/results/h6_linalg]
    scripts/h6_summary.py --n200        print N_200 = r_130 × ΣL_200 × κ

Reads data/results/h6/ from scripts/h6_linalg.sh:
  merge110.txt, merge170.txt      merge's final matrix at density 110 (H4) and 170 (κ);
  <tag>.matrix.txt                the matrix line bwc's balancing printed (rows, cols, weight);
  <tag>.checkpoints.csv           the iteration and mtime of each krylov checkpoint V0-64.<k>;
  <tag>.hw.json, lingen.hw.json   hw-sample summaries;
  lingen.txt                      lingen's length and its wall time.
Time per iteration is the median over the checkpoint intervals after the first, which holds
the start-up. ρ = real / synthetic at RSA-130's shape. T_LA = (3 N_200 / 64 × t_200 × ρ +
T_lingen) × 1.064. Writes <out>.csv and <out>.md. Standard library only.
"""
import csv
import json
import math
import re
import statistics
import sys
from pathlib import Path

R = Path("data/results/h6")
LPB = {100: (25, 26), 110: (26, 27), 120: (27, 28), 130: (28, 29), 200: (32, 33)}
H4_ROWS = {100: 174665, 110: 363780, 120: 662253, 130: 1183823}
H4_LA_130 = {"krylov": 488.77, "lingen": 205.7, "mksol": 272.98, "phase": 1029.5, "weight": 131879138, "iters": 39000}
OVERHEAD = H4_LA_130["phase"] / (H4_LA_130["krylov"] + H4_LA_130["lingen"] + H4_LA_130["mksol"])
CLAIM, LOW, HIGH = 46.0, 23.0, 92.0
GATE = 0.25
H1_RELATIONS_HOURS = 3896.0
TAGS = ("real130", "syn130", "synq4", "synq2", "syn200")


def sigma_l(d):
    return sum(2 ** b / (b * math.log(2)) for b in LPB[d])


def final_matrix(path):
    m = re.search(r"Final matrix has N=(\d+) nc=(\d+) \((\d+)\) W=(\d+)", Path(path).read_text())
    if not m:
        sys.exit(f"{path}: no final matrix line")
    return int(m.group(1)), int(m.group(4))


def kappa():
    n170, w170 = final_matrix(R / "merge170.txt")
    n110, w110 = final_matrix(R / "merge110.txt")
    return n170 / n110, n170, w170, n110, w110


def n200(ratio=None):
    k = kappa()[0]
    r = ratio if ratio is not None else H4_ROWS[130] / sigma_l(130)
    return round(r * sigma_l(200) * k)


MATRIX_LINE = re.compile(r"(\d+) rows (\d+) cols(?: \(\d+ extra rows\))? weight (\d+)")


def matrix_info(tag):
    """Rows, columns and weight from the line bwc's balancing printed for this matrix."""
    m = MATRIX_LINE.search((R / f"{tag}.matrix.txt").read_text())
    if not m:
        sys.exit(f"{R / f'{tag}.matrix.txt'}: no matrix line")
    return int(m.group(1)), int(m.group(2)), int(m.group(3))


def points(tag):
    with open(R / f"{tag}.checkpoints.csv") as fh:
        return sorted((int(r["iteration"]), float(r["mtime"])) for r in csv.DictReader(fh))


def intervals(tag):
    pts = points(tag)
    return [((b[1] - a[1]) / (b[0] - a[0]), a[0], b[0]) for a, b in zip(pts, pts[1:])]


GPU_CLEAN = 0.05  # an interval counts as clean when the GPU, which bwc does not use, was this idle


def interval_table(tag):
    """Per checkpoint interval: seconds per iteration and, from the local hw-sample trace, the
    activity of the GPU (other work on the machine) and the CPU, the clocks and the CPU power.
    Written to <tag>.intervals.csv when the trace is present, read back from it otherwise."""
    path = R / f"{tag}.intervals.csv"
    trace = Path(f"data/hardware/h6-{tag}.jsonl")
    pts = points(tag)
    if not trace.exists():
        if not path.exists():
            return [dict(start=a[0], end=b[0], s_per_iter=(b[1] - a[1]) / (b[0] - a[0]), gpu_active=math.nan)
                    for a, b in zip(pts, pts[1:])]
        with open(path) as fh:
            return [{k: (float(v) if k not in ("start", "end", "samples") else int(v)) for k, v in r.items()}
                    for r in csv.DictReader(fh)]
    samp = [r for r in (json.loads(x) for x in trace.read_text().splitlines() if x.strip()) if r.get("kind") == "sample"]

    def mean(xs):
        xs = [x for x in xs if isinstance(x, (int, float))]
        return sum(xs) / len(xs) if xs else math.nan

    rows = []
    for (i0, t0), (i1, t1) in zip(pts, pts[1:]):
        sel = [r for r in samp if t0 - 0.5 <= r["ts_ms"] / 1000 <= t1 + 0.5]
        clocks = {}
        for r in sel:
            for t in (r.get("cpu") or {}).get("tiers", []):
                clocks.setdefault(t.get("label"), []).append(t.get("freq_mhz"))
        rows.append(dict(start=i0, end=i1, s_per_iter=(t1 - t0) / (i1 - i0),
                         gpu_active=mean([(r.get("gpu") or {}).get("active_ratio") for r in sel]),
                         cpu_active=mean([(r.get("cpu") or {}).get("active_ratio") for r in sel]),
                         clock_p_mhz=statistics.median(clocks["P"]) if clocks.get("P") else math.nan,
                         clock_s_mhz=statistics.median(clocks["S"]) if clocks.get("S") else math.nan,
                         cpu_w=mean([(r.get("cpu") or {}).get("power_w") for r in sel]), samples=len(sel)))
    with open(path, "w", newline="") as fh:
        wr = csv.DictWriter(fh, fieldnames=list(rows[0].keys()))
        wr.writeheader()
        for r in rows:
            wr.writerow({k: (f"{v:.6g}" if isinstance(v, float) else v) for k, v in r.items()})
    return rows


def clean_median(rows):
    xs = [r["s_per_iter"] for r in rows[1:] if r["gpu_active"] < GPU_CLEAN]
    return (statistics.median(xs), len(xs)) if len(xs) >= 2 else (math.nan, len(xs))


def steady(tag):
    per = intervals(tag)
    if len(per) < 2:
        sys.exit(f"{tag}: fewer than two krylov intervals")
    return statistics.median(p[0] for p in per[1:]), per


def hw(name):
    p = R / name
    if not p.exists() or not p.read_text().strip():
        return {}
    t = p.read_text()
    return json.loads(t[t.index("{"):])


def stat(h, key, field):
    v = (h.get(key) or {}).get(field)
    return v if isinstance(v, (int, float)) else math.nan


def trace_clocks(run_id):
    p = Path(f"data/hardware/{run_id}.jsonl")
    if not p.exists():
        return math.nan, math.nan
    clocks = {}
    for line in p.read_text().splitlines():
        r = json.loads(line)
        if r.get("kind") != "sample" or (r.get("cpu") or {}).get("active_ratio", 0) < 0.5:
            continue
        for t in r["cpu"].get("tiers", []):
            clocks.setdefault(t.get("label"), []).append(t.get("freq_mhz"))
    med = lambda k: statistics.median(clocks[k]) if clocks.get(k) else math.nan  # noqa: E731
    return med("P"), med("S")


def lingen():
    t = (R / "lingen.txt").read_text()
    length = int(re.search(r"length (\d+)", t).group(1))
    wct = float(re.search(r"\(wct\) ([\d.]+)", t).group(1))
    return length, wct


def main(argv):
    if len(argv) == 2 and argv[1] == "--n200":
        print(n200())
        return
    out = "data/results/h6_linalg"
    if len(argv) == 3 and argv[1] == "--out":
        out = argv[2]
    elif len(argv) != 1:
        sys.exit("usage: h6_summary.py [--out PATH] | --n200")

    k, n170, w170, n110, w110 = kappa()
    N = n200()
    ratios = [H4_ROWS[d] / sigma_l(d) for d in H4_ROWS]
    n_lo, n_hi = n200(min(ratios)), n200(max(ratios))

    rows = []
    t = {}
    for tag in TAGS:
        if not (R / f"{tag}.checkpoints.csv").exists():
            continue
        n, nc, w = matrix_info(tag)
        s, per = steady(tag)
        t[tag] = s
        h = hw(f"{tag}.hw.json")
        p_mhz, s_mhz = trace_clocks(f"h6-{tag}")
        rows.append(dict(tag=tag, rows=n, cols=nc, weight=w, density=w / n, s_per_iter=s, s_per_iter_per_nz=s / w,
                         intervals=len(per) - 1, first_interval_s_per_iter=per[0][0], last_interval_s_per_iter=per[-1][0],
                         power_w_mean=stat(h, "power_all_w", "mean"), cpu_temp_c_max=stat(h, "cpu_temp_c", "max"),
                         cpu_active_mean=stat(h, "cpu_active_ratio", "mean"), memory_used_ratio_max=stat(h, "memory_used_ratio", "max"),
                         clock_p_mhz=p_mhz, clock_s_mhz=s_mhz))
    with open(out + ".csv", "w", newline="") as fh:
        wr = csv.DictWriter(fh, fieldnames=list(rows[0].keys()))
        wr.writeheader()
        for r in rows:
            wr.writerow({kk: (f"{v:.6g}" if isinstance(v, float) else v) for kk, v in r.items()})

    rho = t["real130"] / t["syn130"]
    gate = abs(t["syn130"] / t["real130"] - 1) <= GATE
    md = ["# H6: block Wiedemann at 200 digits on this laptop\n",
          "Generated by `scripts/h6_summary.py` from `scripts/h6_linalg.sh`. The prediction and criterion are"
          " `docs/h6_falsifiable_prediction.md`. bwc `krylov` with m = n = 64, thr=3x6, interval 500, after `prep`;"
          " per-iteration times are medians over the checkpoint intervals after the first.\n",
          "## The matrix at 200 digits\n",
          f"- Rerunning `merge` on RSA-130's purged relations at density 170 gives {n170:,} rows and weight {w170:,}"
          f" ({w170 / n170:.1f} per row), against {n110:,} rows and {w110:,} at 110: κ = {k:.4f}.",
          f"- The rows divided by ΣL in H4's matrices: {', '.join(f'{x:.4f}' for x in ratios)}. With RSA-130's ratio,"
          f" ΣL_200 = {sigma_l(200):.4g} and κ: **N_200 = {N:,}** rows, weight {170 * N:.3g} at density 170"
          f" ({n_lo:,} to {n_hi:,} over H4's ratios).",
          f"- Cost N × W against RSA-130 at density 110: at 170 the merged RSA-130 matrix has {n170 * w170 / (n110 * w110):.3f} of it.\n",
          "## Time per iteration\n",
          "| matrix | rows | weight | per row | s per iteration | ns per nonzero | intervals | first interval | last interval | mean W | max CPU °C |",
          "|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|"]
    names = {"real130": "RSA-130, real", "syn130": "synthetic, RSA-130's shape", "synq4": "synthetic, N_200/4",
             "synq2": "synthetic, N_200/2", "syn200": "synthetic, N_200"}
    for r in rows:
        md.append(f"| {names[r['tag']]} | {r['rows']:,} | {r['weight']:.3g} | {r['density']:.1f} | {r['s_per_iter']:.4f} |"
                  f" {1e9 * r['s_per_iter_per_nz']:.3f} | {r['intervals']} | {r['first_interval_s_per_iter']:.4f} |"
                  f" {r['last_interval_s_per_iter']:.4f} | {r['power_w_mean']:.1f} | {r['cpu_temp_c_max']:.1f} |")
    md.append(f"\n- Calibration: the synthetic matrix of RSA-130's shape takes {t['syn130']:.4f} s per iteration against the real"
              f" one's {t['real130']:.4f}: ρ = {rho:.3f}. The gate is ±{100 * GATE:.0f}%: "
              + ("**passed**." if gate else "**failed**, so the synthetic matrix does not stand in."))
    md.append(f"- H4 measured RSA-130's real matrix at {H4_LA_130['krylov'] / H4_LA_130['iters']:.4f} s per iteration over its whole"
              f" Krylov run, {1e9 * H4_LA_130['krylov'] / H4_LA_130['iters'] / H4_LA_130['weight']:.3f} ns per nonzero.")
    tables = {r["tag"]: interval_table(r["tag"]) for r in rows}
    clean = {tag: clean_median(tb) for tag, tb in tables.items()}
    md.append("\n## Other work on the machine\n")
    md.append(f"The machine was not reserved. An interval counts as clean when the GPU, which bwc does not use, was under"
              f" {100 * GPU_CLEAN:.0f}% active. Per interval: `data/results/h6/<tag>.intervals.csv`.\n")
    md.append("| matrix | intervals after the first | clean | median, all | median, clean |")
    md.append("|---|---:|---:|---:|---:|")
    for r in rows:
        cm, cn = clean[r["tag"]]
        md.append(f"| {names[r['tag']]} | {len(tables[r['tag']]) - 1} | {cn} | {r['s_per_iter']:.4f} | "
                  + (f"{cm:.4f} |" if cm == cm else "n/a |"))
    if "syn200" in t and gate and not (R / "lingen.txt").exists():
        md.append("\nLingen has not finished: no total yet.")
    if "syn200" in t and gate and (R / "lingen.txt").exists():
        t200 = t["syn200"] * rho
        iters = 3 * N / 64
        L, t_lg = lingen()
        mv = iters * t200
        T = (mv + t_lg) * OVERHEAD / 3600
        verdict = "holds" if LOW <= T <= HIGH else "rejected"
        md.append("\n## The linear algebra at 200 digits\n")
        md.append(f"- Each iteration on the 200-digit matrix: {t['syn200']:.4f} s × ρ = **{t200:.4f} s**"
                  f" ({1e9 * t200 / (170 * N):.3f} ns per nonzero, against {1e9 * t['real130'] / rows[0]['weight']:.3f} at RSA-130).")
        md.append(f"- Iterations: 3 N_200 / 64 = {iters:,.0f} (Krylov {2 * N / 64:,.0f}, mksol {N / 64:,.0f}): {mv / 3600:.1f} hours.")
        md.append(f"- Lingen, thr=1x1, on random input of length {L:,}: {t_lg:,.0f} s = {t_lg / 3600:.2f} hours.")
        md.append(f"- Overhead × {OVERHEAD:.3f} (RSA-130's linear algebra phase over its three steps).")
        md.append(f"- **T_LA = {T:.1f} laptop-hours.**")
        md.append(f"- **H6 {verdict}**: T_LA " + ("lies" if verdict == "holds" else "does not lie")
                  + f" in [{LOW:.0f}, {HIGH:.0f}], a factor of 2 around {CLAIM:.0f}.")
        lo = (mv * (n_lo / N) ** 2 + t_lg * n_lo / N) * OVERHEAD / 3600
        hi = (mv * (n_hi / N) ** 2 + t_lg * n_hi / N) * OVERHEAD / 3600
        md.append(f"- Over H4's range of ratios ({n_lo:,} to {n_hi:,} rows), with the products scaling as N² and lingen as N:"
                  f" {lo:.1f} to {hi:.1f} laptop-hours.")
        md.append(f"- Against H1's 3,896 laptop-hours of relation collection, the linear algebra is {100 * T / (H1_RELATIONS_HOURS + T):.1f}%"
                  f" of the two together. A relation collection that cost nothing would leave at least {T:.0f} laptop-hours"
                  f" (polynomial selection not counted): at most {(H1_RELATIONS_HOURS + T) / T:.0f} times faster.")
        cr, cs, c2 = clean["real130"][0], clean["syn130"][0], clean["syn200"][0]
        if cr == cr and cs == cs and c2 == c2:
            rho_c = cr / cs
            T_c = (iters * c2 * rho_c + t_lg) * OVERHEAD / 3600
            md.append(f"- With clean intervals only (not the protocol, reported beside it): ρ = {rho_c:.3f},"
                      f" t_200 = {c2:.4f} × ρ = {c2 * rho_c:.4f} s, T_LA = **{T_c:.1f} laptop-hours**"
                      + (", also inside the band." if LOW <= T_c <= HIGH else ", outside the band."))
    Path(out + ".md").write_text("\n".join(md) + "\n")
    print(f"κ = {k:.4f}; N_200 = {N:,}; ρ = {rho:.3f} (gate {'passed' if gate else 'failed'})")
    if "syn200" in t and gate and (R / "lingen.txt").exists():
        print(f"t_200 = {t['syn200'] * rho:.4f} s/iter; T_LA = {T:.1f} laptop-hours; H6 {verdict}")
    print(f"wrote {out}.md and {out}.csv")


if __name__ == "__main__":
    main(sys.argv)
