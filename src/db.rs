use crate::metrics::{group_runs, Event, Snapshot};
use postgres::{Client, NoTls};

const INSERT: &str = "\
INSERT INTO events (
    schema_version, ts_ms, run_id, source, kind, scale, instance,
    b, evaluated, smooth, elapsed_ms, rate_per_s, residues
) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)
ON CONFLICT (run_id, schema_version, kind, b, evaluated, elapsed_ms) DO NOTHING";

const LOAD: &str = "\
SELECT schema_version, ts_ms, run_id, source, kind, scale, instance,
       b, evaluated, smooth, elapsed_ms, rate_per_s, residues
FROM events
WHERE run_id IN (
    SELECT run_id FROM (
        SELECT run_id, max(ts_ms) AS last_ms
        FROM events
        GROUP BY run_id
        ORDER BY last_ms DESC
        LIMIT 20
    ) recent
)
ORDER BY ts_ms ASC";

pub struct Store {
    client: Client,
    pending: Vec<Event>,
}

impl Store {
    pub fn connect(url: &str) -> Result<Self, postgres::Error> {
        Ok(Self {
            client: Client::connect(url, NoTls)?,
            pending: Vec::new(),
        })
    }

    pub fn push(&mut self, event: &Event) {
        self.pending.push(event.clone());
        let flush = event.kind != "sample" || self.pending.len() >= 32;
        if flush {
            if let Err(err) = self.flush() {
                eprintln!("metrics db: {err}");
            }
        }
    }

    pub fn flush(&mut self) -> Result<(), String> {
        if self.pending.is_empty() {
            return Ok(());
        }
        self.write_pending()
    }

    fn write_pending(&mut self) -> Result<(), String> {
        let mut tx = self.client.transaction().map_err(|err| err.to_string())?;
        for event in &self.pending {
            let ts_ms = i64::try_from(event.ts_ms).map_err(|_| "ts_ms".to_string())?;
            let b = i64::try_from(event.b).map_err(|_| "b".to_string())?;
            let evaluated = i64::try_from(event.evaluated).map_err(|_| "evaluated".to_string())?;
            let smooth = i64::try_from(event.smooth).map_err(|_| "smooth".to_string())?;
            let elapsed_ms =
                i64::try_from(event.elapsed_ms).map_err(|_| "elapsed_ms".to_string())?;
            tx.execute(
                INSERT,
                &[
                    &event.schema_version,
                    &ts_ms,
                    &event.run_id,
                    &event.source,
                    &event.kind,
                    &event.scale,
                    &event.instance,
                    &b,
                    &evaluated,
                    &smooth,
                    &elapsed_ms,
                    &event.rate_per_s,
                    &event.residues,
                ],
            )
            .map_err(|err| err.to_string())?;
        }
        tx.commit().map_err(|err| err.to_string())?;
        self.pending.clear();
        Ok(())
    }
}

impl Drop for Store {
    fn drop(&mut self) {
        if let Err(err) = self.flush() {
            eprintln!("metrics db: {err}");
        }
    }
}

pub fn load_snapshot(url: &str) -> Result<Snapshot, String> {
    let mut client = Client::connect(url, NoTls).map_err(|err| err.to_string())?;
    let rows = client.query(LOAD, &[]).map_err(|err| err.to_string())?;
    let mut events = Vec::with_capacity(rows.len());
    for row in rows {
        events.push(Event {
            schema_version: row.get(0),
            ts_ms: u64::try_from(row.get::<_, i64>(1)).unwrap_or(0),
            run_id: row.get(2),
            source: row.get(3),
            kind: row.get(4),
            scale: row.get(5),
            instance: row.get(6),
            b: u64::try_from(row.get::<_, i64>(7)).unwrap_or(0),
            evaluated: u64::try_from(row.get::<_, i64>(8)).unwrap_or(0),
            smooth: u64::try_from(row.get::<_, i64>(9)).unwrap_or(0),
            elapsed_ms: u64::try_from(row.get::<_, i64>(10)).unwrap_or(0),
            rate_per_s: row.get(11),
            residues: row.get(12),
        });
    }
    Ok(Snapshot {
        runs: group_runs(events),
    })
}

pub fn register_schemas(
    url: &str,
    documents: &[(String, serde_json::Value)],
) -> Result<(), String> {
    let mut client = Client::connect(url, NoTls).map_err(|err| err.to_string())?;
    for (version, document) in documents {
        client
            .execute(
                "INSERT INTO schema_documents (schema_version, document) VALUES ($1, $2)
                 ON CONFLICT (schema_version) DO UPDATE SET document = EXCLUDED.document",
                &[version, document],
            )
            .map_err(|err| err.to_string())?;
    }
    Ok(())
}
