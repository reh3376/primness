#!/usr/bin/env python3
"""Post measurement events from JSONL files to the bench's /api/ingest, so
runs recorded without PRIMNESS_DATABASE_URL reach Postgres and the bench.

    scripts/ingest-metrics.py [data/metrics/*.jsonl]   (default: every file there)

The bench validates each event against measurement-event/1 and inserts it
(ON CONFLICT DO NOTHING, so re-posting is harmless). Standard library only.
"""
import glob
import json
import os
import sys
import urllib.request


def main(argv):
    url = os.environ.get("PRIMNESS_METRICS_URL", "http://127.0.0.1:9477/api/ingest")
    files = argv[1:] or sorted(glob.glob("data/metrics/*.jsonl"))
    posted = rejected = 0
    for path in files:
        for line in open(path):
            line = line.strip()
            if not line:
                continue
            try:
                json.loads(line)
            except json.JSONDecodeError:
                rejected += 1
                continue
            req = urllib.request.Request(url, data=line.encode(), headers={"Content-Type": "application/json"}, method="POST")
            try:
                with urllib.request.urlopen(req, timeout=5) as resp:
                    if resp.status in (200, 204):
                        posted += 1
                    else:
                        rejected += 1
            except urllib.error.HTTPError as err:
                rejected += 1
                sys.stderr.write(f"{path}: {err.code} {err.read().decode()[:120]}\n")
            except OSError as err:
                sys.exit(f"{url}: {err}")
    print(f"posted {posted} events, rejected {rejected}, from {len(files)} files to {url}")


if __name__ == "__main__":
    main(sys.argv)
