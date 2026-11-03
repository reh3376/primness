-- Measurement event log. The bench reads the latest runs from this table.
-- JSONL files remain a local buffer when the database is down.

CREATE TABLE IF NOT EXISTS schema_migrations (
    version TEXT PRIMARY KEY,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS schema_documents (
    schema_version TEXT PRIMARY KEY,
    document JSONB NOT NULL,
    registered_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS events (
    id BIGSERIAL PRIMARY KEY,
    schema_version TEXT NOT NULL,
    ts_ms BIGINT NOT NULL,
    run_id TEXT NOT NULL,
    source TEXT NOT NULL,
    kind TEXT NOT NULL,
    scale TEXT NOT NULL,
    instance TEXT NOT NULL,
    b BIGINT NOT NULL,
    evaluated BIGINT NOT NULL,
    smooth BIGINT NOT NULL,
    elapsed_ms BIGINT NOT NULL,
    rate_per_s DOUBLE PRECISION NOT NULL,
    residues TEXT NOT NULL DEFAULT '',
    UNIQUE (run_id, schema_version, kind, b, evaluated, elapsed_ms)
);

CREATE INDEX IF NOT EXISTS events_run_ts ON events (run_id, ts_ms);
