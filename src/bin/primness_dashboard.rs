use primness::metrics::{self, Event};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::time::Duration;

const PAGE: &str = include_str!("../dashboard.html");

fn main() {
    let port: u16 = std::env::var("PRIMNESS_DASHBOARD_PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(9477);
    let dir = std::env::var("PRIMNESS_METRICS_DIR")
        .unwrap_or_else(|_| "data/metrics".to_string());
    let dir = PathBuf::from(dir);
    if let Err(err) = std::fs::create_dir_all(&dir) {
        eprintln!("metrics dir {}: {err}", dir.display());
    }

    let listener = TcpListener::bind(("0.0.0.0", port)).unwrap_or_else(|err| {
        eprintln!("TCP {port} is not available: {err}");
        std::process::exit(1);
    });
    eprintln!(
        "Primness bench listening on {port}, reading {}",
        dir.display()
    );

    for conn in listener.incoming() {
        let Ok(mut stream) = conn else { continue };
        let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
        let mut buf = [0u8; 65536];
        let Ok(n) = stream.read(&mut buf) else { continue };
        if n == 0 {
            continue;
        }
        let text = String::from_utf8_lossy(&buf[..n]);
        let mut lines = text.split("\r\n");
        let request = lines.next().unwrap_or("");
        let mut parts = request.split_whitespace();
        let method = parts.next().unwrap_or("");
        let path = parts.next().unwrap_or("/");
        match (method, path) {
            ("GET", "/") | ("GET", "/index.html") => {
                write_response(&mut stream, "200 OK", "text/html; charset=utf-8", PAGE.as_bytes());
            }
            ("GET", "/api/snapshot") => {
                let snap = metrics::load_dir(&dir);
                let body = serde_json::to_string(&snap).unwrap_or_else(|_| "{\"runs\":[]}".into());
                write_response(&mut stream, "200 OK", "application/json", body.as_bytes());
            }
            ("POST", "/api/ingest") => {
                let body = text.split("\r\n\r\n").nth(1).unwrap_or("").trim();
                match serde_json::from_str::<Event>(body) {
                    Ok(event) => {
                        if let Err(err) = append_ingest(&dir, &event) {
                            eprintln!("ingest: {err}");
                            write_response(&mut stream, "500 Internal Server Error", "text/plain", err.as_bytes());
                        } else {
                            write_response(&mut stream, "204 No Content", "text/plain", b"");
                        }
                    }
                    Err(err) => {
                        let msg = format!("bad event: {err}");
                        write_response(&mut stream, "400 Bad Request", "text/plain", msg.as_bytes());
                    }
                }
            }
            _ => write_response(&mut stream, "404 Not Found", "text/plain", b"not found"),
        }
    }
}

fn append_ingest(dir: &std::path::Path, event: &Event) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|err| err.to_string())?;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("ingest.jsonl"))
        .map_err(|err| err.to_string())?;
    let line = serde_json::to_string(event).map_err(|err| err.to_string())?;
    writeln!(file, "{line}").map_err(|err| err.to_string())
}

fn write_response(stream: &mut TcpStream, status: &str, content_type: &str, body: &[u8]) {
    let header = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(header.as_bytes());
    let _ = stream.write_all(body);
}
