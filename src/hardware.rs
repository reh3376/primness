//! Hardware telemetry recorded beside a run.
//!
//! Sources on this Mac: `macmon pipe` (IOReport without root: CPU and GPU
//! activity, frequency, power, temperature, fans, unified memory) and
//! `ioreg -c IOAccelerator` (the GPU accelerator's utilization and the
//! unified memory it holds). Each interval produces one `hardware-sample/1`
//! record. A trace is a `start` record with the machine metadata, the
//! `sample` records, and a `finish` record with min, mean and max of the
//! sampled metrics. Records are validated against the schema, appended as
//! JSON lines under `PRIMNESS_HARDWARE_DIR` (default `data/hardware`), and
//! inserted into `hardware_samples` when `PRIMNESS_DATABASE_URL` connects.
//! Telemetry never fails the run: a sample that cannot be read is reported
//! on stderr and skipped.

use crate::metrics::now_ms;
use crate::schema::Registry;
use postgres::{Client, NoTls};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Instant;

pub const V1: &str = "hardware-sample/1";
pub const SAMPLER: &str = "hw-sample";
pub const SAMPLER_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DEFAULT_INTERVAL_MS: u64 = 1000;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Tier {
    pub label: String,
    pub cores: u64,
    pub max_freq_mhz: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Machine {
    pub model: String,
    pub chip: String,
    pub os_version: String,
    pub os_build: String,
    pub cpu_cores: u64,
    pub cpu_tiers: Vec<Tier>,
    pub gpu_cores: u64,
    pub memory_total_bytes: u64,
    pub sampler: String,
    pub sampler_version: String,
    pub macmon_version: String,
    pub sources: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TierLoad {
    pub label: String,
    pub active_ratio: f64,
    pub freq_mhz: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Cpu {
    pub active_ratio: f64,
    pub scaled_ratio: f64,
    pub power_w: f64,
    pub tiers: Vec<TierLoad>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Gpu {
    pub active_ratio: f64,
    pub scaled_ratio: f64,
    pub freq_mhz: u64,
    pub power_w: f64,
    pub device_utilization_pct: u64,
    pub renderer_utilization_pct: u64,
    pub tiler_utilization_pct: u64,
    pub memory_in_use_bytes: u64,
    pub memory_alloc_bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Memory {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub swap_used_bytes: u64,
    pub gpu_in_use_bytes: u64,
    pub cpu_used_bytes: u64,
    pub used_ratio: f64,
    pub gpu_share_of_total: f64,
    pub gpu_share_of_used: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Power {
    pub all_w: f64,
    pub sys_w: f64,
    pub cpu_w: f64,
    pub gpu_w: f64,
    pub ane_w: f64,
    pub ram_w: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Fan {
    pub name: String,
    pub rpm: u64,
    pub max_rpm: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Thermal {
    pub cpu_temp_c: f64,
    pub gpu_temp_c: f64,
    pub fans: Vec<Fan>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Stat {
    pub min: f64,
    pub mean: f64,
    pub max: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Summary {
    pub samples: u64,
    pub duration_ms: u64,
    pub cpu_active_ratio: Stat,
    pub gpu_active_ratio: Stat,
    pub gpu_device_utilization_pct: Stat,
    pub memory_used_ratio: Stat,
    pub gpu_share_of_total: Stat,
    pub cpu_temp_c: Stat,
    pub gpu_temp_c: Stat,
    pub power_all_w: Stat,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Sample {
    pub schema_version: String,
    pub ts_ms: u64,
    pub run_id: String,
    pub kind: String,
    pub interval_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub machine: Option<Machine>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu: Option<Cpu>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gpu: Option<Gpu>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory: Option<Memory>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub power: Option<Power>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thermal: Option<Thermal>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<Summary>,
}

impl Sample {
    fn bare(run_id: &str, kind: &str, interval_ms: u64) -> Self {
        Self {
            schema_version: V1.into(),
            ts_ms: now_ms(),
            run_id: run_id.into(),
            kind: kind.into(),
            interval_ms,
            machine: None,
            cpu: None,
            gpu: None,
            memory: None,
            power: None,
            thermal: None,
            summary: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Parsing the two sources

/// One `macmon pipe` line reduced to the fields a sample records.
#[derive(Clone, Debug, PartialEq)]
pub struct MacmonReading {
    pub cpu: Cpu,
    pub gpu_active_ratio: f64,
    pub gpu_scaled_ratio: f64,
    pub gpu_freq_mhz: u64,
    pub gpu_power_w: f64,
    pub ram_total: u64,
    pub ram_usage: u64,
    pub swap_usage: u64,
    pub power: Power,
    pub thermal: Thermal,
}

fn number(value: &Value, key: &str) -> Result<f64, String> {
    value
        .get(key)
        .and_then(Value::as_f64)
        .ok_or_else(|| format!("macmon line has no numeric {key}"))
}

fn unsigned(value: &Value, key: &str) -> Result<u64, String> {
    let n = number(value, key)?;
    if n < 0.0 {
        return Err(format!("macmon {key} is negative"));
    }
    Ok(n.round() as u64)
}

fn ratio(value: &Value, key: &str) -> Result<f64, String> {
    Ok(number(value, key)?.clamp(0.0, 1.0))
}

fn watts(value: &Value, key: &str) -> Result<f64, String> {
    Ok(number(value, key)?.max(0.0))
}

pub fn parse_macmon(line: &str) -> Result<MacmonReading, String> {
    let v: Value = serde_json::from_str(line).map_err(|err| format!("macmon json: {err}"))?;
    let mut tiers = Vec::new();
    if let Some(list) = v.get("cpu_tiers").and_then(Value::as_array) {
        for tier in list {
            tiers.push(TierLoad {
                label: tier
                    .get("label")
                    .and_then(Value::as_str)
                    .unwrap_or("?")
                    .to_string(),
                active_ratio: ratio(tier, "active_ratio")?,
                freq_mhz: unsigned(tier, "freq_mhz")?,
            });
        }
    }
    let memory = v
        .get("memory")
        .ok_or_else(|| "macmon line has no memory".to_string())?;
    let temp = v
        .get("temp")
        .ok_or_else(|| "macmon line has no temp".to_string())?;
    let mut fans = Vec::new();
    if let Some(list) = v.get("fans").and_then(Value::as_array) {
        for fan in list {
            fans.push(Fan {
                name: fan
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or("fan")
                    .to_string(),
                rpm: unsigned(fan, "rpm")?,
                max_rpm: unsigned(fan, "max_rpm")?,
            });
        }
    }
    Ok(MacmonReading {
        cpu: Cpu {
            active_ratio: ratio(&v, "cpu_active_ratio")?,
            scaled_ratio: ratio(&v, "cpu_scaled_ratio")?,
            power_w: watts(&v, "cpu_power")?,
            tiers,
        },
        gpu_active_ratio: ratio(&v, "gpu_active_ratio")?,
        gpu_scaled_ratio: ratio(&v, "gpu_scaled_ratio")?,
        gpu_freq_mhz: unsigned(&v, "gpu_freq_mhz")?,
        gpu_power_w: watts(&v, "gpu_power")?,
        ram_total: unsigned(memory, "ram_total")?,
        ram_usage: unsigned(memory, "ram_usage")?,
        swap_usage: unsigned(memory, "swap_usage")?,
        power: Power {
            all_w: watts(&v, "all_power")?,
            sys_w: watts(&v, "sys_power")?,
            cpu_w: watts(&v, "cpu_power")?,
            gpu_w: watts(&v, "gpu_power")?,
            ane_w: watts(&v, "ane_power")?,
            ram_w: watts(&v, "ram_power")?,
        },
        thermal: Thermal {
            cpu_temp_c: number(temp, "cpu_temp_avg")?,
            gpu_temp_c: number(temp, "gpu_temp_avg")?,
            fans,
        },
    })
}

/// The GPU accelerator's performance statistics from `ioreg -a -r -d 1 -c IOAccelerator`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GpuStats {
    pub device_utilization_pct: u64,
    pub renderer_utilization_pct: u64,
    pub tiler_utilization_pct: u64,
    pub in_use_bytes: u64,
    pub alloc_bytes: u64,
    pub core_count: Option<u64>,
}

fn integer_after_key(xml: &str, from: usize, key: &str) -> Option<u64> {
    let marker = format!("<key>{key}</key>");
    let at = xml.get(from..)?.find(&marker)? + from + marker.len();
    let rest = &xml[at..];
    let open = rest.find("<integer>")?;
    if !rest[..open].trim().is_empty() {
        return None;
    }
    let after = &rest[open + "<integer>".len()..];
    let close = after.find("</integer>")?;
    after[..close].trim().parse().ok()
}

pub fn parse_ioreg_plist(xml: &str) -> Result<GpuStats, String> {
    let perf = xml
        .find("<key>PerformanceStatistics</key>")
        .ok_or_else(|| "ioreg: no PerformanceStatistics".to_string())?;
    let read = |key: &str| {
        integer_after_key(xml, perf, key).ok_or_else(|| format!("ioreg: no integer {key}"))
    };
    Ok(GpuStats {
        device_utilization_pct: read("Device Utilization %")?.min(100),
        renderer_utilization_pct: read("Renderer Utilization %")?.min(100),
        tiler_utilization_pct: read("Tiler Utilization %")?.min(100),
        in_use_bytes: read("In use system memory")?,
        alloc_bytes: read("Alloc system memory")?,
        core_count: integer_after_key(xml, 0, "gpu-core-count"),
    })
}

pub fn read_ioreg() -> Result<GpuStats, String> {
    let output = Command::new("ioreg")
        .args(["-a", "-r", "-d", "1", "-c", "IOAccelerator"])
        .output()
        .map_err(|err| format!("ioreg: {err}"))?;
    if !output.status.success() {
        return Err(format!("ioreg exited with {}", output.status));
    }
    parse_ioreg_plist(&String::from_utf8_lossy(&output.stdout))
}

/// Combine the two sources into one `sample` record.
pub fn compose_sample(run_id: &str, interval_ms: u64, m: &MacmonReading, g: &GpuStats) -> Sample {
    let total = m.ram_total.max(1);
    let gpu_in_use = g.in_use_bytes.min(total);
    let used = m.ram_usage.min(total);
    let mut sample = Sample::bare(run_id, "sample", interval_ms);
    sample.cpu = Some(m.cpu.clone());
    sample.gpu = Some(Gpu {
        active_ratio: m.gpu_active_ratio,
        scaled_ratio: m.gpu_scaled_ratio,
        freq_mhz: m.gpu_freq_mhz,
        power_w: m.gpu_power_w,
        device_utilization_pct: g.device_utilization_pct,
        renderer_utilization_pct: g.renderer_utilization_pct,
        tiler_utilization_pct: g.tiler_utilization_pct,
        memory_in_use_bytes: g.in_use_bytes,
        memory_alloc_bytes: g.alloc_bytes,
    });
    sample.memory = Some(Memory {
        total_bytes: total,
        used_bytes: used,
        swap_used_bytes: m.swap_usage,
        gpu_in_use_bytes: gpu_in_use,
        cpu_used_bytes: used.saturating_sub(gpu_in_use),
        used_ratio: used as f64 / total as f64,
        gpu_share_of_total: gpu_in_use as f64 / total as f64,
        gpu_share_of_used: if used == 0 {
            0.0
        } else {
            (gpu_in_use as f64 / used as f64).min(1.0)
        },
    });
    sample.power = Some(m.power.clone());
    sample.thermal = Some(m.thermal.clone());
    sample
}

// ---------------------------------------------------------------------------
// Machine metadata

fn sysctl(names: &[&str]) -> Result<Vec<String>, String> {
    let mut args = vec!["-n"];
    args.extend_from_slice(names);
    let output = Command::new("sysctl")
        .args(&args)
        .output()
        .map_err(|err| format!("sysctl: {err}"))?;
    let text = String::from_utf8_lossy(&output.stdout);
    let values: Vec<String> = text.lines().map(|l| l.trim().to_string()).collect();
    if values.len() != names.len() {
        return Err(format!(
            "sysctl returned {} values for {} names",
            values.len(),
            names.len()
        ));
    }
    Ok(values)
}

pub fn macmon_version() -> Result<String, String> {
    let output = Command::new("macmon")
        .arg("--version")
        .output()
        .map_err(|err| format!("macmon: {err}"))?;
    let text = String::from_utf8_lossy(&output.stdout);
    Ok(text
        .split_whitespace()
        .last()
        .unwrap_or("unknown")
        .to_string())
}

pub fn machine_metadata() -> Result<Machine, String> {
    let values = sysctl(&[
        "hw.model",
        "kern.osproductversion",
        "kern.osversion",
        "hw.memsize",
        "hw.ncpu",
    ])?;
    let output = Command::new("macmon")
        .args(["pipe", "-s", "1", "-i", "200", "--soc-info"])
        .stderr(Stdio::null())
        .output()
        .map_err(|err| format!("macmon: {err}"))?;
    let line = String::from_utf8_lossy(&output.stdout);
    let line = line
        .lines()
        .find(|l| !l.trim().is_empty())
        .ok_or_else(|| "macmon printed no sample".to_string())?;
    let v: Value = serde_json::from_str(line).map_err(|err| format!("macmon json: {err}"))?;
    let soc = v
        .get("soc")
        .ok_or_else(|| "macmon --soc-info has no soc".to_string())?;
    let chip = soc
        .get("chip_name")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let mut cpu_tiers = Vec::new();
    if let Some(list) = soc.get("cpu_tiers").and_then(Value::as_array) {
        for tier in list {
            let max_freq = tier
                .get("freqs")
                .and_then(Value::as_array)
                .and_then(|f| f.iter().filter_map(Value::as_u64).max())
                .unwrap_or(0);
            cpu_tiers.push(Tier {
                label: tier
                    .get("label")
                    .and_then(Value::as_str)
                    .unwrap_or("?")
                    .to_string(),
                cores: tier.get("cores").and_then(Value::as_u64).unwrap_or(0).max(1),
                max_freq_mhz: max_freq,
            });
        }
    }
    if cpu_tiers.is_empty() {
        cpu_tiers.push(Tier {
            label: "all".into(),
            cores: values[4].parse().unwrap_or(1),
            max_freq_mhz: 0,
        });
    }
    let gpu_cores = read_ioreg().ok().and_then(|g| g.core_count).unwrap_or(0);
    Ok(Machine {
        model: values[0].clone(),
        chip,
        os_version: values[1].clone(),
        os_build: values[2].clone(),
        cpu_cores: values[4].parse().map_err(|_| "hw.ncpu".to_string())?,
        cpu_tiers,
        gpu_cores,
        memory_total_bytes: values[3].parse().map_err(|_| "hw.memsize".to_string())?,
        sampler: SAMPLER.into(),
        sampler_version: SAMPLER_VERSION.into(),
        macmon_version: macmon_version()?,
        sources: vec!["macmon".into(), "ioreg".into(), "sysctl".into()],
    })
}

// ---------------------------------------------------------------------------
// Summary

#[derive(Clone, Debug, Default)]
struct StatAcc {
    min: f64,
    max: f64,
    sum: f64,
    n: u64,
}

impl StatAcc {
    fn add(&mut self, x: f64) {
        if self.n == 0 {
            self.min = x;
            self.max = x;
        } else {
            self.min = self.min.min(x);
            self.max = self.max.max(x);
        }
        self.sum += x;
        self.n += 1;
    }

    fn stat(&self) -> Stat {
        if self.n == 0 {
            Stat {
                min: 0.0,
                mean: 0.0,
                max: 0.0,
            }
        } else {
            Stat {
                min: self.min,
                mean: self.sum / self.n as f64,
                max: self.max,
            }
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct SummaryAcc {
    samples: u64,
    cpu_active: StatAcc,
    gpu_active: StatAcc,
    gpu_device: StatAcc,
    mem_used: StatAcc,
    gpu_share: StatAcc,
    cpu_temp: StatAcc,
    gpu_temp: StatAcc,
    power_all: StatAcc,
}

impl SummaryAcc {
    pub fn add(&mut self, sample: &Sample) {
        let (Some(cpu), Some(gpu), Some(memory), Some(power), Some(thermal)) = (
            &sample.cpu,
            &sample.gpu,
            &sample.memory,
            &sample.power,
            &sample.thermal,
        ) else {
            return;
        };
        self.samples += 1;
        self.cpu_active.add(cpu.active_ratio);
        self.gpu_active.add(gpu.active_ratio);
        self.gpu_device.add(gpu.device_utilization_pct as f64);
        self.mem_used.add(memory.used_ratio);
        self.gpu_share.add(memory.gpu_share_of_total);
        self.cpu_temp.add(thermal.cpu_temp_c);
        self.gpu_temp.add(thermal.gpu_temp_c);
        self.power_all.add(power.all_w);
    }

    pub fn finish(&self, duration_ms: u64) -> Summary {
        Summary {
            samples: self.samples,
            duration_ms,
            cpu_active_ratio: self.cpu_active.stat(),
            gpu_active_ratio: self.gpu_active.stat(),
            gpu_device_utilization_pct: self.gpu_device.stat(),
            memory_used_ratio: self.mem_used.stat(),
            gpu_share_of_total: self.gpu_share.stat(),
            cpu_temp_c: self.cpu_temp.stat(),
            gpu_temp_c: self.gpu_temp.stat(),
            power_all_w: self.power_all.stat(),
        }
    }
}

// ---------------------------------------------------------------------------
// Storage

pub fn hardware_dir() -> PathBuf {
    PathBuf::from(
        std::env::var("PRIMNESS_HARDWARE_DIR").unwrap_or_else(|_| "data/hardware".to_string()),
    )
}

pub fn trace_path(run_id: &str) -> PathBuf {
    hardware_dir().join(format!("{run_id}.jsonl"))
}

pub struct HardwareStore {
    client: Client,
}

impl HardwareStore {
    pub fn from_env() -> Option<Self> {
        let url = std::env::var("PRIMNESS_DATABASE_URL").ok()?;
        if url.is_empty() {
            return None;
        }
        match Client::connect(&url, NoTls) {
            Ok(client) => Some(Self { client }),
            Err(err) => {
                eprintln!("hardware db: {err}");
                None
            }
        }
    }

    pub fn insert(&mut self, sample: &Sample, document: &Value) -> Result<(), String> {
        let ts_ms = i64::try_from(sample.ts_ms).map_err(|_| "ts_ms".to_string())?;
        self.client
            .execute(
                "INSERT INTO hardware_samples (schema_version, run_id, ts_ms, kind, sample)
                 VALUES ($1, $2, $3, $4, $5)
                 ON CONFLICT (run_id, ts_ms, kind) DO NOTHING",
                &[
                    &sample.schema_version,
                    &sample.run_id,
                    &ts_ms,
                    &sample.kind,
                    document,
                ],
            )
            .map(|_| ())
            .map_err(|err| err.to_string())
    }
}

struct Writer {
    registry: Registry,
    file: File,
    db: Option<HardwareStore>,
}

impl Writer {
    fn write(&mut self, sample: &Sample) -> Result<(), String> {
        let document = serde_json::to_value(sample).map_err(|err| err.to_string())?;
        self.registry.validate_value(V1, &document)?;
        let line = serde_json::to_string(&document).map_err(|err| err.to_string())?;
        writeln!(self.file, "{line}").map_err(|err| format!("hardware trace: {err}"))?;
        self.file
            .flush()
            .map_err(|err| format!("hardware trace: {err}"))?;
        if let Some(db) = self.db.as_mut() {
            if let Err(err) = db.insert(sample, &document) {
                eprintln!("hardware db: {err}");
            }
        }
        Ok(())
    }
}

/// Every record of one run's trace, in file order.
pub fn load_run(dir: &Path, run_id: &str) -> Vec<Sample> {
    let path = dir.join(format!("{run_id}.jsonl"));
    let Ok(file) = File::open(path) else {
        return Vec::new();
    };
    BufReader::new(file)
        .lines()
        .map_while(Result::ok)
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| serde_json::from_str::<Sample>(&line).ok())
        .collect()
}

// ---------------------------------------------------------------------------
// The sampler

/// Samples in a background thread until `stop` is called.
pub struct Sampler {
    stop: Arc<AtomicBool>,
    child: Arc<Mutex<Child>>,
    handle: JoinHandle<Result<Summary, String>>,
    pub path: PathBuf,
}

impl Sampler {
    pub fn start(run_id: &str, interval_ms: u64) -> Result<Self, String> {
        let interval_ms = interval_ms.max(100);
        let registry = Registry::from_env()?;
        let machine = machine_metadata()?;
        let dir = hardware_dir();
        fs::create_dir_all(&dir).map_err(|err| format!("{}: {err}", dir.display()))?;
        let path = dir.join(format!("{run_id}.jsonl"));
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|err| format!("{}: {err}", path.display()))?;
        let mut writer = Writer {
            registry,
            file,
            db: HardwareStore::from_env(),
        };
        let mut child = Command::new("macmon")
            .args(["pipe", "-i", &interval_ms.to_string()])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|err| format!("macmon: {err}"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "macmon stdout".to_string())?;
        if let Some(stderr) = child.stderr.take() {
            // macmon's own messages go to our stderr, prefixed, so an early
            // exit is visible in the run log.
            std::thread::spawn(move || {
                for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                    eprintln!("macmon: {line}");
                }
            });
        }
        let mut start = Sample::bare(run_id, "start", interval_ms);
        start.machine = Some(machine);
        writer.write(&start)?;

        let stop = Arc::new(AtomicBool::new(false));
        let child = Arc::new(Mutex::new(child));
        let (stop_flag, run_id_owned) = (stop.clone(), run_id.to_string());
        let handle = std::thread::spawn(move || -> Result<Summary, String> {
            let started = Instant::now();
            let mut acc = SummaryAcc::default();
            for line in BufReader::new(stdout).lines() {
                if stop_flag.load(Ordering::SeqCst) {
                    break;
                }
                let Ok(line) = line else { break };
                if line.trim().is_empty() {
                    continue;
                }
                let sample = parse_macmon(&line).and_then(|m| {
                    read_ioreg().map(|g| compose_sample(&run_id_owned, interval_ms, &m, &g))
                });
                match sample {
                    Ok(sample) => {
                        acc.add(&sample);
                        if let Err(err) = writer.write(&sample) {
                            eprintln!("hw-sample: {err}");
                        }
                    }
                    Err(err) => eprintln!("hw-sample: {err}"),
                }
            }
            if !stop_flag.load(Ordering::SeqCst) {
                eprintln!(
                    "hw-sample: macmon stopped on its own after {} ms",
                    started.elapsed().as_millis()
                );
            }
            let summary = acc.finish(started.elapsed().as_millis() as u64);
            let mut finish = Sample::bare(&run_id_owned, "finish", interval_ms);
            finish.summary = Some(summary.clone());
            writer.write(&finish)?;
            Ok(summary)
        });
        Ok(Self {
            stop,
            child,
            handle,
            path,
        })
    }

    /// Stop sampling, write the finish record, and return the summary.
    pub fn stop(self) -> Result<Summary, String> {
        self.stop.store(true, Ordering::SeqCst);
        if let Ok(mut child) = self.child.lock() {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.handle
            .join()
            .map_err(|_| "hardware sampler thread panicked".to_string())?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MACMON: &str = r#"{"all_power":2.3,"ane_power":0.0,"cpu_active_ratio":0.0422,"cpu_power":2.18,"cpu_scaled_ratio":0.022,"cpu_tiers":[{"active_ratio":0.0633,"cores":[],"freq_mhz":1344,"label":"P","scaled_ratio":0.033},{"active_ratio":0.0,"cores":[],"freq_mhz":1308,"label":"S","scaled_ratio":0.0}],"fans":[{"max_rpm":5349,"name":"fan0","rpm":1471}],"gpu_active_ratio":0.0963,"gpu_freq_mhz":338,"gpu_power":0.12,"gpu_scaled_ratio":0.02,"memory":{"ram_total":137438953472,"ram_usage":85836840960,"swap_total":0,"swap_usage":0},"ram_power":0.59,"sys_power":14.04,"temp":{"cpu_temp_avg":48.62,"gpu_temp_avg":48.62},"timestamp":"2026-11-03T22:31:59+00:00"}"#;

    const IOREG: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<plist version="1.0"><array><dict>
<key>IOClass</key><string>AGXAcceleratorG17X</string>
<key>gpu-core-count</key><integer>40</integer>
<key>PerformanceStatistics</key>
<dict>
  <key>Alloc system memory</key>
  <integer>21225373696</integer>
  <key>Device Utilization %</key>
  <integer>3</integer>
  <key>In use system memory</key>
  <integer>17700585472</integer>
  <key>In use system memory (driver)</key>
  <integer>0</integer>
  <key>Renderer Utilization %</key>
  <integer>3</integer>
  <key>Tiler Utilization %</key>
  <integer>2</integer>
</dict></dict></array></plist>"#;

    #[test]
    fn parses_macmon_and_ioreg() {
        let m = parse_macmon(MACMON).unwrap();
        assert_eq!(m.cpu.tiers.len(), 2);
        assert_eq!(m.cpu.tiers[0].label, "P");
        assert_eq!(m.ram_total, 137_438_953_472);
        assert_eq!(m.thermal.fans[0].rpm, 1471);
        assert!((m.thermal.cpu_temp_c - 48.62).abs() < 1e-9);
        let g = parse_ioreg_plist(IOREG).unwrap();
        assert_eq!(g.device_utilization_pct, 3);
        assert_eq!(g.in_use_bytes, 17_700_585_472);
        assert_eq!(g.alloc_bytes, 21_225_373_696);
        assert_eq!(g.core_count, Some(40));
        assert!(parse_ioreg_plist("<plist/>").is_err());
        assert!(parse_macmon("{}").is_err());
    }

    #[test]
    fn composed_sample_validates_and_derives_memory_split() {
        let m = parse_macmon(MACMON).unwrap();
        let g = parse_ioreg_plist(IOREG).unwrap();
        let sample = compose_sample("run-a", 1000, &m, &g);
        let memory = sample.memory.as_ref().unwrap();
        assert_eq!(memory.cpu_used_bytes, 85_836_840_960 - 17_700_585_472);
        assert!((memory.gpu_share_of_total - 17_700_585_472.0 / 137_438_953_472.0).abs() < 1e-12);
        assert!(memory.gpu_share_of_used > 0.2 && memory.gpu_share_of_used < 0.21);
        let registry = Registry::from_env().unwrap();
        registry
            .validate_value(V1, &serde_json::to_value(&sample).unwrap())
            .unwrap();
        let text = serde_json::to_string(&sample).unwrap();
        let back: Sample = serde_json::from_str(&text).unwrap();
        assert_eq!(back, sample);
    }

    #[test]
    fn start_requires_machine_and_finish_requires_summary() {
        let registry = Registry::from_env().unwrap();
        let start = Sample::bare("run-a", "start", 1000);
        let err = registry
            .validate_value(V1, &serde_json::to_value(&start).unwrap())
            .unwrap_err();
        assert!(err.contains("machine"), "{err}");
        let m = parse_macmon(MACMON).unwrap();
        let g = parse_ioreg_plist(IOREG).unwrap();
        let mut acc = SummaryAcc::default();
        acc.add(&compose_sample("run-a", 1000, &m, &g));
        acc.add(&compose_sample("run-a", 1000, &m, &g));
        let summary = acc.finish(2000);
        assert_eq!(summary.samples, 2);
        assert_eq!(summary.gpu_device_utilization_pct.max, 3.0);
        let mut finish = Sample::bare("run-a", "finish", 1000);
        finish.summary = Some(summary);
        registry
            .validate_value(V1, &serde_json::to_value(&finish).unwrap())
            .unwrap();
        let mut bad = Sample::bare("run-a", "finish", 1000);
        bad.interval_ms = 50;
        assert!(registry
            .validate_value(V1, &serde_json::to_value(&bad).unwrap())
            .is_err());
    }

    #[test]
    fn loads_a_trace_from_jsonl() {
        let dir = std::env::temp_dir().join(format!("primness-hw-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let m = parse_macmon(MACMON).unwrap();
        let g = parse_ioreg_plist(IOREG).unwrap();
        let sample = compose_sample("run-a", 1000, &m, &g);
        let mut file = File::create(dir.join("run-a.jsonl")).unwrap();
        writeln!(file, "{}", serde_json::to_string(&sample).unwrap()).unwrap();
        writeln!(file, "{{\"broken\":").unwrap();
        let loaded = load_run(&dir, "run-a");
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].kind, "sample");
        assert!(load_run(&dir, "missing").is_empty());
        let _ = fs::remove_dir_all(&dir);
    }
}
