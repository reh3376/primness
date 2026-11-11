#!/usr/bin/env python3
"""H5: the sieve proper's share at the laptop's one-hour reach (docs/h5_falsifiable_prediction.md).

    scripts/h5_summary.py [--dir data/results/h5] [--out data/results/h5_sieve_share]

Reads the nine parts' `las -T` statistics (<dir>/las.<k>.txt), the special-q count of every
workunit of H4's RSA-130 run (<dir>/run_special_q.csv), H4's phases (data/results/h4_reach.csv)
and, for the comparison at 200 digits, H1's statistics (data/results/h1/las.<q>.txt) and its
integral (data/results/h1_cost.md). Per part: the categorized times per special-q, weighted
by the run's special-q's in that part. Then the sieve proper's share of the counted time, the
share s of RSA-130's wall time, the ceiling 1 / (1 - s) and the criterion. Writes <out>.csv
and <out>.md. Standard library only.
"""
import csv
import math
import re
import sys
from pathlib import Path

Q0, Q1, PARTS = 750000, 4330000, 9
GROUPS = {
    "norms": ("4", "260"),
    "sieving": ("2", "3", "258"),
    "survivor search": ("1",),
    "cofactorization": ("5", "6", "261"),
    "bookkeeping": ("0", "-1"),
}
SIEVE_PROPER = ("norms", "sieving", "survivor search")
LIMIT = 2 / 3          # H5 holds if s < 2/3 (a ceiling below 3x)
RECORDS_REL_SHARE = (0.83, 0.91)  # relation collection's share of the records' total (H1)
H1_RELATIONS = 461_027_928       # CADO-NFS's target at 200 digits (H1)


def parse_las(path):
    text = Path(path).read_text()
    out = {"path": str(path)}
    m = re.search(r"# Total (\d+) reports \[[^\]]*\] in ([\d.]+) elapsed s \[([\d.]+)% CPU\]", text)
    if not m:
        sys.exit(f"{path}: no closing statistics (did las finish?)")
    out["relations"], out["elapsed"], out["cpu_pct"] = int(m.group(1)), float(m.group(2)), float(m.group(3))
    nq = re.findall(r"out of (\d+) pushed", text)
    out["special_q"] = int(nq[-1])
    cats = {}
    block = text.split("# Categorized timings", 1)
    if len(block) == 2:
        for line in block[1].splitlines()[1:]:
            mm = re.match(r"# (-?\d+) [^:]*: ([\d.]+)", line)
            if mm:
                cats[mm.group(1)] = float(mm.group(2))
            elif line.startswith("# total counted time"):
                break
    if not cats:
        sys.exit(f"{path}: no categorized timings (was -T given?)")
    out["groups"] = {g: sum(cats.get(c, 0.0) for c in ids) for g, ids in GROUPS.items()}
    out["counted"] = sum(cats.values())
    out["survivors"] = {k: int(v) for k, v in re.findall(r"# survivors (\S+): (\d+)", text)}
    return out


def run_weights(path):
    """Special-q's the H4 run sieved in each of the nine parts, from its workunit outputs."""
    step = (Q1 - Q0) // PARTS
    w = [0] * PARTS
    with open(path) as fh:
        for row in csv.DictReader(fh):
            k = min((int(row["q0"]) - Q0) // step, PARTS - 1)
            w[k] += int(row["special_q"])
    return w


def h4_rsa130():
    with open("data/results/h4_reach.csv") as fh:
        for row in csv.DictReader(fh):
            if row["digits"] == "130":
                return float(row["sieving_s"]), float(row["wall_s"]), int(row["relations"])
    sys.exit("data/results/h4_reach.csv: no RSA-130 row")


def main(argv):
    d, out = Path("data/results/h5"), "data/results/h5_sieve_share"
    i = 1
    while i < len(argv):
        if argv[i] == "--dir":
            d = Path(argv[i + 1]); i += 2
        elif argv[i] == "--out":
            out = argv[i + 1]; i += 2
        else:
            sys.exit("usage: h5_summary.py [--dir DIR] [--out PATH]")
    parts = [parse_las(d / f"las.{k}.txt") for k in range(1, PARTS + 1)]
    weights = run_weights(d / "run_special_q.csv")
    sieving_s, wall_s, rels130 = h4_rsa130()
    phase_share = sieving_s / wall_s

    # the run's estimated totals per group (core-seconds) and its sieve positions
    totals = {g: 0.0 for g in GROUPS}
    positions = cofactored = after = 0.0
    for p, w in zip(parts, weights):
        scale = w / p["special_q"]
        for g in GROUPS:
            totals[g] += p["groups"][g] * scale
        positions += p["survivors"].get("before_sieve", 0) * scale
        after += p["survivors"].get("after_sieve", 0) * scale
        cofactored += p["survivors"].get("enter_cofactoring", 0) * scale
    counted = sum(totals.values())
    share = {g: totals[g] / counted for g in GROUPS}
    p_sieve = sum(share[g] for g in SIEVE_PROPER)
    s = p_sieve * phase_share
    ceiling = 1 / (1 - s)
    ceiling_rel = 1 / (1 - phase_share)
    verdict = "holds" if s < LIMIT else "rejected"
    run_q = sum(weights)

    # 200 digits, from H1's las statistics and integral
    h1 = []
    for f in sorted(Path("data/results/h1").glob("las.*.txt"), key=lambda p: int(p.stem.split(".")[1])):
        r = parse_las(f)
        r["q"] = int(f.stem.split(".")[1])
        h1.append(r)
    p200 = [sum(r["groups"][g] for g in SIEVE_PROPER) / r["counted"] for r in h1]
    m = re.search(r"after ([\d,]+) special-q's", Path("data/results/h1_cost.md").read_text())
    h1_q = int(m.group(1).replace(",", "")) if m else math.nan
    pos200_per_q = sum(r["survivors"].get("before_sieve", 0) for r in h1) / sum(r["special_q"] for r in h1)
    ns200 = 1e9 * sum(r["counted"] for r in h1) / sum(r["survivors"].get("before_sieve", 0) for r in h1)
    ns130 = 1e9 * counted / positions
    pos200 = h1_q * pos200_per_q

    rows = []
    for k, (p, w) in enumerate(zip(parts, weights), 1):
        sv = p["survivors"]
        rows.append(dict(
            part=k, q0=Q0 + (k - 1) * ((Q1 - Q0) // PARTS), run_special_q=w, sampled_special_q=p["special_q"],
            relations_per_q=p["relations"] / p["special_q"], counted_s_per_q=p["counted"] / p["special_q"],
            **{f"share_{g.replace(' ', '_')}": p["groups"][g] / p["counted"] for g in GROUPS},
            positions_per_q=sv.get("before_sieve", 0) / p["special_q"],
            after_sieve_per_q=sv.get("after_sieve", 0) / p["special_q"],
            cofactored_per_q=sv.get("enter_cofactoring", 0) / p["special_q"],
            elapsed_s=p["elapsed"], cpu_pct=p["cpu_pct"]))
    with open(out + ".csv", "w", newline="") as fh:
        wr = csv.DictWriter(fh, fieldnames=list(rows[0].keys()))
        wr.writeheader()
        for r in rows:
            wr.writerow({k: (f"{v:.6g}" if isinstance(v, float) else v) for k, v in r.items()})

    md = ["# H5: the sieve proper's share at the laptop's one-hour reach\n",
          "Generated by `scripts/h5_summary.py` from `scripts/h5_sieve_share.sh`. The prediction and criterion are"
          " `docs/h5_falsifiable_prediction.md`. RSA-130 as H4 ran it: the polynomial and side-1 factor base of that run"
          " (hashes in `data/results/h5/inputs.sha256`) and its client parameters; nine `las -t 2 -T` processes at once,"
          " one per ninth of [7.5e5, 4.33e6), 300 random special-q's each.\n",
          "## Per part\n",
          "| part | q from | run's special-q's | relations per q | counted core-s per q | sieve proper | cofactorization | bookkeeping |",
          "|---:|---:|---:|---:|---:|---:|---:|---:|"]
    for r in rows:
        sp = r["share_norms"] + r["share_sieving"] + r["share_survivor_search"]
        md.append(f"| {r['part']} | {r['q0']:,} | {r['run_special_q']:,} | {r['relations_per_q']:.1f} | {r['counted_s_per_q']:.3f} |"
                  f" {100 * sp:.1f}% | {100 * r['share_cofactorization']:.1f}% | {100 * r['share_bookkeeping']:.1f}% |")
    md.append(f"\nThe run sieved {run_q:,} special-q's (counted from its workunit outputs).\n")
    md.append("## The run's counted time, weighted\n")
    md.append("| group | share of `las`'s counted time |")
    md.append("|---|---:|")
    for g in GROUPS:
        md.append(f"| {g} | {100 * share[g]:.1f}% |")
    md.append(f"| **sieve proper** (norms, sieving, survivor search) | **{100 * p_sieve:.1f}%** |")
    md.append("\n## The statistic and the criterion\n")
    md.append(f"- The sieving phase is {100 * phase_share:.1f}% of RSA-130's wall time in H4 ({sieving_s:,.0f} of {wall_s:,.0f} s).")
    md.append(f"- s = {p_sieve:.3f} × {phase_share:.3f} = **{s:.3f}**: the sieve proper's share of the complete factorization.")
    md.append(f"- A sieve that cost nothing and found the same relations: **{ceiling:.2f} times** faster at most.")
    md.append(f"- **H5 {verdict}**: s " + ("<" if verdict == "holds" else "≥") + f" 2/3 (a ceiling of 3 times).")
    md.append(f"- A relation collection that cost nothing at all (sieve and cofactorization): {ceiling_rel:.2f} times at most.")
    md.append("\n## At 200 digits, from H1\n")
    lo, hi = min(p200), max(p200)
    a, b = RECORDS_REL_SHARE
    md.append(f"- The sieve proper is {100 * lo:.0f} to {100 * hi:.0f}% of `las`'s counted time over H1's six ranges.")
    md.append(f"- With relation collection at {100 * a:.0f} to {100 * b:.0f}% of the total (the records' share),"
              f" a free sieve makes the factorization {1 / (1 - lo * a):.1f} to {1 / (1 - hi * b):.1f} times faster at most,"
              f" and a free relation collection {1 / (1 - a):.1f} to {1 / (1 - b):.1f} times.")
    md.append("\n## Where the growth from RSA-130 to 200 digits comes from (H1 and H4 data, not part of the verdict)\n")
    md.append("| | RSA-130 (H4, H5) | 200 digits (H1) | ratio |")
    md.append("|---|---:|---:|---:|")
    md.append(f"| relations needed | {rels130:,} | {H1_RELATIONS:,} | {H1_RELATIONS / rels130:,.1f} |")
    md.append(f"| sieve positions | {positions:.3g} | {pos200:.3g} | {pos200 / positions:,.0f} |")
    md.append(f"| positions per relation | {positions / rels130:.3g} | {pos200 / H1_RELATIONS:.3g} | {(pos200 / H1_RELATIONS) / (positions / rels130):,.0f} |")
    md.append(f"| `las` core-ns per position | {ns130:.1f} | {ns200:.1f} | {ns200 / ns130:.2f} |")
    md.append(f"| `las` core-seconds in all | {positions * ns130 / 1e9:.3g} | {pos200 * ns200 / 1e9:.3g} | {(pos200 * ns200) / (positions * ns130):,.0f} |")
    md.append("\nThe cost per sieve position barely moves between the two sizes. Nearly all of the growth is in the number of"
              " positions, which is the number of relations needed times how rarely a position yields one.")
    Path(out + ".md").write_text("\n".join(md) + "\n")
    print(f"sieve proper {100 * p_sieve:.1f}% of counted time; s = {s:.3f}; ceiling {ceiling:.2f}x; H5 {verdict}")
    print(f"wrote {out}.md and {out}.csv")


if __name__ == "__main__":
    main(sys.argv)
