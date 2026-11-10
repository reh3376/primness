# Hardware telemetry

Every instrument run of 30 digits or more records the machine beside the measurement. The record is `hardware-sample/1` (`schemas/hardware-sample/1.json`): a `start` line with the machine metadata, one `sample` line per interval, and a `finish` line with min, mean and max of the sampled metrics. `schema_version` is required on every line, as on every data-collection schema in this repository. A new field set or source is a new version file; this one stays.

## Sources

- **macmon** (`brew install macmon`, IOReport without root): CPU active and frequency-scaled ratios per tier, GPU active ratio and frequency, CPU, GPU, ANE, RAM, package and system power, CPU and GPU temperature, fan speed, unified memory total, used and swap.
- **ioreg** (`ioreg -a -r -d 1 -c IOAccelerator`): the GPU accelerator's `Device Utilization %`, `Renderer Utilization %`, `Tiler Utilization %`, and the unified memory it holds (`In use system memory`, `Alloc system memory`).
- **sysctl**: model, OS version and build, core count, installed memory.

`memory.gpu_in_use_bytes` is the accelerator's in-use allocation; `memory.cpu_used_bytes` is used memory minus that; `memory.gpu_share_of_total` and `gpu_share_of_used` are the two ratios the roadmap asks for. `powermetrics` is not used because it needs root.

## Recording

- `hw-sample --run-id ID --interval-ms 1000 -- <command>` samples while a command runs, for example CADO-NFS polynomial selection through `scripts/cado-polyselect.sh`.
- `d1-baseline` starts the sampler the same way; its `d1-baseline/2` result embeds the finish summary under `hardware`. The default grid ran 95 s at a mean CPU activity of 0.51 and a CPU temperature peak of 90.5 °C.
- `d1-measure` on an instrument polynomial starts the sampler itself with the measurement `run_id` and stops it at the end. `--no-hw-sample` turns it off; `--hw-interval-ms` changes the interval (minimum 100). The finish summary is embedded in the `d1-measurement/1` result under `hardware`, with the trace path.
- The trace goes to `$PRIMNESS_HARDWARE_DIR/<run_id>.jsonl` (default `data/hardware`, gitignored) and, when `PRIMNESS_DATABASE_URL` connects, to the `hardware_samples` table (`db/migrations/002_hardware_samples.sql`), one JSONB row per line keyed by `run_id`, `ts_ms`, `kind`.
- Telemetry never fails the run. A missing `macmon` prints a warning and the measurement continues without a trace.

## Curation

- The curated record of a run is the `finish` line (or the `hardware` object in the result file): sample count, duration, and min, mean and max of CPU active ratio, GPU active ratio, GPU device utilization, memory used ratio, GPU share of memory, CPU and GPU temperature, and total power. Instrument runs keep it as `data/instruments/<id>/polyselect.hardware.json`; measurement runs keep it inside `data/results/<file>.json`. Both are committed.
- The full trace stays local and in Postgres. The bench plots it for the selected run: utilization in one chart, temperature and power in another, and the min, mean, max table.

## Known gaps

- macmon labels the 12-core tier `P` and the 6-core tier `S` on this M5 Max, while `sysctl hw.perflevel0.physicalcpu` reports 6 and `hw.perflevel1.physicalcpu` reports 12. The sample keeps macmon's labels and frequencies as reported; which tier is the performance tier is an open question, not a derived field.
- `Device Utilization %` is the driver's instantaneous value, not an average over the interval.
- The sampler is one process reading `macmon pipe`; `ioreg` is run once per sample. Its own cost is small (one short process per interval).
