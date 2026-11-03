-- Hardware telemetry recorded beside a run. One row per hardware-sample/1
-- line; the document is kept whole so a new schema version needs no new
-- columns. JSONL files under data/hardware remain the local buffer.

CREATE TABLE IF NOT EXISTS hardware_samples (
    id BIGSERIAL PRIMARY KEY,
    schema_version TEXT NOT NULL,
    run_id TEXT NOT NULL,
    ts_ms BIGINT NOT NULL,
    kind TEXT NOT NULL,
    sample JSONB NOT NULL,
    UNIQUE (run_id, ts_ms, kind)
);

CREATE INDEX IF NOT EXISTS hardware_samples_run_ts ON hardware_samples (run_id, ts_ms);
