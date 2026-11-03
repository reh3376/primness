# Primness bench

Local view of data collection and measurement rate. It does not gate CI or decide whether a D1 claim holds.

The page is Leptos server-rendered HTML from the `primness-bench` package (`bench/`, binary `primness-dashboard`). The measurement crate does not compile the UI. The `primness-bench` container serves it at `http://127.0.0.1:$PRIMNESS_DASHBOARD_PORT/` (default 9477, loopback only). The binary binds `PRIMNESS_DASHBOARD_HOST`, default `127.0.0.1`; the container sets `0.0.0.0` so the loopback port mapping can reach it. `scripts/up-dashboard.sh` checks that port, and the Postgres port, before `docker compose up`. The page reloads itself every 2 seconds.

Postgres is the record of a run. The login and host port are `POSTGRES_*` and `PRIMNESS_DATABASE_URL` in the gitignored `.env`. The database files stay in the Docker volume `primness-pg`. Migrations live in `db/migrations/` and `scripts/apply-migrations.sh` applies each file once.

Harnesses still append JSON lines under `data/metrics/` so a run is not lost when Postgres is down. When `PRIMNESS_DATABASE_URL` is set they also insert into `events`. The bench reads Postgres, and falls back to those files if the database cannot be reached. A run can also `POST` the same JSON to `/api/ingest` by setting `PRIMNESS_METRICS_URL`.

Collection is governed by JSON Schema files under `schemas/`. The measurement contract is `schemas/measurement-event/1.json` (`$id` `measurement-event/1`). A new version is a new file. The sink and `/api/ingest` reject an event whose `schema_version` is unknown or whose JSON does not match that file. The measurement itself still finishes. `scripts/apply-migrations.sh` stores each schema document in `schema_documents`, and the bench registers the same documents when it starts. The roadmap snapshot gate is a separate contract, `schemas/current-state/1.json` (`$id` `current-state/1`). An external polynomial file is `schemas/instrument-polynomial/1.json` (`$id` `instrument-polynomial/1`). Neither is a measurement event.

```sh
scripts/up-dashboard.sh
cargo run --bin d1-measure -- --region 80 examples/demo.poly
cargo run --bin d1_generate_instances
```

Toy runs are marked `scale: toy`; instrument runs are `scaled-down` until the protocol's region is fixed. The bench says so on the page. Neither is the 40–80 digit protocol.

Below the measurement table the page plots the hardware trace of the selected run from `$PRIMNESS_HARDWARE_DIR/<run_id>.jsonl` (`hardware-sample/1`, see `docs/hardware_telemetry.md`): utilization and memory in one chart, temperature and power in another, then min, mean and max.

Each line has `ts_ms`, `run_id`, `source`, `kind` (`start`, `sample`, or `finish`), `scale`, `instance`, `b`, `evaluated`, `smooth`, `elapsed_ms`, `rate_per_s`, and `residues` (a string of `1` and `0` for the latest values). Gold cells in the strip are smooth values. The gold trace is density. The blue trace is the chunk rate, scaled to the fastest chunk in that run.
