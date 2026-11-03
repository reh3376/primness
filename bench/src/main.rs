mod page;

use primness::db::Store;
use primness::metrics::{self, Event};
use primness::schema::Registry;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::time::Duration;

fn main() {
    let host = std::env::var("PRIMNESS_DASHBOARD_HOST")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "127.0.0.1".to_string());
    let port: u16 = std::env::var("PRIMNESS_DASHBOARD_PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(9477);
    let dir = PathBuf::from(
        std::env::var("PRIMNESS_METRICS_DIR").unwrap_or_else(|_| "data/metrics".to_string()),
    );
    if let Err(err) = std::fs::create_dir_all(&dir) {
        eprintln!("metrics dir {}: {err}", dir.display());
    }
    let hardware_dir = primness::hardware::hardware_dir();
    if let Err(err) = std::fs::create_dir_all(&hardware_dir) {
        eprintln!("hardware dir {}: {err}", hardware_dir.display());
    }
    register_schemas();

    let listener = TcpListener::bind((host.as_str(), port)).unwrap_or_else(|err| {
        eprintln!("TCP {host}:{port} is not available: {err}");
        std::process::exit(1);
    });
    eprintln!(
        "Primness bench listening on {host}:{port}, reading {}",
        dir.display()
    );

    for conn in listener.incoming() {
        let Ok(mut stream) = conn else { continue };
        let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
        let mut buf = [0u8; 65536];
        let Ok(n) = stream.read(&mut buf) else {
            continue;
        };
        if n == 0 {
            continue;
        }
        let text = String::from_utf8_lossy(&buf[..n]);
        let mut lines = text.split("\r\n");
        let request = lines.next().unwrap_or("");
        let mut parts = request.split_whitespace();
        let method = parts.next().unwrap_or("");
        let path = parts.next().unwrap_or("/");
        let (route, query) = path.split_once('?').unwrap_or((path, ""));
        match (method, route) {
            ("GET", "/") | ("GET", "/index.html") => {
                let selected = query_param(query, "run").unwrap_or_default();
                let snap = metrics::load_snapshot(&dir);
                let html = page::render(snap, &selected, &hardware_dir);
                write_response(
                    &mut stream,
                    "200 OK",
                    "text/html; charset=utf-8",
                    html.as_bytes(),
                );
            }
            ("GET", "/api/snapshot") => {
                let snap = metrics::load_snapshot(&dir);
                let body = serde_json::to_string(&snap).unwrap_or_else(|_| "{\"runs\":[]}".into());
                write_response(&mut stream, "200 OK", "application/json", body.as_bytes());
            }
            ("POST", "/api/ingest") => {
                let body = text.split("\r\n\r\n").nth(1).unwrap_or("").trim();
                match serde_json::from_str::<Event>(body) {
                    Ok(event) => match accept_event(&dir, &event) {
                        Ok(()) => write_response(&mut stream, "204 No Content", "text/plain", b""),
                        Err((status, err)) => {
                            eprintln!("ingest: {err}");
                            write_response(&mut stream, status, "text/plain", err.as_bytes());
                        }
                    },
                    Err(err) => {
                        let msg = format!("bad event: {err}");
                        write_response(
                            &mut stream,
                            "400 Bad Request",
                            "text/plain",
                            msg.as_bytes(),
                        );
                    }
                }
            }
            _ => write_response(&mut stream, "404 Not Found", "text/plain", b"not found"),
        }
    }
}

fn register_schemas() {
    let Ok(url) = std::env::var("PRIMNESS_DATABASE_URL") else {
        return;
    };
    if url.is_empty() {
        return;
    }
    let registry = match Registry::from_env() {
        Ok(registry) => registry,
        Err(err) => {
            eprintln!("schema registry: {err}");
            return;
        }
    };
    let documents = registry.documents();
    for attempt in 1..=30 {
        match primness::db::register_schemas(&url, &documents) {
            Ok(()) => {
                eprintln!("registered {} schema document(s)", documents.len());
                return;
            }
            Err(err) => {
                if attempt == 30 {
                    eprintln!("schema register: {err}");
                }
                std::thread::sleep(Duration::from_secs(1));
            }
        }
    }
}

fn accept_event(dir: &Path, event: &Event) -> Result<(), (&'static str, String)> {
    let registry = Registry::from_env().map_err(|err| ("500 Internal Server Error", err))?;
    registry
        .validate(event)
        .map_err(|err| ("400 Bad Request", err))?;
    if let Ok(url) = std::env::var("PRIMNESS_DATABASE_URL") {
        if !url.is_empty() {
            match Store::connect(&url) {
                Ok(mut store) => {
                    store.push(event);
                    match store.flush() {
                        Ok(()) => return Ok(()),
                        Err(err) => {
                            eprintln!("ingest db: {err}");
                            append_jsonl(dir, event)?;
                            return Ok(());
                        }
                    }
                }
                Err(err) => {
                    eprintln!("ingest db: {err}");
                    append_jsonl(dir, event)?;
                    return Ok(());
                }
            }
        }
    }
    append_jsonl(dir, event)?;
    Ok(())
}

fn append_jsonl(dir: &Path, event: &Event) -> Result<(), (&'static str, String)> {
    std::fs::create_dir_all(dir).map_err(|err| ("500 Internal Server Error", err.to_string()))?;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join(format!("{}.jsonl", event.run_id)))
        .map_err(|err| ("500 Internal Server Error", err.to_string()))?;
    let line = serde_json::to_string(event)
        .map_err(|err| ("500 Internal Server Error", err.to_string()))?;
    writeln!(file, "{line}").map_err(|err| ("500 Internal Server Error", err.to_string()))
}

fn query_param(query: &str, name: &str) -> Option<String> {
    for pair in query.split('&') {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        if key == name {
            return Some(url_decode(value));
        }
    }
    None
}

fn url_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(byte) =
                u8::from_str_radix(std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or(""), 16)
            {
                out.push(byte);
                i += 3;
                continue;
            }
        }
        if bytes[i] == b'+' {
            out.push(b' ');
        } else {
            out.push(bytes[i]);
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn write_response(stream: &mut TcpStream, status: &str, content_type: &str, body: &[u8]) {
    let header = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(header.as_bytes());
    let _ = stream.write_all(body);
}
