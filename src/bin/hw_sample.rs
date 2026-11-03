// hw-sample: record a hardware-sample/1 trace beside a run.
//
//   hw-sample --run-id ID [--interval-ms 1000] -- <command> [args...]
//       samples while the command runs, then stops.
//   hw-sample --run-id ID [--interval-ms 1000] --samples N
//       records N samples and stops.
//
// The trace goes to $PRIMNESS_HARDWARE_DIR/<run-id>.jsonl (default
// data/hardware) and to Postgres when PRIMNESS_DATABASE_URL connects.

use primness::hardware::{Sampler, DEFAULT_INTERVAL_MS};
use primness::metrics::now_ms;
use std::env;
use std::process::{Command, ExitCode};
use std::time::Duration;

fn usage() -> ExitCode {
    eprintln!(
        "Usage: hw-sample [--run-id ID] [--interval-ms N] (--samples N | -- <command> [args...])"
    );
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let mut run_id = format!("hw-{}-{}", std::process::id(), now_ms());
    let mut interval_ms = DEFAULT_INTERVAL_MS;
    let mut samples: Option<u64> = None;
    let mut command: Vec<String> = Vec::new();
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--run-id" => match args.next() {
                Some(value) if !value.is_empty() => run_id = value,
                _ => return usage(),
            },
            "--interval-ms" => match args.next().and_then(|v| v.parse().ok()) {
                Some(value) => interval_ms = value,
                None => return usage(),
            },
            "--samples" => match args.next().and_then(|v| v.parse().ok()) {
                Some(value) if value > 0 => samples = Some(value),
                _ => return usage(),
            },
            "--" => {
                command = args.collect();
                break;
            }
            _ => return usage(),
        }
    }
    if command.is_empty() && samples.is_none() {
        return usage();
    }

    let sampler = match Sampler::start(&run_id, interval_ms) {
        Ok(sampler) => sampler,
        Err(err) => {
            eprintln!("hw-sample: {err}");
            return ExitCode::from(1);
        }
    };
    eprintln!("hw-sample: run {run_id}, every {interval_ms} ms, trace {}", sampler.path.display());

    let mut command_status = ExitCode::SUCCESS;
    if command.is_empty() {
        let n = samples.unwrap_or(1);
        std::thread::sleep(Duration::from_millis(interval_ms * n + interval_ms / 2));
    } else {
        match Command::new(&command[0]).args(&command[1..]).status() {
            Ok(status) => {
                if !status.success() {
                    eprintln!("hw-sample: command exited with {status}");
                    command_status = ExitCode::from(1);
                }
            }
            Err(err) => {
                eprintln!("hw-sample: {}: {err}", command[0]);
                command_status = ExitCode::from(1);
            }
        }
    }

    match sampler.stop() {
        Ok(summary) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&summary).unwrap_or_default()
            );
            command_status
        }
        Err(err) => {
            eprintln!("hw-sample: {err}");
            ExitCode::from(1)
        }
    }
}
