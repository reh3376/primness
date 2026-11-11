#!/usr/bin/env python3
"""H4: the laptop's one-hour reach with CADO-NFS end to end (docs/h4_falsifiable_prediction.md).

    scripts/h4_summary.py [--dir data/results/h4] [--out data/results/h4_reach]
    scripts/h4_summary.py --n D              print RSA-D, the product of its published factors
    scripts/h4_summary.py --filter-log LOG   print LOG's Info lines without paths, addresses or hashes

Reads per instance <dir>/rsa<d>.{time,factors,log.txt,hw.json} from scripts/h4_reach.sh. The
time is BSD `time -p -l` of the whole cado-nfs.py process. The phases come from the log's
timestamps. A run counts only if the factors it printed are the published ones. The reach d*
is interpolated linearly in ln T against the digit count, between the last run under one hour
and the first run over it, and the criterion is applied. Writes <out>.csv and <out>.md.
Standard library only.
"""
import csv
import json
import math
import os
import re
import statistics
import sys
from datetime import datetime
from pathlib import Path

# RSA challenge numbers: the published factorizations (1991 to 1999).
PUBLISHED = {
    100: (37975227936943673922808872755445627854565536638199,
          40094690950920881030683735292761468389214899724061),
    110: (6122421090493547576937037317561418841225758554253106999,
          5846418214406154678836553182979162384198610505601062333),
    120: (327414555693498015751146303749141488063642403240171463406883,
          693342667110830181197325401899700641361965863127336680673013),
    130: (39685999459597454290161126162883786067576449112810064832555157243,
          45534498646735972188403686897274408864356301263205069600999044599),
    140: (3398717423028438554530123627613875835633986495969597423490929302771479,
          6264200187401285096151654948264442219302037178623509019111660653946049),
}
HOUR = 3600.0
C_L = 1.923                       # (64/9)^(1/3), o(1) = 0
T200 = (4300.0, 4500.0, 4700.0)   # hours for a complete 200-digit factorization, from H1
CLAIM, LOW, HIGH = 132.0, 127.0, 137.0
PHASES = [  # label, the log line that starts it (the first occurrence counts)
    ("polynomial selection", "Polynomial Selection (size optimized): Starting"),
    ("factor base, free relations", "Generate Factor Base: Starting"),
    ("sieving", "Lattice Sieving: Starting"),
    ("filtering", "Lattice Sieving: Cancelling remaining workunits"),
    ("linear algebra", "Linear Algebra: Starting"),
    ("characters, square root", "Quadratic Characters: Starting"),
]
END = "Total cpu/elapsed time for entire Complete Factorization"
DROP = ("Client Launcher", "Server Launcher", "API server", "whitelist", "certsha1", "--server=",
        "Marking workunit", "Adding workunit", "Sending workunit", "got request", "Result file",
        "resumed with", "polynomial from file", "Received", "Newly arrived")
INFO = re.compile(r"(^| )Info:")
TS = re.compile(r"^(\d{4}-\d\d-\d\d \d\d:\d\d:\d\d,\d{3}) ")
IPV4 = re.compile(r"\b(\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})\b")


def f_L(ln_n):
    """The L_N[1/3] exponent (ln N)^(1/3) (ln ln N)^(2/3)."""
    return ln_n ** (1 / 3) * math.log(ln_n) ** (2 / 3)


def predicted_seconds(digits, t200=4500.0):
    return t200 * HOUR * math.exp(C_L * (f_L(digits * math.log(10)) - f_L(200 * math.log(10))))


def filter_log(path):
    root, home = str(Path.cwd()), os.path.expanduser("~")
    out = []
    for line in Path(path).read_text(errors="replace").splitlines():
        line = re.sub(r"\x1b\[[0-9;]*m", "", line)
        line = re.sub(r"^PID\d+ ", "", line)
        if not INFO.search(line) or any(k in line for k in DROP):
            continue
        if any(m.group(0) != "127.0.0.1" for m in IPV4.finditer(line)):
            continue
        out.append(line.replace(root, ".").replace(home, "~"))
    return "\n".join(out) + "\n"


def parse_time(path):
    t = {}
    for line in Path(path).read_text().splitlines():
        m = re.match(r"^(real|user|sys) ([\d.]+)$", line.strip())
        if m:
            t[m.group(1)] = float(m.group(2))
        m = re.match(r"^\s*(\d+)\s+maximum resident set size", line)
        if m:
            t["maxrss"] = int(m.group(1))
    return t


def parse_log(path):
    first, marks, end, info = None, {}, None, {}
    for line in Path(path).read_text().splitlines():
        mm = re.search(r"Using default parameter file (\S+)", line)
        if mm:
            info["params"] = Path(mm.group(1)).name
        m = TS.match(line)
        if not m:
            continue
        ts = datetime.strptime(m.group(1), "%Y-%m-%d %H:%M:%S,%f").timestamp()
        first = ts if first is None else first
        for label, marker in PHASES:
            if marker in line and label not in marks:
                marks[label] = ts
        if END in line:
            end = ts
            mm = re.search(r"Factorization ([\d.]+)/([\d.]+)", line)
            if mm:
                info["cado_cpu"], info["cado_elapsed"] = float(mm.group(1)), float(mm.group(2))
        mm = re.search(r"Lattice Sieving: Total number of relations: (\d+)", line)
        if mm:
            info["relations"] = int(mm.group(1))
        mm = re.search(r"(\d+) unique relations remain in total", line)
        if mm:
            info["unique"] = int(mm.group(1))
        mm = re.search(r"Merged matrix has (\d+) rows and total weight (\d+)", line)
        if mm:
            info["rows"], info["weight"] = int(mm.group(1)), int(mm.group(2))
        mm = re.search(r"best polynomial has Murphy_E = ([\d.e+-]+)", line)
        if mm:
            info["murphy_e"] = float(mm.group(1))
        mm = re.search(r"Average J: [\d.]+ for (\d+) special-q", line)
        if mm:
            info["special_q"] = int(mm.group(1))
    phases = {}
    order = [lab for lab, _ in PHASES if lab in marks]
    for k, lab in enumerate(order):
        stop = marks[order[k + 1]] if k + 1 < len(order) else end
        if stop is not None:
            phases[lab] = stop - marks[lab]
    if first is not None and order:
        phases["start-up"] = marks[order[0]] - first
    info["log_span"] = (end - first) if (end is not None and first is not None) else math.nan
    windows = []
    if first is not None and end is not None:
        windows.append(("whole run", first, end))
    for lab, nxt in (("sieving", "filtering"), ("linear algebra", "characters, square root")):
        if lab in marks and nxt in marks:
            windows.append((lab, marks[lab], marks[nxt]))
    info["windows"] = windows
    return phases, info


def trace_stats(d, windows):
    """Per window: CPU activity, the median clock of each of macmon's tiers, power and the
    hottest CPU reading, from the hw-sample trace data/hardware/h4-rsa<d>.jsonl when present."""
    p = Path(f"data/hardware/h4-rsa{d}.jsonl")
    if not p.exists():
        return []
    samples = [r for r in (json.loads(x) for x in p.read_text().splitlines() if x.strip()) if r.get("kind") == "sample"]

    def mean(vals):
        v = [x for x in vals if isinstance(x, (int, float))]
        return sum(v) / len(v) if v else math.nan

    out = []
    for label, t0, t1 in windows:
        sel = [r for r in samples if t0 <= r["ts_ms"] / 1000 <= t1]
        if not sel:
            continue
        clocks = {}
        for r in sel:
            for t in (r.get("cpu") or {}).get("tiers", []):
                clocks.setdefault(t.get("label"), []).append(t.get("freq_mhz"))
        temps = [(r.get("thermal") or {}).get("cpu_temp_c") for r in sel]
        out.append(dict(
            digits=d, window=label, samples=len(sel),
            cpu_active=mean([(r.get("cpu") or {}).get("active_ratio") for r in sel]),
            clock_p_mhz=statistics.median(clocks["P"]) if clocks.get("P") else math.nan,
            clock_s_mhz=statistics.median(clocks["S"]) if clocks.get("S") else math.nan,
            cpu_w=mean([(r.get("cpu") or {}).get("power_w") for r in sel]),
            all_w=mean([(r.get("power") or {}).get("all_w") for r in sel]),
            cpu_temp_c_max=max((x for x in temps if isinstance(x, (int, float))), default=math.nan)))
    return out


def load_run(d, root):
    tf = root / f"rsa{d}.time"
    if not tf.exists():
        return None
    r = {"digits": d, **parse_time(tf)}
    p, q = PUBLISHED[d]
    got = sorted(int(x) for x in re.findall(r"\d+", (root / f"rsa{d}.factors").read_text()))
    r["factors_ok"] = got == sorted((p, q)) and got[0] * got[1] == p * q
    lf = root / f"rsa{d}.log.txt"
    r["phases"], r["info"] = parse_log(lf) if lf.exists() else ({}, {})
    hw = root / f"rsa{d}.hw.json"
    r["hw"] = {}
    if hw.exists() and hw.read_text().strip():
        text = hw.read_text()
        r["hw"] = json.loads(text[text.index("{"):])
    return r


def stat(hw, key, field):
    v = (hw.get(key) or {}).get(field)
    return v if isinstance(v, (int, float)) else math.nan


def reach(runs):
    """d* where the wall time crosses one hour, linear in ln T on the digit count."""
    ok = sorted((r for r in runs if r["factors_ok"]), key=lambda r: r["digits"])
    if len(ok) < 2:
        return None, "fewer than two verified runs"
    under = [r for r in ok if r["real"] <= HOUR]
    over = [r for r in ok if r["real"] > HOUR]
    if under and over:
        lo, hi, how = under[-1], over[0], "interpolated"
    elif under:
        lo, hi, how = ok[-2], ok[-1], "extrapolated upward (no run over one hour)"
    else:
        lo, hi, how = ok[0], ok[1], "extrapolated downward (no run under one hour)"
    x0, x1 = lo["digits"], hi["digits"]
    y0, y1 = math.log(lo["real"]), math.log(hi["real"])
    return x0 + (math.log(HOUR) - y0) * (x1 - x0) / (y1 - y0), f"{how} between RSA-{x0} and RSA-{x1}"


def hms(s):
    if s != s:
        return "n/a"
    s = int(round(s))
    return f"{s // 3600}:{s % 3600 // 60:02d}:{s % 60:02d}"


def main(argv):
    if len(argv) == 3 and argv[1] == "--n":
        p, q = PUBLISHED[int(argv[2])]
        print(p * q)
        return
    if len(argv) == 3 and argv[1] == "--filter-log":
        sys.stdout.write(filter_log(argv[2]))
        return
    root, out = Path("data/results/h4"), "data/results/h4_reach"
    i = 1
    while i < len(argv):
        if argv[i] == "--dir":
            root = Path(argv[i + 1]); i += 2
        elif argv[i] == "--out":
            out = argv[i + 1]; i += 2
        else:
            sys.exit("usage: h4_summary.py [--dir DIR] [--out PATH] | --n D | --filter-log LOG")
    runs = [r for r in (load_run(d, root) for d in sorted(PUBLISHED)) if r]
    if not runs:
        sys.exit(f"{root}: no runs")
    d_star, how = reach(runs)
    labels = ["start-up"] + [lab for lab, _ in PHASES]

    rows = []
    for r in runs:
        ph, info, hw = r["phases"], r["info"], r["hw"]
        pred = predicted_seconds(r["digits"])
        rows.append(dict(
            digits=r["digits"], params=info.get("params", ""), factors_ok=r["factors_ok"],
            wall_s=r["real"], driver_user_s=r.get("user", math.nan), driver_sys_s=r.get("sys", math.nan),
            cado_cpu_s=info.get("cado_cpu", math.nan), cado_elapsed_s=info.get("cado_elapsed", math.nan),
            cpu_cores_mean=info.get("cado_cpu", math.nan) / info.get("cado_elapsed", math.nan),
            driver_maxrss_bytes=r.get("maxrss", ""),
            **{f"{lab.replace(', ', '_').replace(' ', '_').replace('-', '_')}_s": ph.get(lab, math.nan) for lab in labels},
            outside_log_s=r["real"] - info.get("log_span", math.nan),
            murphy_e=info.get("murphy_e", math.nan), special_q=info.get("special_q", ""),
            relations=info.get("relations", ""), unique=info.get("unique", ""),
            matrix_rows=info.get("rows", ""), matrix_weight=info.get("weight", ""),
            l_predicted_s=pred, measured_over_predicted=r["real"] / pred,
            power_w_mean=stat(hw, "power_all_w", "mean"), cpu_temp_c_max=stat(hw, "cpu_temp_c", "max"),
            cpu_active_mean=stat(hw, "cpu_active_ratio", "mean"), memory_used_ratio_max=stat(hw, "memory_used_ratio", "max")))
    with open(out + ".csv", "w", newline="") as fh:
        w = csv.DictWriter(fh, fieldnames=list(rows[0].keys()))
        w.writeheader()
        for row in rows:
            w.writerow({k: (f"{v:.6g}" if isinstance(v, float) else v) for k, v in row.items()})

    ln10 = math.log(10)
    md = ["# H4: the laptop's one-hour reach with CADO-NFS\n",
          "Generated by `scripts/h4_summary.py` from `scripts/h4_reach.sh`. The prediction and criterion are"
          " `docs/h4_falsifiable_prediction.md`. CADO-NFS at the commit in `data/results/h4/cado.commit`, its default"
          " parameter file per size (sha256 in `data/results/h4/params.c<d>.sha256`), `cado-nfs.py N -t 18`;"
          " the work server bound to localhost.\n",
          "## Runs\n",
          "| instance | parameters | wall time | factors | cores busy (CADO CPU / elapsed) | CPU active (hw-sample mean) | L-scaled time | measured / L-scaled | mean power W | max CPU °C |",
          "|---|---|---:|---|---:|---:|---:|---:|---:|---:|"]
    for r, row in zip(runs, rows):
        md.append(f"| RSA-{r['digits']} | {row['params']} | {hms(r['real'])} | {'published' if r['factors_ok'] else '**wrong**'} |"
                  f" {row['cpu_cores_mean']:.1f} of 18 | {100 * row['cpu_active_mean']:.0f}% | {hms(row['l_predicted_s'])} |"
                  f" {row['measured_over_predicted']:.1f} | {row['power_w_mean']:.1f} | {row['cpu_temp_c_max']:.1f} |")
    md.append("\nThe wall time is BSD `time` of the whole `cado-nfs.py` process. Its user and system times (in the CSV, `driver_*`)"
              " leave out the sieving clients, so CPU use comes from CADO-NFS's own total and from the hardware samples.")
    md.append("\n## Where the time goes\n")
    md.append("Wall time per phase from the log's timestamps; sieving includes the filtering passes that run while it continues.\n")
    md.append("| instance | " + " | ".join(labels) + " | outside the log |")
    md.append("|---|" + "---:|" * (len(labels) + 1))
    for r, row in zip(runs, rows):
        cells = []
        for lab in labels:
            v = r["phases"].get(lab, math.nan)
            cells.append(f"{hms(v)} ({100 * v / r['real']:.0f}%)" if v == v else "n/a")
        md.append(f"| RSA-{r['digits']} | " + " | ".join(cells) + f" | {hms(row['outside_log_s'])} |")
    md.append("\n| instance | Murphy E | special-q's | relations | unique | matrix rows | matrix weight |")
    md.append("|---|---:|---:|---:|---:|---:|---:|")
    for r, row in zip(runs, rows):
        def n(v):
            return f"{v:,}" if isinstance(v, int) else "n/a"
        md.append(f"| RSA-{r['digits']} | {row['murphy_e']:.3g} | {n(row['special_q'])} | {n(row['relations'])} | {n(row['unique'])} |"
                  f" {n(row['matrix_rows'])} | {n(row['matrix_weight'])} |")
    machine = [s for r in runs for s in trace_stats(r["digits"], r["info"].get("windows", []))]
    if machine:
        with open(str(root / "hardware_by_phase.csv"), "w", newline="") as fh:
            w = csv.DictWriter(fh, fieldnames=list(machine[0].keys()))
            w.writeheader()
            for s in machine:
                w.writerow({k: (f"{v:.6g}" if isinstance(v, float) else v) for k, v in s.items()})
        md.append("\n## The machine during the runs\n")
        md.append("From the hardware traces (`data/hardware/h4-rsa<d>.jsonl`, one sample a second, local), in"
                  f" `{root}/hardware_by_phase.csv`. Clocks are the medians of macmon's two CPU tiers under macmon's"
                  " labels: P has 12 cores and S has 6.\n")
        md.append("| instance | window | samples | CPU active | P clock MHz | S clock MHz | CPU W | all W | max CPU °C |")
        md.append("|---|---|---:|---:|---:|---:|---:|---:|---:|")
        for s in machine:
            md.append(f"| RSA-{s['digits']} | {s['window']} | {s['samples']:,} | {100 * s['cpu_active']:.0f}% |"
                      f" {s['clock_p_mhz']:.0f} | {s['clock_s_mhz']:.0f} | {s['cpu_w']:.1f} | {s['all_w']:.1f} | {s['cpu_temp_c_max']:.1f} |")
    md.append("\n## The reach and the criterion\n")
    if d_star is None:
        md.append(f"- No reach: {how}.")
    else:
        verdict = "holds" if LOW <= d_star <= HIGH else "rejected"
        md.append(f"- One hour of wall time is reached at **d\\* = {d_star:.1f} digits** ({how}).")
        md.append(f"- **H4 {verdict}**: d\\* " + ("lies" if verdict == "holds" else "does not lie")
                  + f" in [{LOW:.0f}, {HIGH:.0f}], 5 digits around the {CLAIM:.0f} that H1 carried down by L_N[1/3, {C_L}] gives.")
        span = f_L(200 * ln10) - f_L(d_star * ln10)
        ce = [math.log(t) / span for t in T200]
        md.append(f"- The effective constant that takes one hour at d\\* to H1's 4,300 to 4,700 hours at 200 digits:"
                  f" c_eff = {min(ce):.3f} to {max(ce):.3f}, against {C_L}.")
        md.append(f"- From d\\* to 200 digits is {200 - d_star:.0f} digits; at 200 digits the laptop needs 4,300 to 4,700 times its hour.")
    ok = [r for r in runs if r["factors_ok"]]
    if len(ok) >= 2:
        xs = [f_L(r["digits"] * ln10) for r in ok]
        ys = [math.log(r["real"]) for r in ok]
        mx, my = sum(xs) / len(xs), sum(ys) / len(ys)
        sxx = sum((x - mx) ** 2 for x in xs)
        slope = sum((x - mx) * (y - my) for x, y in zip(xs, ys)) / sxx
        md.append(f"- Across the runs, ln T rises {slope:.3f} per unit of the L_N[1/3] exponent (the local constant), against {C_L}.")
        for a, b in zip(ok, ok[1:]):
            dd = b["digits"] - a["digits"]
            ratio = b["real"] / a["real"]
            md.append(f"- RSA-{a['digits']} to RSA-{b['digits']}: × {ratio:.2f} in wall time, doubling every {dd * math.log(2) / math.log(ratio):.1f} digits"
                      f" (L-scaled: × {predicted_seconds(b['digits']) / predicted_seconds(a['digits']):.2f}).")
    Path(out + ".md").write_text("\n".join(md) + "\n")
    print("; ".join(f"RSA-{r['digits']} {hms(r['real'])} {'ok' if r['factors_ok'] else 'WRONG'}" for r in runs))
    if d_star is not None:
        print(f"d* = {d_star:.2f} digits ({how}); H4 {'holds' if LOW <= d_star <= HIGH else 'rejected'}")
    print(f"wrote {out}.md and {out}.csv")


if __name__ == "__main__":
    main(sys.argv)
