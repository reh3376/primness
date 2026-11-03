#!/usr/bin/env python3
"""Flatten a hardware-sample/1 JSONL trace into a CSV of its sample rows.

    scripts/hardware_trace_csv.py data/hardware/<run_id>.jsonl data/results/<stem>.hardware.csv

Columns are one per sampled metric; the start and finish records are
skipped (the finish summary lives in the result JSON). Only the standard
library is used.
"""
import csv
import json
import sys
from pathlib import Path

COLUMNS = [
    "ts_ms", "elapsed_s", "cpu_active_ratio", "cpu_scaled_ratio", "cpu_power_w",
    "gpu_active_ratio", "gpu_freq_mhz", "gpu_power_w", "gpu_device_utilization_pct",
    "gpu_renderer_utilization_pct", "gpu_memory_in_use_bytes",
    "memory_total_bytes", "memory_used_bytes", "memory_used_ratio",
    "memory_cpu_used_bytes", "memory_gpu_share_of_total", "memory_gpu_share_of_used",
    "power_all_w", "power_sys_w", "power_ram_w", "cpu_temp_c", "gpu_temp_c", "fan0_rpm", "fan1_rpm",
]


def rows(path):
    first = None
    for line in Path(path).read_text().splitlines():
        if not line.strip():
            continue
        rec = json.loads(line)
        if rec.get("schema_version") != "hardware-sample/1" or rec.get("kind") != "sample":
            continue
        if first is None:
            first = rec["ts_ms"]
        cpu, gpu, mem, pw, th = rec["cpu"], rec["gpu"], rec["memory"], rec["power"], rec["thermal"]
        fans = th.get("fans", [])
        yield [
            rec["ts_ms"], round((rec["ts_ms"] - first) / 1000, 3),
            cpu["active_ratio"], cpu["scaled_ratio"], cpu["power_w"],
            gpu["active_ratio"], gpu["freq_mhz"], gpu["power_w"], gpu["device_utilization_pct"],
            gpu["renderer_utilization_pct"], gpu["memory_in_use_bytes"],
            mem["total_bytes"], mem["used_bytes"], mem["used_ratio"],
            mem["cpu_used_bytes"], mem["gpu_share_of_total"], mem["gpu_share_of_used"],
            pw["all_w"], pw["sys_w"], pw["ram_w"], th["cpu_temp_c"], th["gpu_temp_c"],
            fans[0]["rpm"] if len(fans) > 0 else "", fans[1]["rpm"] if len(fans) > 1 else "",
        ]


def main(argv):
    if len(argv) != 3:
        sys.exit("usage: hardware_trace_csv.py <trace.jsonl> <out.csv>")
    out = Path(argv[2])
    out.parent.mkdir(parents=True, exist_ok=True)
    with out.open("w", newline="") as f:
        w = csv.writer(f)
        w.writerow(COLUMNS)
        n = 0
        for r in rows(argv[1]):
            w.writerow(r)
            n += 1
    print(f"wrote {out}: {n} samples")


if __name__ == "__main__":
    main(sys.argv)
