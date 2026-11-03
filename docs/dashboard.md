# Primness bench

Local view of data collection and measurement rate. It does not gate CI or decide whether a D1 claim holds.

The page is served by the `primness-bench` container at `http://127.0.0.1:$PRIMNESS_DASHBOARD_PORT/` from the gitignored `.env` (default 9477). The port is loopback only. `scripts/up-dashboard.sh` checks that port is free before `docker compose up`.

Harnesses append JSON lines under `data/metrics/`, which the container reads. A run can also `POST` the same JSON to `http://127.0.0.1:9477/api/ingest` by setting `PRIMNESS_METRICS_URL`. `PRIMNESS_METRICS_DIR` overrides the directory. The default is `data/metrics`.

```sh
scripts/up-dashboard.sh
cargo run --bin d1-measure -- --region 80 examples/demo.poly
cargo run --bin d1_generate_instances
```

Current harness output is marked `scale: toy`. The bench says so on the page. That is not the 40–80 digit protocol.

Each line has `ts_ms`, `run_id`, `source`, `kind` (`start`, `sample`, or `finish`), `scale`, `instance`, `b`, `evaluated`, `smooth`, `elapsed_ms`, `rate_per_s`, and `residues` (a string of `1` and `0` for the latest values). Gold cells in the strip are smooth values. The gold trace is density. The blue trace is the chunk rate, scaled to the fastest chunk in that run.
