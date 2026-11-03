#!/usr/bin/env python3
"""Summarize every d1-measurement/2 result under data/results and apply the
D1 rejection criterion from docs/d1_falsifiable_prediction.md.

    scripts/d1_summary.py [--suffix scaled-down] [--out data/results/summary]

Reads data/results/d1-*.<suffix>.json, writes <out>.csv (one row per
instance, bound and side) and <out>.md (the criterion applied per bound to
the algebraic gap after alpha). The noise is the Poisson noise of the
counts, sqrt(sum of squared standard errors) / n, and, beside it, the
spread of the gaps across instances, std / sqrt(n). Standard library only.
"""
import csv
import glob
import json
import math
import statistics
import sys
from pathlib import Path


def main(argv):
    suffix, out = "scaled-down", "data/results/summary"
    i = 1
    while i < len(argv):
        if argv[i] == "--suffix":
            suffix = argv[i + 1]; i += 2
        elif argv[i] == "--out":
            out = argv[i + 1]; i += 2
        else:
            sys.exit("usage: d1_summary.py [--suffix S] [--out PATH]")
    files = sorted(glob.glob(f"data/results/d1-*.{suffix}.json"))
    results = []
    for f in files:
        r = json.loads(Path(f).read_text())
        if r.get("schema_version") != "d1-measurement/2":
            continue
        results.append((Path(f).name.split(".")[0], r))
    if not results:
        sys.exit("no d1-measurement/2 results found")
    rows = []
    for inst, r in results:
        for b in r["results"]:
            for side in ("algebraic", "rational"):
                s = b[side]
                rows.append({
                    "instance": inst, "digits": r["n_digits"], "b": b["b"], "side": side,
                    "evaluated": r["evaluated"], "smooth": s["smooth"], "observed": s["observed"],
                    "observed_stderr": s["observed_stderr"], "predicted": s["predicted"], "gap": s["gap"],
                    "predicted_alpha": s["predicted_alpha"], "gap_alpha": s["gap_alpha"],
                    "gap_alpha_stderr_units": s["gap_alpha"] / s["observed_stderr"] if s["observed_stderr"] else 0.0,
                    "predicted_local": s["predicted_local"], "gap_local": s["gap_local"],
                    "mean_u": s["mean_u"], "alpha": s["alpha"], "implied_alpha": s.get("implied_alpha", ""),
                    "method": r.get("smoothness", {}).get("method", ""), "elapsed_ms": r["elapsed_ms"],
                })
    out_csv = Path(out + ".csv")
    out_csv.parent.mkdir(parents=True, exist_ok=True)
    with out_csv.open("w", newline="") as fh:
        w = csv.DictWriter(fh, fieldnames=list(rows[0].keys()))
        w.writeheader(); w.writerows(rows)
    L = []
    L.append(f"# D1 summary across instances ({suffix})")
    L.append("")
    L.append(f"{len(results)} instances: {', '.join(i for i, _ in results)}. Per instance, bound and side the rows are in `{out_csv}`. Every prediction is the α-adjusted Dickman mean with α computed in the harness (`docs/murphy_alpha.md`).")
    L.append("")
    L.append("## Rejection criterion, algebraic side")
    L.append("")
    L.append("The claim is rejected at a bound when the mean gap (observed − α-adjusted prediction) over the instances is smaller than two standard deviations of the measurement noise. Two noise estimates are shown: the Poisson noise of the counts, and the spread of the gaps across instances.")
    L.append("")
    L.append("| B | instances | mean gap | Poisson noise | gap / Poisson noise | spread across instances (std/√n) | gap / spread | mean relative gap | verdict |")
    L.append("|---|---|---|---|---|---|---|---|---|")
    bounds = sorted({row["b"] for row in rows})
    for b in bounds:
        rs = [row for row in rows if row["b"] == b and row["side"] == "algebraic"]
        n = len(rs)
        gaps = [row["gap_alpha"] for row in rs]
        mean = statistics.mean(gaps)
        poisson = math.sqrt(sum(row["observed_stderr"] ** 2 for row in rs)) / n
        spread = statistics.stdev(gaps) / math.sqrt(n) if n > 1 else float("nan")
        rel = statistics.mean(row["gap_alpha"] / row["predicted_alpha"] for row in rs if row["predicted_alpha"])
        verdict = "rejected (within 2σ of Poisson noise)" if abs(mean) < 2 * poisson else ("not rejected by Poisson noise; " + ("within 2σ of the instance spread" if not math.isnan(spread) and abs(mean) < 2 * spread else "outside 2σ of the instance spread"))
        L.append(f"| 2^{int(b).bit_length()-1} | {n} | {mean:+.6f} | {poisson:.6f} | {mean/poisson if poisson else 0:+.1f} | {spread:.6f} | {mean/spread if spread else 0:+.1f} | {rel*100:+.2f}% | {verdict} |")
    L.append("")
    L.append("## Per instance, algebraic side at the largest bound")
    L.append("")
    top = max(bounds)
    L.append("| instance | digits | values | observed | α-adjusted | gap / se | α | implied α | method | seconds |")
    L.append("|---|---|---|---|---|---|---|---|---|---|")
    for row in rows:
        if row["b"] == top and row["side"] == "algebraic":
            L.append(f"| {row['instance']} | {row['digits']} | {row['evaluated']:,} | {row['observed']:.5f} | {row['predicted_alpha']:.5f} | {row['gap_alpha_stderr_units']:+.1f} | {row['alpha']:+.3f} | {row['implied_alpha'] if row['implied_alpha'] == '' else f'{float(row['implied_alpha']):+.2f}'} | {row['method']} | {row['elapsed_ms']/1000:.1f} |")
    L.append("")
    L.append("## Rational side at the largest bound")
    L.append("")
    L.append("| instance | observed | α-adjusted | gap / se | local density | local gap / se |")
    L.append("|---|---|---|---|---|---|")
    for row in rows:
        if row["b"] == top and row["side"] == "rational":
            L.append(f"| {row['instance']} | {row['observed']:.5f} | {row['predicted_alpha']:.5f} | {row['gap_alpha_stderr_units']:+.0f} | {row['predicted_local']:.5f} | {row['gap_local']/row['observed_stderr'] if row['observed_stderr'] else 0:+.0f} |")
    Path(out + ".md").write_text("\n".join(L) + "\n")
    print(f"wrote {out_csv} ({len(rows)} rows) and {out}.md")


if __name__ == "__main__":
    main(sys.argv)
