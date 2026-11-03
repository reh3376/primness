use crate::db::Store;
use crate::schema::{Registry, V1};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// One observation from a harness. A run is the sequence of events that share `run_id`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Event {
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
    pub ts_ms: u64,
    pub run_id: String,
    pub source: String,
    pub kind: String,
    pub scale: String,
    pub instance: String,
    pub b: u64,
    pub evaluated: u64,
    pub smooth: u64,
    pub elapsed_ms: u64,
    pub rate_per_s: f64,
    pub residues: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SamplePoint {
    pub ts_ms: u64,
    pub b: u64,
    pub evaluated: u64,
    pub smooth: u64,
    pub elapsed_ms: u64,
    pub rate_per_s: f64,
    pub density: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct RunView {
    pub run_id: String,
    pub schema_version: String,
    pub source: String,
    pub scale: String,
    pub instance: String,
    pub started_ms: u64,
    pub last_ms: u64,
    pub last_kind: String,
    pub evaluated: u64,
    pub smooth: u64,
    pub elapsed_ms: u64,
    pub rate_per_s: f64,
    pub b: u64,
    pub residues: String,
    pub samples: Vec<SamplePoint>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Snapshot {
    pub runs: Vec<RunView>,
}

fn default_schema_version() -> String {
    V1.to_string()
}

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn density(smooth: u64, evaluated: u64) -> f64 {
    if evaluated == 0 {
        0.0
    } else {
        smooth as f64 / evaluated as f64
    }
}

/// Append-only writer. Metrics never fail the measurement; errors are printed and dropped.
pub struct Sink {
    file: Option<File>,
    url: Option<String>,
    registry: Option<Registry>,
    db: Option<Store>,
}

impl Sink {
    pub fn open(dir: &Path, run_id: &str, url: Option<String>) -> Self {
        let path = dir.join(format!("{run_id}.jsonl"));
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|err| eprintln!("metrics file {}: {err}", path.display()))
            .ok();
        let registry = Registry::from_env()
            .map_err(|err| eprintln!("schema registry: {err}"))
            .ok();
        let db = std::env::var("PRIMNESS_DATABASE_URL")
            .ok()
            .filter(|url| !url.is_empty())
            .and_then(|url| {
                Store::connect(&url)
                    .map_err(|err| eprintln!("metrics db: {err}"))
                    .ok()
            });
        Self {
            file,
            url,
            registry,
            db,
        }
    }

    pub fn from_env(run_id: &str) -> Self {
        let dir =
            std::env::var("PRIMNESS_METRICS_DIR").unwrap_or_else(|_| "data/metrics".to_string());
        let url = std::env::var("PRIMNESS_METRICS_URL").ok();
        let dir_path = PathBuf::from(&dir);
        if let Err(err) = fs::create_dir_all(&dir_path) {
            eprintln!("metrics dir {dir}: {err}");
        }
        Self::open(&dir_path, run_id, url)
    }

    pub fn emit(&mut self, event: &Event) {
        if let Some(registry) = &self.registry {
            if let Err(err) = registry.validate(event) {
                eprintln!("event rejected by schema: {err}");
                return;
            }
        } else {
            eprintln!("event rejected: schema registry is not loaded");
            return;
        }
        if let Some(file) = self.file.as_mut() {
            if let Ok(line) = serde_json::to_string(event) {
                if let Err(err) = writeln!(file, "{line}") {
                    eprintln!("metrics write: {err}");
                } else if let Err(err) = file.flush() {
                    eprintln!("metrics flush: {err}");
                }
            }
        }
        if let Some(url) = &self.url {
            if let Err(err) = post_event(url, event) {
                eprintln!("metrics post {url}: {err}");
            }
        }
        if let Some(db) = self.db.as_mut() {
            db.push(event);
        }
    }
}

impl Drop for Sink {
    fn drop(&mut self) {
        if let Some(db) = self.db.as_mut() {
            if let Err(err) = db.flush() {
                eprintln!("metrics db: {err}");
            }
        }
    }
}

pub fn post_event(url: &str, event: &Event) -> Result<(), String> {
    let body = serde_json::to_string(event).map_err(|err| err.to_string())?;
    let (host, port, path) = parse_http_url(url)?;
    let mut stream = TcpStream::connect((host.as_str(), port)).map_err(|err| err.to_string())?;
    let _ = stream.set_write_timeout(Some(std::time::Duration::from_secs(2)));
    let req = format!(
        "POST {path} HTTP/1.1\r\nHost: {host}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream
        .write_all(req.as_bytes())
        .map_err(|err| err.to_string())
}

fn parse_http_url(url: &str) -> Result<(String, u16, String), String> {
    let rest = url
        .strip_prefix("http://")
        .ok_or_else(|| format!("only http:// metrics URLs are accepted, got {url}"))?;
    let (authority, path) = rest.split_once('/').unwrap_or((rest, ""));
    let path = format!("/{path}");
    let (host, port) = if let Some((host, port)) = authority.rsplit_once(':') {
        let port = port
            .parse::<u16>()
            .map_err(|_| format!("bad port in {url}"))?;
        (host.to_string(), port)
    } else {
        (authority.to_string(), 80)
    };
    if host.is_empty() {
        return Err(format!("missing host in {url}"));
    }
    Ok((host, port, path))
}

pub fn load_snapshot(dir: &Path) -> Snapshot {
    if let Ok(url) = std::env::var("PRIMNESS_DATABASE_URL") {
        if !url.is_empty() {
            match crate::db::load_snapshot(&url) {
                Ok(snap) => return snap,
                Err(err) => eprintln!("metrics db read: {err}; using files"),
            }
        }
    }
    load_dir(dir)
}

pub fn load_dir(dir: &Path) -> Snapshot {
    let mut events = Vec::new();
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return Snapshot { runs: Vec::new() },
    };
    let mut files: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "jsonl"))
        .collect();
    files.sort();
    for path in files {
        let Ok(file) = File::open(&path) else {
            continue;
        };
        for line in BufReader::new(file).lines() {
            let Ok(line) = line else { continue };
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            match serde_json::from_str::<Event>(line) {
                Ok(event) => events.push(event),
                Err(_) => continue,
            }
        }
    }
    Snapshot {
        runs: group_runs(events),
    }
}

pub fn group_runs(events: Vec<Event>) -> Vec<RunView> {
    let mut runs: Vec<RunView> = Vec::new();
    for event in events {
        if event.kind != "start" && event.kind != "sample" && event.kind != "finish" {
            continue;
        }
        let slot = runs.iter_mut().find(|run| run.run_id == event.run_id);
        let run = if let Some(run) = slot {
            run
        } else {
            runs.push(RunView {
                run_id: event.run_id.clone(),
                schema_version: event.schema_version.clone(),
                source: event.source.clone(),
                scale: event.scale.clone(),
                instance: event.instance.clone(),
                started_ms: event.ts_ms,
                last_ms: event.ts_ms,
                last_kind: event.kind.clone(),
                evaluated: 0,
                smooth: 0,
                elapsed_ms: 0,
                rate_per_s: 0.0,
                b: event.b,
                residues: String::new(),
                samples: Vec::new(),
            });
            runs.last_mut().expect("just pushed")
        };
        run.last_ms = event.ts_ms;
        run.last_kind = event.kind.clone();
        if !event.schema_version.is_empty() {
            run.schema_version = event.schema_version.clone();
        }
        if !event.scale.is_empty() {
            run.scale = event.scale;
        }
        if !event.instance.is_empty() {
            run.instance = event.instance;
        }
        if event.kind == "start" {
            continue;
        }
        run.evaluated = event.evaluated;
        run.smooth = event.smooth;
        run.elapsed_ms = event.elapsed_ms;
        run.rate_per_s = event.rate_per_s;
        run.b = event.b;
        if !event.residues.is_empty() {
            run.residues = event.residues;
        }
        if event.kind != "sample" {
            continue;
        }
        run.samples.push(SamplePoint {
            ts_ms: event.ts_ms,
            b: event.b,
            evaluated: event.evaluated,
            smooth: event.smooth,
            elapsed_ms: event.elapsed_ms,
            rate_per_s: event.rate_per_s,
            density: density(event.smooth, event.evaluated),
        });
    }
    runs.sort_by(|a, b| b.last_ms.cmp(&a.last_ms));
    runs
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(kind: &str, evaluated: u64, smooth: u64) -> Event {
        Event {
            schema_version: crate::schema::V1.into(),
            ts_ms: 1_000 + evaluated,
            run_id: "run-a".into(),
            source: "d1-measure".into(),
            kind: kind.into(),
            scale: "toy".into(),
            instance: "demo.poly".into(),
            b: 32,
            evaluated,
            smooth,
            elapsed_ms: evaluated,
            rate_per_s: 1000.0,
            residues: if kind == "sample" {
                "1100".into()
            } else {
                String::new()
            },
        }
    }

    #[test]
    fn groups_samples_under_one_run() {
        let runs = group_runs(vec![
            event("start", 0, 0),
            event("sample", 10, 4),
            event("finish", 10, 4),
        ]);
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].evaluated, 10);
        assert_eq!(runs[0].samples.len(), 1);
        assert_eq!(runs[0].samples[0].density, 0.4);
        assert_eq!(runs[0].residues, "1100");
        assert_eq!(runs[0].last_kind, "finish");
        assert_eq!(runs[0].schema_version, crate::schema::V1);
    }

    #[test]
    fn skips_a_partial_line_and_keeps_the_rest() {
        let dir = std::env::temp_dir().join(format!("primness-metrics-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("run.jsonl");
        let mut file = File::create(&path).unwrap();
        let good = serde_json::to_string(&event("sample", 5, 1)).unwrap();
        writeln!(file, "{good}").unwrap();
        writeln!(file, "{{\"run_id\":").unwrap();
        let snap = load_dir(&dir);
        assert_eq!(snap.runs.len(), 1);
        assert_eq!(snap.runs[0].evaluated, 5);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn parses_metrics_url() {
        let (host, port, path) = parse_http_url("http://127.0.0.1:9477/api/ingest").unwrap();
        assert_eq!(host, "127.0.0.1");
        assert_eq!(port, 9477);
        assert_eq!(path, "/api/ingest");
    }
}
