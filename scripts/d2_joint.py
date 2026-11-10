#!/usr/bin/env python3
"""D2 part 1: are values that the NFS combines smooth together more often than apart?

    scripts/d2_joint.py [--only d1-01,d1-02] [--out data/results/d2_multiple_fields]

docs/d2_falsifiable_prediction.md fixes the protocol and the rejection
criterion before measuring; docs/d2_multiple_fields.md records the result.

Per instance X the columns are its instrument polynomial F_0, the three
rotated fields F_1..F_3 of data/results/d2/fields.csv and the rational side
G, and the same five of the next instance Y (d1-10 is followed by d1-01).
Every column's smooth probability per bound and cell of ln c is the one D5
uses (scripts/d5_clusters.py prediction_grid: the conditional baseline plus
the first-order term of that polynomial's own root counts). The script
writes one manifest per instance under data/dumps/d2/joint/, runs d2-joint on
it (the per-value sums, data/results/d2/joint/<X>.csv), and combines the
instances:

    cross      (F_i, G) of X, i = 0..3        control (F_i of X, G of Y)
    algebraic  (F_i, F_j) of X, i < j         control (F_i of X, F_j of Y)

R = O/E - 1 per class, its error sqrt(V)/E, the mean over instances weighted
by 1/error^2 with its error widened by sqrt(chi^2/9) when that exceeds 1, and
the criterion. Writes <out>.csv and <out>.md. Standard library only.
"""
import csv
import json
import math
import subprocess
import sys
from collections import defaultdict
from pathlib import Path

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent))
import d1_conditional as d1  # noqa: E402
import d5_clusters as d5  # noqa: E402

PROTOCOL = (1 << 20, 1 << 24, 1 << 28)
CLASSES = ("cross", "algebraic")
JOINT = "target/release/d2-joint"


def chosen_fields(path="data/results/d2/fields.csv"):
    """instance -> [field ids by rank]."""
    out = defaultdict(dict)
    with open(path) as fh:
        for row in csv.DictReader(fh):
            if row["rank"]:
                out[row["instance"]][int(row["rank"])] = row["field"]
    return {k: [v[r] for r in sorted(v)] for k, v in out.items()}


class Grids:
    """Smooth probability per bound and cell for one side of one polynomial."""

    def __init__(self):
        self.base, self.q = d1.load_baseline("data/results/baseline.consecutive.json")
        self.rough = self.base["cofactor"]["rough_bound"]
        self.width = self.base["cofactor"]["bin_width"]
        self.tails, cover = d1.load_root_tail("data/results/root_tail.json", self.rough)
        field_tails, field_cover = d1.load_root_tail("data/results/d2/root_tail.json", self.rough)
        if not self.tails or not field_tails:
            sys.exit("root tails missing: run d1-tail for the instruments and for data/instruments/d2")
        self.tails.update(field_tails)
        self.omega = d1.buchstab_omega()
        self.bounds = sorted(b for b in self.q if b <= min(cover, field_cover))
        self.top = (max(max(qb) for qb in self.q.values()) + 2) * self.width
        self.cache = {}

    def hist(self, poly):
        """(side, b) -> bins, and the region, from poly's d1-measurement/4 results."""
        files = sorted(Path("data/results").glob(f"{poly}.scaled-down*.json")) if "." not in poly else [Path(f"data/results/d2/{poly}.json")]
        hist, region = {}, None
        for f in files:
            result, h = d1.load_instance(f)
            if result is None:
                continue
            region = region or result["region"]
            for key, bins in h.items():
                hist.setdefault(key, bins)
        if region is None:
            sys.exit(f"{poly}: no d1-measurement/4 result")
        return hist, region

    def get(self, poly, side):
        key = (poly, side)
        if key not in self.cache:
            hist, region = self.hist(poly)
            rows = []
            for b in self.bounds:
                bins = hist.get((side, b))
                if bins is None:
                    sys.exit(f"{poly} {side}: no cofactor histogram at {b}")
                s_by_bin = d1.first_order_by_bin(self.q[b], bins, self.tails[(poly, side)], b, self.width, self.rough,
                                                 self.omega, region["a_max"], region["b_max"])
                ps, _ = d5.prediction_grid(self.q[b], b, self.width, s_by_bin, self.top)
                rows.append([None if math.isnan(p) else p for p in ps])
            self.cache[key] = rows
        return self.cache[key]


def columns_of(inst, fields):
    """The five columns of one instance: F_0, F_1..F_3, G, as (name, poly, side, dump)."""
    out = [(inst, inst, "algebraic", f"data/dumps/{inst}.values.csv")]
    out += [(f, f, "algebraic", f"data/dumps/d2/{f}.values.csv") for f in fields[inst]]
    out.append((f"{inst}/rational", inst, "rational", f"data/dumps/{inst}.values.csv"))
    return out


def run_job(x, y, fields, grids):
    cols = columns_of(x, fields) + columns_of(y, fields)
    manifest = dict(
        schema_version="d2-joint-manifest/1",
        bounds=grids.bounds,
        cofactor_bin_width=grids.width,
        columns=[dict(name=n, dump=d, side=s, pi=grids.get(p, s)) for n, p, s, d in cols],
        classes=[
            dict(name="cross", pairs=[[i, 4] for i in range(4)]),
            dict(name="algebraic", pairs=[[i, j] for i in range(4) for j in range(i + 1, 4)]),
            dict(name="cross control", pairs=[[i, 9] for i in range(4)]),
            dict(name="algebraic control", pairs=[[i, 5 + j] for i in range(4) for j in range(i + 1, 4)]),
        ],
        unions=[dict(name="cross", rational=4, fields=[0, 1, 2, 3])],
    )
    for _, _, _, d in cols:
        if not Path(d).exists():
            sys.exit(f"{d} is missing: run scripts/d2_run_all.sh")
    mpath = Path(f"data/dumps/d2/joint/{x}.manifest.json")
    mpath.parent.mkdir(parents=True, exist_ok=True)
    mpath.write_text(json.dumps(manifest))
    out = Path(f"data/results/d2/joint/{x}.csv")
    out.parent.mkdir(parents=True, exist_ok=True)
    subprocess.run([JOINT, "--manifest", str(mpath), "--out", str(out)], check=True)
    return out


def read_sums(path):
    with open(path) as fh:
        return list(csv.DictReader(fh))


def ratio(o, e, v):
    if e <= 0:
        return math.nan, math.nan
    return o / e - 1.0, math.sqrt(v) / e


def combine(rs, ses):
    """Weighted mean, its error widened by the Birge ratio, chi^2 and the count."""
    use = [(r, s) for r, s in zip(rs, ses) if not math.isnan(r) and s > 0]
    if not use:
        return dict(mean=math.nan, err=math.nan, chi2=math.nan, n=0)
    w = [1 / s ** 2 for _, s in use]
    m = sum(wi * r for wi, (r, _) in zip(w, use)) / sum(w)
    se = 1 / math.sqrt(sum(w))
    chi2 = sum(wi * (r - m) ** 2 for wi, (r, _) in zip(w, use))
    n = len(use)
    birge = math.sqrt(chi2 / (n - 1)) if n > 1 else 1.0
    return dict(mean=m, err=se * max(1.0, birge), chi2=chi2, n=n)


def main(argv):
    only, out = None, "data/results/d2_multiple_fields"
    i = 1
    while i < len(argv):
        if argv[i] == "--only":
            only = set(argv[i + 1].split(",")); i += 2
        elif argv[i] == "--out":
            out = argv[i + 1]; i += 2
        else:
            sys.exit("usage: d2_joint.py [--only IDS] [--out PATH]")
    fields = chosen_fields()
    instances = sorted(fields)
    grids = Grids()
    rows, columns, unions = [], [], []
    for k, x in enumerate(instances):
        y = instances[(k + 1) % len(instances)]
        if only and x not in only:
            continue
        path = run_job(x, y, fields, grids)
        print(f"{x} (control {y}): {path}", flush=True)
        for r in read_sums(path):
            b = int(r["b"])
            o, e, v = int(r["observed"]), float(r["expected"]), float(r["variance"])
            if r["kind"] == "column":
                cert = int(r["certain"])
                unc = e - cert
                kap = float(r["kappa"])
                columns.append(dict(instance=x, column=r["name"], b=b, observed=o, predicted=e, kappa=kap,
                                    kappa_err=math.sqrt(v) / unc if unc > 0 else math.nan))
            elif r["kind"] in ("class", "pair"):
                rr, se = ratio(o, e, v)
                rows.append(dict(instance=x, kind=r["kind"], cls=r["name"], s=r["s"], t=r["t"], b=b,
                                 observed=o, expected=e, ratio=rr, err=se))
            elif r["kind"] == "union":
                unions.append(dict(instance=x, b=b, observed=o, expected=e))
    if not rows:
        sys.exit("nothing measured")
    check_against_d5(columns)
    with open(out + ".csv", "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=list(rows[0].keys()))
        w.writeheader()
        for r in rows:
            w.writerow({k: (f"{v:.6g}" if isinstance(v, float) else v) for k, v in r.items()})
    write_summary(rows, columns, unions, fields, grids.bounds, out)
    print(f"wrote {out}.csv ({len(rows)} rows) and {out}.md")


def check_against_d5(columns):
    """The instruments' predictions summed here and by D5 must agree."""
    path = Path("data/results/d5_clusters.csv")
    if not path.exists():
        return
    d5_tot = {}
    with open(path) as fh:
        for r in csv.DictReader(fh):
            if r["partition"] == "64 sectors":
                d5_tot[(r["instance"], r["side"], int(r["b"]))] = float(r["predicted"])
    worst = 0.0
    for c in columns:
        if c["column"] != c["instance"] and c["column"] != f"{c['instance']}/rational":
            continue
        side = "rational" if c["column"].endswith("/rational") else "algebraic"
        ref = d5_tot.get((c["instance"], side, c["b"]))
        if ref:
            worst = max(worst, abs(c["predicted"] / ref - 1))
    print(f"instrument predictions against D5's (six significant digits): largest relative difference {worst:.1e}")
    if worst > 2e-5:
        sys.exit("the per-value predictions differ from D5's")


def label(b):
    return f"2^{b.bit_length() - 1}"


def pct(x, e=None):
    return f"{100 * x:+.2f}%" if e is None else f"{100 * x:+.2f}% ± {100 * e:.2f}%"


def write_summary(rows, columns, unions, fields, bounds, out):
    instances = sorted({r["instance"] for r in rows})
    agg = {}
    for cls in ("cross", "algebraic", "cross control", "algebraic control"):
        for b in bounds:
            rs = [r for r in rows if r["kind"] == "class" and r["cls"] == cls and r["b"] == b]
            agg[(cls, b)] = combine([r["ratio"] for r in rs], [r["err"] for r in rs])
    L = ["# D2 part 1: several number fields sharing the rational side", ""]
    L.append(f"{len(instances)} instances, the D1 region; per instance the instrument polynomial F_0 and three rotations f + v·g (`data/results/d2/fields.csv`). "
             "Every value's smooth probability is D5's (conditional baseline plus the polynomial's own first-order root-count term), each column scaled by its own κ = O/P. "
             "R = O/E − 1 for the values at which both members of a pair are smooth, summed over the pairs of a class. "
             f"The protocol and the criterion are `docs/d2_falsifiable_prediction.md`, the reading `docs/d2_multiple_fields.md`; per-instance rows are in `{out}.csv`.")
    L.append("")
    L.append("## Joint smoothness against independence")
    L.append("")
    L.append("Mean of R over the instances weighted by their noise, ± its error (widened where the instances scatter beyond their noise); χ² about the mean on the instances used.")
    L.append("")
    L.append("| class | " + " | ".join(label(b) for b in bounds) + " |")
    L.append("|---|" + "---:|" * len(bounds))
    for cls in ("cross", "cross control", "algebraic", "algebraic control"):
        L.append(f"| {cls} | " + " | ".join(f"{pct(agg[(cls, b)]['mean'], agg[(cls, b)]['err'])} (χ² {agg[(cls, b)]['chi2']:.1f}/{agg[(cls, b)]['n']})" for b in bounds) + " |")
    L.append("")
    std = {}
    for b in bounds:
        rs = [r for r in rows if r["kind"] == "pair" and r["cls"] == "cross" and r["s"] == r["instance"] and r["b"] == b]
        std[b] = combine([r["ratio"] for r in rs], [r["err"] for r in rs])
    L.append("**The standard pair** (F_0, G), the two sides of the ordinary NFS: " + ", ".join(f"{pct(std[b]['mean'], std[b]['err'])} at {label(b)}" for b in bounds) + ".")
    L.append("")
    L.append("**Per instance**, R of each class ± its error at the protocol bounds:")
    L.append("")
    L.append("| instance | class | " + " | ".join(label(b) for b in PROTOCOL) + " |")
    L.append("|---|---|" + "---:|" * len(PROTOCOL))
    for x in instances:
        for cls in ("cross", "algebraic"):
            cells = []
            for b in PROTOCOL:
                r = next(r for r in rows if r["kind"] == "class" and r["cls"] == cls and r["b"] == b and r["instance"] == x)
                cells.append(pct(r["ratio"], r["err"]))
            L.append(f"| {x} | {cls} | " + " | ".join(cells) + " |")
    L.append("")
    L.append("## The fields")
    L.append("")
    L.append("κ − 1 (observed over predicted smooth values, over the values not smooth for certain), the D1 check on the rotated fields: weighted mean over the thirty ± its error, and the range.")
    L.append("")
    L.append("| columns | " + " | ".join(label(b) for b in bounds) + " |")
    L.append("|---|" + "---:|" * len(bounds))
    groups = (("instruments F_0", lambda c: c["column"] == c["instance"]),
              ("rotated fields", lambda c: "." in c["column"] and c["column"].split(".r")[0] == c["instance"]),
              ("rational sides", lambda c: c["column"] == f"{c['instance']}/rational"))
    for name, pick in groups:
        cells = []
        for b in bounds:
            cs = [c for c in columns if pick(c) and c["b"] == b]
            m = combine([c["kappa"] - 1 for c in cs], [c["kappa_err"] for c in cs])
            lo, hi = min(c["kappa"] - 1 for c in cs), max(c["kappa"] - 1 for c in cs)
            cells.append(f"{pct(m['mean'], m['err'])} [{100 * lo:+.1f}, {100 * hi:+.1f}]")
        L.append(f"| {name} | " + " | ".join(cells) + " |")
    L.append("")
    L.append("**MNFS yield**: relations Σ_i O(F_i, G) from the four fields over O(F_0, G) from the instrument alone, and the pairs (a, b) with at least one relation over the same, summed over the instances:")
    L.append("")
    L.append("| | " + " | ".join(label(b) for b in bounds) + " |")
    L.append("|---|" + "---:|" * len(bounds))
    rel, uni = [], []
    for b in bounds:
        o0 = sum(r["observed"] for r in rows if r["kind"] == "pair" and r["cls"] == "cross" and r["s"] == r["instance"] and r["b"] == b)
        oall = sum(r["observed"] for r in rows if r["kind"] == "class" and r["cls"] == "cross" and r["b"] == b)
        ou = sum(u["observed"] for u in unions if u["b"] == b)
        rel.append(f"{oall / o0:.2f}" if o0 else "–")
        uni.append(f"{ou / o0:.2f}" if o0 else "–")
    L.append("| relations, four fields over one | " + " | ".join(rel) + " |")
    L.append("| pairs with a relation, four fields over one | " + " | ".join(uni) + " |")
    L.append("")
    L.append("## The criterion")
    L.append("")
    L.append("At each protocol bound: the related class's weighted mean R over its error, and the related minus the control over their combined error. The claim is supported only where both exceed 3.")
    L.append("")
    L.append("| class | " + " | ".join(f"{label(b)} related | {label(b)} minus control" for b in PROTOCOL) + " |")
    L.append("|---|" + "---:|---:|" * len(PROTOCOL))
    support = []
    for cls in CLASSES:
        cells = []
        for b in PROTOCOL:
            a, c = agg[(cls, b)], agg[(cls + " control", b)]
            za = a["mean"] / a["err"]
            zd = (a["mean"] - c["mean"]) / math.hypot(a["err"], c["err"])
            cells.append(f"{za:+.1f} | {zd:+.1f}")
            if za > 3 and zd > 3:
                support.append((cls, b, a["mean"]))
        L.append(f"| {cls} | " + " | ".join(cells) + " |")
    L.append("")
    if support:
        L.append("**Supported** at " + ", ".join(f"{c} at {label(b)} (R {pct(x)})" for c, b, x in support) + ".")
    else:
        L.append("**Rejected**: at no protocol bound does a related class exceed zero by three errors and its control by three combined errors.")
    Path(out + ".md").write_text("\n".join(L) + "\n")


if __name__ == "__main__":
    main(sys.argv)
