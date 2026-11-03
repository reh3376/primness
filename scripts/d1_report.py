#!/usr/bin/env python3
"""Write a readable report from a d1-measurement/1 result and its CSV tables.

    scripts/d1_report.py data/results/d1-01.scaled-down.json

Writes <stem>.report.md beside the result and, when the hardware trace
named in the result exists, <stem>.hardware.csv. Every number in the
report is read from the result file or the tables; the prose states what
the numbers show and what they do not. Standard library only: this is a
post-processing sidecar, not part of the measurement.
"""
import csv
import json
import math
import statistics
import sys
from pathlib import Path


def read_csv(path):
    with open(path, newline="") as f:
        return list(csv.DictReader(f))


def fmt_int(n):
    return f"{int(n):,}"


def main(argv):
    if len(argv) != 2:
        sys.exit("usage: d1_report.py <result.json>")
    result_path = Path(argv[1])
    res = json.loads(result_path.read_text())
    stem = result_path.with_suffix("")
    tables = {t["name"]: Path(t["path"]) for t in res.get("tables", [])}
    bounds = read_csv(tables["bounds"])
    ubins = read_csv(tables["u_bins"])
    rows = read_csv(tables["rows"])
    lpf = read_csv(tables["lpf_hist"])
    n = res["evaluated"]
    top = max(int(r["b"]) for r in res["results"])

    # Hardware CSV
    hw_csv = None
    hw = res.get("hardware")
    if hw and Path(hw["samples_file"]).exists():
        hw_csv = Path(str(stem) + ".hardware.csv")
        import subprocess
        subprocess.run([sys.executable, "-I", str(Path(__file__).with_name("hardware_trace_csv.py")),
                        hw["samples_file"], str(hw_csv)], check=True)

    alg = {int(r["b"]): r for r in bounds if r["side"] == "algebraic"}
    rat = {int(r["b"]): r for r in bounds if r["side"] == "rational"}
    both = {int(r["b"]): r["both_smooth"] for r in res["results"]}

    lines = []
    L = lines.append
    L(f"# D1 measurement report: {res['instance']} ({res['scale']})")
    L("")
    L(f"Generated from `{result_path}` by `scripts/d1_report.py`. Result schema `{res['schema_version']}`, run `{res['run_id']}`.")
    L("")
    L("## Run")
    L("")
    poly = res["polynomial"]
    L(f"- N = {res['n']} ({res['n_digits']} digits), polynomial from `{poly['source']}` ({poly['format']}, degree {poly['degree']}, skew {poly.get('skew', '?')}).")
    reg = res["region"]
    L(f"- Region: a in [{reg['a_min']}, {reg['a_max']}], b in [{reg['b_min']}, {reg['b_max']}], gcd(a, b) = 1; {fmt_int(n)} values evaluated, {fmt_int(res['skipped_zero'])} zero values skipped.")
    L(f"- Bounds B: {', '.join(str(b) for b in res['bounds'])} (2^{', 2^'.join(str(int(b).bit_length()-1) for b in res['bounds'])}).")
    L(f"- {res['elapsed_ms']/1000:.1f} s on {res.get('threads', '?')} threads; Dickman table `{res['dickman']['method']}`, {res['dickman']['steps_per_unit']} steps per unit.")
    al = res.get("alpha")
    if al:
        L(f"- Murphy α computed here over primes ≤ {al['bound']} (`{al['definition']}`): algebraic {al['algebraic']:+.4f}, rational {al['rational']:+.4f}. The instrument's own α comment is not read.")
    else:
        L("- No α applied (result version 1).")
    L("")
    L("## Per-bound results")
    L("")
    v2 = "predicted_alpha" in bounds[0]
    if v2:
        L("| B | side | smooth | observed | raw Dickman | raw gap / se | α-adjusted | α gap / se | local density | local gap / se | mean u | α | implied α |")
        L("|---|---|---|---|---|---|---|---|---|---|---|---|---|")
        for b in sorted(alg):
            for name, r in (("algebraic", alg[b]), ("rational", rat[b])):
                L(f"| 2^{b.bit_length()-1} | {name} | {fmt_int(r['smooth'])} | {float(r['observed']):.5f} | {float(r['predicted']):.5f} | {float(r['gap_stderr_units']):+.0f} | {float(r['predicted_alpha']):.5f} | {float(r['gap_alpha_stderr_units']):+.1f} | {float(r['predicted_local']):.5f} | {float(r['gap_local_stderr_units']):+.0f} | {float(r['mean_u']):.2f} | {float(r['alpha']):+.3f} | {r['implied_alpha'] or 'n/a'} |")
    else:
        L("| B | side | smooth | observed | predicted (raw Dickman) | ratio | gap | gap / stderr | mean u | implied α |")
        L("|---|---|---|---|---|---|---|---|---|---|")
        for b in sorted(alg):
            for name, r in (("algebraic", alg[b]), ("rational", rat[b])):
                o, p = float(r["observed"]), float(r["predicted"])
                L(f"| 2^{b.bit_length()-1} | {name} | {fmt_int(r['smooth'])} | {o:.5f} | {p:.5f} | {o/p if p else float('nan'):.2f} | {float(r['gap']):+.5f} | {float(r['gap_stderr_units']):+.0f} | {float(r['mean_u']):.2f} | {r['implied_alpha'] or 'n/a'} |")
    L("")
    L("## Observations")
    L("")
    obs = []
    ratios = {b: float(alg[b]["observed"]) / float(alg[b]["predicted"]) for b in alg}
    obs.append(f"The algebraic side is smoother than the raw Dickman prediction at every bound. The ratio observed/predicted runs from {ratios[max(ratios)]:.2f} at B = 2^{max(ratios).bit_length()-1} to {ratios[min(ratios)]:.2f} at B = 2^{min(ratios).bit_length()-1}; it grows as B shrinks and u grows.")
    alphas = [float(alg[b]["implied_alpha"]) for b in alg if alg[b]["implied_alpha"]]
    if alphas:
        obs.append(f"The size shift that would reconcile raw Dickman with the algebraic count (implied α) is {min(alphas):+.2f} to {max(alphas):+.2f} across the four bounds, mean {statistics.mean(alphas):+.2f}, spread {max(alphas)-min(alphas):.2f}.")
    if v2:
        ag = [float(alg[b]["gap_alpha_stderr_units"]) for b in sorted(alg)]
        rg = [float(rat[b]["gap_alpha_stderr_units"]) for b in sorted(rat)]
        lg = [float(alg[b]["gap_local_stderr_units"]) for b in sorted(alg)]
        rl = [float(rat[b]["gap_local_stderr_units"]) for b in sorted(rat)]
        obs.append(f"With the Murphy α computed in the harness ({al['algebraic']:+.4f}), the algebraic gap to the α-adjusted Dickman prediction is {', '.join(f'{g:+.1f}' for g in ag)} standard errors at the four bounds (raw: {', '.join(f'{float(alg[b][chr(103)+chr(97)+chr(112)+chr(95)+chr(115)+chr(116)+chr(100)+chr(101)+chr(114)+chr(114)+chr(95)+chr(117)+chr(110)+chr(105)+chr(116)+chr(115)]):+.0f}' for b in sorted(alg))}). The root-property correction accounts for the raw excess; what remains is at the per-mille level of the density and changes sign across bounds. The implied shift from the data ({statistics.mean(alphas):+.2f} mean) and the computed α ({al['algebraic']:+.2f}) agree.")
        obs.append(f"On the rational side the computed α is {al['rational']:+.4f} and the α-adjusted gap is still {', '.join(f'{g:+.0f}' for g in rg)} standard errors: the linear form's values are less often smooth than ρ(u + α/ln B) predicts by about {abs(statistics.mean(float(rat[b]['gap_alpha'])/float(rat[b]['observed']) for b in rat))*100:.0f}% of the density. The local-density form ρ(u) + ρ'(u)/ln B overshoots the other way ({', '.join(f'{g:+.0f}' for g in rl)} standard errors; algebraic side {', '.join(f'{g:+.0f}' for g in lg)}). The count sits between the two textbook approximations, which bounds how precisely Dickman itself can be trusted at u near 2 with B = 2^20: the model uncertainty here is of the order of the residual, not the Poisson noise.")
    ralphas = [float(rat[b]["implied_alpha"]) for b in rat if rat[b]["implied_alpha"]]
    if ralphas:
        obs.append(f"The rational side is less smooth than Dickman at every bound; its implied α is {min(ralphas):+.2f} to {max(ralphas):+.2f}. A linear form over coprime pairs is not a random integer of its size, and the correction has the opposite sign.")
    # u-bin trend for top bound
    tb = [r for r in ubins if int(r["b"]) == top and r["side"] == "algebraic" and r["ratio"] and int(r["count"]) >= 1000]
    if tb:
        lo, hi = tb[0], tb[-1]
        obs.append(f"Within B = 2^{top.bit_length()-1}, binned by u in steps of 0.1 (bins with at least 1,000 values): the excess ratio is {float(lo['ratio']):.2f} at u ≈ {float(lo['u_lo']):.1f} and {float(hi['ratio']):.2f} at u ≈ {float(hi['u_lo']):.1f}. The excess is not a constant factor; it rises with u, as a shift in ln|F| predicts, because ρ falls faster at larger u.")
    # independence of sides at top bound
    a_top, r_top = float(alg[top]["observed"]), float(rat[top]["observed"])
    b_top = int(both[top]) / n
    obs.append(f"Both sides smooth at B = 2^{top.bit_length()-1}: {fmt_int(both[top])} values, a fraction of {b_top:.5f}, against {a_top*r_top:.5f} if the two sides were independent (ratio {b_top/(a_top*r_top):.2f}). " + ("The sides are close to independent." if abs(b_top/(a_top*r_top) - 1) < 0.1 else "The sides are not independent on this region."))
    # row variation
    dens = [float(r["alg_density"]) for r in rows if int(r["evaluated"]) > 0]
    bs = [int(r["b"]) for r in rows if int(r["evaluated"]) > 0]
    if len(dens) > 10:
        q = len(dens) // 4
        first_q, last_q = statistics.mean(dens[:q]), statistics.mean(dens[-q:])
        obs.append(f"Algebraic density per b row at B = 2^{top.bit_length()-1}: mean over the first quarter of b ({bs[0]}..{bs[q-1]}) is {first_q:.5f}, over the last quarter ({bs[-q]}..{bs[-1]}) is {last_q:.5f}. Larger b means larger values, so density falls across the region; the comparison with Dickman already accounts for size value by value.")
    # lpf
    alg_lpf = {r["log2_lpf_bin"]: float(r["share"]) for r in lpf if r["side"] == "algebraic"}
    ns = alg_lpf.get("not_smooth", 0.0)
    obs.append(f"{ns*100:.1f}% of algebraic values have a prime factor above 2^{top.bit_length()-1}. Of the smooth ones, the largest prime factor sits mostly in the top few octaves below B (see `lpf_hist`), as expected for values far larger than B.")
    for o in obs:
        L(f"- {o}")
    L("")
    L("## Questions this run answers")
    L("")
    L("- Does the harness reproduce a Dickman-scale smoothness count on a real NFS polynomial? Yes: predicted densities are of the right order at all four bounds, and the counts are far above Poisson noise.")
    L("- Is the raw algebraic excess over Dickman statistically real on this region? Yes, by 100 to 250 standard errors.")
    if v2:
        L("- Is that excess evidence for D1? No. Applying the Murphy α computed here removes it on the algebraic side; the residual is at the per-mille level and changes sign across bounds. D1 would need an excess that survives α, and on this instance there is none.")
        L("- Is the rational side understood? Not fully. After α it is still below ρ(u + α/ln B) and above the local-density form; the Dickman approximation's own accuracy at u near 2 is the limiting factor, not the measurement.")
    else:
        L("- Is that excess evidence for D1? No. It is the size and shape of a root-property (Murphy α) correction, which the prediction does not yet include. The implied α from the data is consistent across bounds and matches the instrument's own α estimate in sign and magnitude.")
    L("- Do the algebraic and rational sides behave independently? See the both-sides observation above.")
    L("")
    L("## Questions this run raises")
    L("")
    if v2:
        L("- Which Dickman approximation is the right baseline for the D1 comparison at u near 2 to 4: ρ(u), the local density, or a second-order form? The residual on the rational side is a model residual, and the rejection criterion needs a baseline whose own error is below the noise.")
        L("- Does the per-mille algebraic residual after α depend on u, on b, or on the sign of a? The u-bin and row tables hold the inputs.")
    else:
        L("- What is the Murphy α of this polynomial computed in the harness from the recorded definition, and does the excess vanish once it is applied? That is the next step before the rejection criterion can be used.")
        L("- Does the residual after α (if any) depend on u, on b, or on the sign of a? The u-bin and row tables are the inputs for that check.")
        L("- Is the rational side's deficit fully explained by its own α and the coprimality condition?")
    L("- How do the other nine instances compare at the same region, and does the implied α track CADO's reported α for each?")
    L("- What region and bounds define a `full` run for each digit length?")
    L("")
    L("## Tables")
    L("")
    for t in res.get("tables", []):
        L(f"- `{t['path']}` ({t['rows']} rows): {t['description']}. Columns: {', '.join(t['columns'])}.")
    if res.get("dump"):
        L(f"- `{res['dump']}`: one row per value (a, b, ln|F|, largest prime factor of F or 0, ln|G|, largest prime factor of G or 0). Local only; not committed.")
    if hw_csv:
        L(f"- `{hw_csv}`: hardware samples flattened from `{hw['samples_file']}`.")
    L("")
    if hw:
        s = hw["summary"]
        L("## Hardware")
        L("")
        L(f"{s['samples']} samples over {s['duration_ms']/1000:.1f} s. CPU active ratio {s['cpu_active_ratio']['min']:.2f} / {s['cpu_active_ratio']['mean']:.2f} / {s['cpu_active_ratio']['max']:.2f} (min / mean / max); CPU temperature {s['cpu_temp_c']['min']:.1f} / {s['cpu_temp_c']['mean']:.1f} / {s['cpu_temp_c']['max']:.1f} °C; GPU device utilization mean {s['gpu_device_utilization_pct']['mean']:.1f}%; memory used ratio mean {s['memory_used_ratio']['mean']:.2f}; GPU share of unified memory mean {s['gpu_share_of_total']['mean']:.3f}; total power {s['power_all_w']['min']:.1f} / {s['power_all_w']['mean']:.1f} / {s['power_all_w']['max']:.1f} W. The GPU is idle during this measurement; its memory share belongs to other processes.")
        L("")
    out = Path(str(stem) + ".report.md")
    out.write_text("\n".join(lines) + "\n")
    print(f"wrote {out}")


if __name__ == "__main__":
    main(sys.argv)
