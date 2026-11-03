#!/usr/bin/env python3
"""Pick the root-optimized polynomial with the largest MurphyE from a
polyselect_ropt output file and write it as a CADO polynomial file.

    scripts/cado-best-poly.py data/instruments/d1-01/ropt.polys data/instruments/d1-01/cado.poly

The MurphyE value is an instrument score. It chooses which candidate the
harness measures; it is never used as the measurement. The written file
keeps it as a comment line, which the harness parser ignores.
"""
import re
import sys
from pathlib import Path


def blocks(text):
    current = None
    for line in text.splitlines():
        if line.startswith("### root-optimized polynomial"):
            if current:
                yield current
            current = {"lines": [], "murphy_e": None, "comments": []}
            continue
        if line.startswith("###"):
            if current:
                yield current
            current = None
            continue
        if current is None:
            continue
        m = re.search(r"MurphyE\(([^)]*)\)=([0-9.eE+-]+)", line)
        if m:
            current["murphy_e"] = float(m.group(2))
            current["murphy_params"] = m.group(1)
            current["comments"].append(line)
            continue
        if line.startswith("#"):
            current["comments"].append(line)
            continue
        if ":" in line:
            current["lines"].append(line.strip())
    if current:
        yield current


def main(argv):
    if len(argv) != 3:
        sys.exit("usage: cado-best-poly.py <ropt.polys> <cado.poly>")
    src, dst = Path(argv[1]), Path(argv[2])
    text = src.read_text()
    header = next((l for l in text.splitlines() if l.startswith("# (")), "")
    candidates = [b for b in blocks(text) if b["murphy_e"] is not None and b["lines"]]
    if not candidates:
        sys.exit(f"{src}: no root-optimized polynomial with a MurphyE value")
    best = max(candidates, key=lambda b: b["murphy_e"])
    keys = [l.split(":")[0] for l in best["lines"]]
    for required in ("n", "Y0", "Y1", "c0"):
        if required not in keys:
            sys.exit(f"{src}: best polynomial lacks {required}")
    out = [
        f"# Selected from {src} by scripts/cado-best-poly.py: largest MurphyE of {len(candidates)} root-optimized candidates.",
        f"# polyselect_ropt command line: {header.lstrip('# ')}" if header else "# polyselect_ropt command line unknown",
        f"# MurphyE({best.get('murphy_params', '')}) = {best['murphy_e']:.4e}  (instrument score; not used by the harness)",
    ]
    out += [c for c in best["comments"] if "MurphyE" not in c]
    out += best["lines"]
    dst.write_text("\n".join(out) + "\n")
    print(f"wrote {dst}: MurphyE {best['murphy_e']:.4e}, {len(candidates)} candidates")


if __name__ == "__main__":
    main(sys.argv)
