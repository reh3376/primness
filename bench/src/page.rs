use leptos::prelude::*;
use primness::hardware::{self, Sample};
use primness::metrics::{density, RunView, Snapshot};
use primness::schema::V1;
use std::path::Path;

const CSS: &str = r#"
  :root {
    --ink: #1c2430;
    --paper: #d6deea;
    --dim: #8aa0b4;
    --gold: #f2c14e;
    --blue: #6ea8c9;
    --mesh: #243140;
    --line: #3a4a5c;
    --focus: #f2c14e;
  }
  * { box-sizing: border-box; }
  html, body { margin: 0; background: var(--ink); color: var(--paper); }
  body {
    font-family: "Avenir Next", Futura, "Trebuchet MS", sans-serif;
    font-weight: 400;
    line-height: 1.45;
  }
  main { max-width: 920px; margin: 0; padding: 28px 32px 72px; }
  header { display: flex; justify-content: space-between; align-items: baseline; gap: 16px; }
  h1 {
    margin: 0;
    font-family: Futura, "Avenir Next Condensed", "Avenir Next", sans-serif;
    font-weight: 500;
    font-size: 2.4rem;
    letter-spacing: -0.03em;
    line-height: 1;
  }
  .state { font-variant-numeric: tabular-nums; color: var(--dim); }
  h2 { margin: 30px 0 6px; font-size: 1.25rem; font-weight: 500; letter-spacing: -0.01em; }
  .state.collecting { color: var(--gold); }
  .reading { margin: 22px 0 8px; font-size: 1.35rem; line-height: 1.3; max-width: 38rem; }
  .scale, .schema, .legend { margin: 0 0 12px; color: var(--dim); max-width: 40rem; }
  .strip-wrap { background: var(--mesh); margin: 8px -32px 0; padding: 16px 32px 8px; }
  svg { display: block; width: 100%; }
  nav { margin: 18px 0 0; display: flex; flex-wrap: wrap; gap: 8px 18px; }
  a.run { color: var(--dim); text-decoration: none; }
  a.run.on { color: var(--paper); }
  a.run:focus-visible { outline: 2px solid var(--focus); outline-offset: 2px; }
  table {
    width: 100%;
    border-collapse: collapse;
    margin-top: 22px;
    font-variant-numeric: tabular-nums;
  }
  th, td { text-align: left; padding: 7px 10px 7px 0; vertical-align: baseline; }
  th { color: var(--dim); font-weight: 500; }
  td { border-top: 1px solid var(--line); }
  code {
    font-family: "SF Mono", ui-monospace, Menlo, monospace;
    font-size: 0.92rem;
    color: var(--gold);
  }
  @media (max-width: 640px) {
    main { padding: 20px 16px 48px; }
    .strip-wrap { margin: 8px -16px 0; padding: 12px 16px 8px; }
    h1 { font-size: 1.8rem; }
    table { font-size: 0.92rem; }
  }
  @media (prefers-reduced-motion: reduce) {
    * { scroll-behavior: auto; }
  }
"#;

pub fn render(snapshot: Snapshot, selected: &str, hardware_dir: &Path) -> String {
    let selected = selected.to_string();
    let run_id = snapshot
        .runs
        .iter()
        .find(|run| run.run_id == selected)
        .or_else(|| snapshot.runs.first())
        .map(|run| run.run_id.clone());
    let hardware = HardwareModel::load(hardware_dir, run_id.as_deref());
    let body = Owner::new().with(|| {
        view! { <Page snapshot=snapshot selected=selected hardware=hardware/> }.to_html()
    });
    format!("<!DOCTYPE html>\n{body}")
}

#[component]
fn Page(snapshot: Snapshot, selected: String, hardware: HardwareModel) -> impl IntoView {
    let model = PageModel::from_snapshot(&snapshot, &selected);
    let hw_machine = hardware.machine;
    let hw_util_legend = hardware.util_legend;
    let hw_thermal_legend = hardware.thermal_legend;
    let hw_util_lines = hardware
        .util_series
        .into_iter()
        .map(|(points, color)| {
            view! { <polyline points=points fill="none" stroke=color stroke-width="2"></polyline> }
        })
        .collect_view();
    let hw_thermal_lines = hardware
        .thermal_series
        .into_iter()
        .map(|(points, color)| {
            view! { <polyline points=points fill="none" stroke=color stroke-width="2"></polyline> }
        })
        .collect_view();
    let hw_rows = hardware
        .rows
        .into_iter()
        .map(|cols| {
            let cells = cols
                .into_iter()
                .map(|text| view! { <td>{text}</td> })
                .collect_view();
            view! { <tr>{cells}</tr> }
        })
        .collect_view();
    let state_class = model.state_class.to_string();
    let state = model.state;
    let reading = model.reading;
    let scale = model.scale;
    let schema = model.schema;
    let legend = model.legend;
    let strip_label = model.strip_label;
    let chart_label = model.chart_label;
    let cells = model
        .cells
        .into_iter()
        .map(|(x, width, fill)| {
            view! {
                <rect x=x y="26" width=width height="36" fill=fill></rect>
            }
        })
        .collect_view();
    let runs = model
        .runs
        .into_iter()
        .map(|(href, label, class_name)| {
            view! { <a href=href class=class_name>{label}</a> }
        })
        .collect_view();
    let rows = model
        .rows
        .into_iter()
        .map(|cols| {
            let cells = cols
                .into_iter()
                .map(|text| view! { <td>{text}</td> })
                .collect_view();
            view! { <tr>{cells}</tr> }
        })
        .collect_view();
    let density_points = model.density_points;
    let rate_points = model.rate_points;

    view! {
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <meta http-equiv="refresh" content="2"/>
                <title>Primness bench</title>
                <style>{CSS}</style>
            </head>
            <body>
                <main>
                    <header>
                        <h1>Primness bench</h1>
                        <p class=state_class>{state}</p>
                    </header>
                    <p class="reading">{reading}</p>
                    <p class="scale">{scale}</p>
                    <p class="schema">{schema}</p>
                    <div class="strip-wrap">
                        <svg viewBox="0 0 880 88" role="img" aria-label=strip_label>
                            {cells}
                        </svg>
                    </div>
                    <svg viewBox="0 0 880 220" role="img" aria-label=chart_label>
                        <line x1="8" y1="12" x2="872" y2="12" stroke="#3a4a5c" stroke-width="1"></line>
                        <line x1="8" y1="62.5" x2="872" y2="62.5" stroke="#3a4a5c" stroke-width="1"></line>
                        <line x1="8" y1="113" x2="872" y2="113" stroke="#3a4a5c" stroke-width="1"></line>
                        <line x1="8" y1="163.5" x2="872" y2="163.5" stroke="#3a4a5c" stroke-width="1"></line>
                        <line x1="8" y1="202" x2="872" y2="202" stroke="#3a4a5c" stroke-width="1"></line>
                        <polyline points=density_points fill="none" stroke="#f2c14e" stroke-width="2"></polyline>
                        <polyline points=rate_points fill="none" stroke="#6ea8c9" stroke-width="2"></polyline>
                    </svg>
                    <p class="legend">{legend}</p>
                    <nav>{runs}</nav>
                    <table>
                        <thead>
                            <tr>
                                <th>Bound</th>
                                <th>Evaluated</th>
                                <th>Smooth</th>
                                <th>Density</th>
                                <th>Chunk rate</th>
                                <th>Elapsed</th>
                            </tr>
                        </thead>
                        <tbody>{rows}</tbody>
                    </table>
                    <h2>"Hardware"</h2>
                    <p class="scale">{hw_machine}</p>
                    <svg viewBox="0 0 880 220" role="img" aria-label="Utilization and memory over the run">
                        <line x1="8" y1="12" x2="872" y2="12" stroke="#3a4a5c" stroke-width="1"></line>
                        <line x1="8" y1="62.5" x2="872" y2="62.5" stroke="#3a4a5c" stroke-width="1"></line>
                        <line x1="8" y1="113" x2="872" y2="113" stroke="#3a4a5c" stroke-width="1"></line>
                        <line x1="8" y1="163.5" x2="872" y2="163.5" stroke="#3a4a5c" stroke-width="1"></line>
                        <line x1="8" y1="202" x2="872" y2="202" stroke="#3a4a5c" stroke-width="1"></line>
                        {hw_util_lines}
                    </svg>
                    <p class="legend">{hw_util_legend}</p>
                    <svg viewBox="0 0 880 220" role="img" aria-label="Temperature and power over the run">
                        <line x1="8" y1="12" x2="872" y2="12" stroke="#3a4a5c" stroke-width="1"></line>
                        <line x1="8" y1="113" x2="872" y2="113" stroke="#3a4a5c" stroke-width="1"></line>
                        <line x1="8" y1="202" x2="872" y2="202" stroke="#3a4a5c" stroke-width="1"></line>
                        {hw_thermal_lines}
                    </svg>
                    <p class="legend">{hw_thermal_legend}</p>
                    <table>
                        <thead>
                            <tr>
                                <th>Metric</th>
                                <th>Min</th>
                                <th>Mean</th>
                                <th>Max</th>
                            </tr>
                        </thead>
                        <tbody>{hw_rows}</tbody>
                    </table>
                </main>
            </body>
        </html>
    }
}

struct PageModel {
    state_class: &'static str,
    state: String,
    reading: String,
    scale: String,
    schema: String,
    legend: String,
    strip_label: String,
    chart_label: String,
    cells: Vec<(String, String, String)>,
    density_points: String,
    rate_points: String,
    runs: Vec<(String, String, String)>,
    rows: Vec<[String; 6]>,
}

impl PageModel {
    fn from_snapshot(snapshot: &Snapshot, selected: &str) -> Self {
        let run = snapshot
            .runs
            .iter()
            .find(|run| run.run_id == selected)
            .or_else(|| snapshot.runs.first());
        let runs = snapshot
            .runs
            .iter()
            .map(|candidate| {
                let on = run.is_some_and(|current| current.run_id == candidate.run_id);
                let class_name = if on {
                    "run on".to_string()
                } else {
                    "run".to_string()
                };
                let href = format!("/?run={}", url_encode(&candidate.run_id));
                let label = format!("{} {}", candidate.source, candidate.instance);
                (href, label, class_name)
            })
            .collect();
        let Some(run) = run else {
            return Self {
                state_class: "state",
                state: "Idle".into(),
                reading: "No measurements yet.".into(),
                scale: "From the repo root, run cargo run --bin d1-measure -- --region 80 examples/demo.poly".into(),
                schema: format!("Events must match schema {V1}."),
                legend: "Gold cells and the gold trace are smooth density. Blue cells are values that were not smooth. The blue trace is chunk rate.".into(),
                strip_label: "No polynomial values yet.".into(),
                chart_label: "No density or rate samples yet.".into(),
                cells: Vec::new(),
                density_points: String::new(),
                rate_points: String::new(),
                runs,
                rows: Vec::new(),
            };
        };
        let live = run.last_kind != "finish" && now_ms().saturating_sub(run.last_ms) < 4_000;
        let (density_points, rate_points) = chart_points(&run.samples);
        Self {
            state_class: if live { "state collecting" } else { "state" },
            state: if live { "Collecting".into() } else { "Finished".into() },
            reading: format!(
                "{} values a second across this run. {} of {} are smooth at bound {}.",
                fmt_int(run.rate_per_s.round() as u64),
                fmt_int(run.smooth),
                fmt_int(run.evaluated),
                fmt_int(run.b)
            ),
            scale: scale_sentence(run),
            schema: format!(
                "Schema {}.",
                if run.schema_version.is_empty() { V1 } else { run.schema_version.as_str() }
            ),
            legend: "Gold cells and the gold trace are smooth density. Blue cells are values that were not smooth. The blue trace is chunk rate, scaled to the fastest chunk in this run.".into(),
            strip_label: "Recent polynomial values. Gold cells are smooth. Blue cells are not.".into(),
            chart_label: "Density in gold and chunk rate in blue across the run.".into(),
            cells: strip_cells(&run.residues),
            density_points,
            rate_points,
            runs,
            rows: bound_rows(run),
        }
    }
}

fn scale_sentence(run: &RunView) -> String {
    let lead = if run.scale == "full" {
        format!("Full-size run on {}.", run.instance)
    } else {
        format!(
            "Scaled-down {} run on {}. These figures are not the 40–80 digit claim.",
            run.scale, run.instance
        )
    };
    format!(
        "{lead} Elapsed {}. Density {}.",
        fmt_elapsed(run.elapsed_ms),
        fmt_density(density(run.smooth, run.evaluated))
    )
}

fn bound_rows(run: &RunView) -> Vec<[String; 6]> {
    let mut by_bound: Vec<&primness::metrics::SamplePoint> = Vec::new();
    for sample in &run.samples {
        if by_bound.last().is_some_and(|prev| prev.b == sample.b) {
            *by_bound.last_mut().expect("just checked") = sample;
        } else {
            by_bound.push(sample);
        }
    }
    by_bound.reverse();
    by_bound
        .into_iter()
        .map(|sample| {
            [
                fmt_int(sample.b),
                fmt_int(sample.evaluated),
                fmt_int(sample.smooth),
                fmt_density(sample.density),
                format!("{}/s", fmt_int(sample.rate_per_s.round() as u64)),
                fmt_elapsed(sample.elapsed_ms),
            ]
        })
        .collect()
}

fn strip_cells(residues: &str) -> Vec<(String, String, String)> {
    let n = residues.len().max(48);
    let gap = 3.0;
    let cw = ((880.0 - gap * (n as f64 + 1.0)) / n as f64).max(2.0);
    (0..n)
        .map(|i| {
            let bit = residues.chars().nth(i);
            let fill = match bit {
                Some('1') => "#f2c14e",
                Some('0') => "#6ea8c9",
                _ => "#15202b",
            };
            let x = gap + i as f64 * (cw + gap);
            (format!("{x:.1}"), format!("{cw:.1}"), fill.to_string())
        })
        .collect()
}

fn chart_points(samples: &[primness::metrics::SamplePoint]) -> (String, String) {
    if samples.is_empty() {
        return (String::new(), String::new());
    }
    let max_rate = samples
        .iter()
        .map(|sample| sample.rate_per_s)
        .fold(1.0_f64, f64::max);
    let plot_w = 880.0 - 16.0;
    let plot_h = 220.0 - 12.0 - 18.0;
    let mut density_points = String::new();
    let mut rate_points = String::new();
    for (i, sample) in samples.iter().enumerate() {
        let x = if samples.len() == 1 {
            8.0 + plot_w / 2.0
        } else {
            8.0 + plot_w * i as f64 / (samples.len() - 1) as f64
        };
        let density_y = 12.0 + plot_h * (1.0 - sample.density.clamp(0.0, 1.0));
        let rate_y = 12.0 + plot_h * (1.0 - (sample.rate_per_s / max_rate).clamp(0.0, 1.0));
        if i > 0 {
            density_points.push(' ');
            rate_points.push(' ');
        }
        density_points.push_str(&format!("{x:.1},{density_y:.1}"));
        rate_points.push_str(&format!("{x:.1},{rate_y:.1}"));
    }
    (density_points, rate_points)
}

fn fmt_int(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, ch) in digits.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out.chars().rev().collect()
}

fn fmt_density(value: f64) -> String {
    format!("{value:.3}")
}

fn fmt_elapsed(ms: u64) -> String {
    if ms < 1000 {
        format!("{ms} ms")
    } else {
        format!("{:.1} s", ms as f64 / 1000.0)
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn url_encode(value: &str) -> String {
    let mut out = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use primness::metrics::SamplePoint;

    fn run() -> RunView {
        RunView {
            run_id: "run-a".into(),
            schema_version: V1.into(),
            source: "d1-measure".into(),
            scale: "toy".into(),
            instance: "demo.poly".into(),
            started_ms: 1,
            last_ms: 2,
            last_kind: "finish".into(),
            evaluated: 3200,
            smooth: 802,
            elapsed_ms: 12,
            rate_per_s: 800_000.0,
            b: 32,
            residues: "10".into(),
            samples: vec![SamplePoint {
                ts_ms: 2,
                b: 32,
                evaluated: 3200,
                smooth: 802,
                elapsed_ms: 12,
                rate_per_s: 362_000.0,
                density: 802.0 / 3200.0,
            }],
        }
    }

    #[test]
    fn empty_page_names_the_schema() {
        let html = render(Snapshot { runs: vec![] }, "", Path::new("/nonexistent"));
        assert!(html.contains("Primness bench"));
        assert!(html.contains(V1));
        assert!(html.contains("No measurements yet."));
    }

    #[test]
    fn run_page_shows_rate_density_and_schema() {
        let html = render(Snapshot { runs: vec![run()] }, "run-a", Path::new("/nonexistent"));
        assert!(html.contains("800,000 values a second"));
        assert!(html.contains("#f2c14e"));
        assert!(html.contains("#6ea8c9"));
        assert!(html.contains(V1));
        assert!(html.contains("toy"));
        assert!(html.contains("362,000/s"));
    }
}

// ---------------------------------------------------------------------------
// Hardware trace of the selected run

const GOLD: &str = "#f2c14e";
const BLUE: &str = "#6ea8c9";
const GREEN: &str = "#8fd3a4";
const PAPER: &str = "#d6deea";
const LILAC: &str = "#c9a4f2";

struct HardwareModel {
    machine: String,
    util_legend: String,
    thermal_legend: String,
    util_series: Vec<(String, &'static str)>,
    thermal_series: Vec<(String, &'static str)>,
    rows: Vec<[String; 4]>,
}

impl HardwareModel {
    fn empty(machine: String) -> Self {
        Self {
            machine,
            util_legend: String::new(),
            thermal_legend: String::new(),
            util_series: Vec::new(),
            thermal_series: Vec::new(),
            rows: Vec::new(),
        }
    }

    fn load(dir: &Path, run_id: Option<&str>) -> Self {
        let Some(run_id) = run_id else {
            return Self::empty("No run selected.".into());
        };
        let records = hardware::load_run(dir, run_id);
        Self::from_records(run_id, &records)
    }

    fn from_records(run_id: &str, records: &[Sample]) -> Self {
        let samples: Vec<&Sample> = records.iter().filter(|s| s.kind == "sample").collect();
        if samples.is_empty() {
            return Self::empty(format!(
                "No hardware trace for {run_id}. Instrument runs record one as {}; see docs/hardware_telemetry.md.",
                hardware::V1
            ));
        }
        let machine = records.iter().find_map(|s| s.machine.as_ref());
        let interval = samples[0].interval_ms;
        let machine_line = match machine {
            Some(m) => format!(
                "{} ({}), {} CPU cores, {} GPU cores, {} GB unified memory, macOS {}. {} samples every {} ms, {} via {}.",
                m.chip,
                m.model,
                m.cpu_cores,
                m.gpu_cores,
                m.memory_total_bytes / (1 << 30),
                m.os_version,
                samples.len(),
                interval,
                hardware::V1,
                m.sources.join(", ")
            ),
            None => format!("{} samples every {} ms, {}.", samples.len(), interval, hardware::V1),
        };
        let pick = |f: &dyn Fn(&Sample) -> Option<f64>| -> Vec<f64> {
            samples.iter().map(|s| f(s).unwrap_or(0.0)).collect()
        };
        let cpu = pick(&|s| s.cpu.as_ref().map(|c| c.active_ratio));
        let gpu = pick(&|s| s.gpu.as_ref().map(|g| g.active_ratio));
        let gpu_dev = pick(&|s| s.gpu.as_ref().map(|g| g.device_utilization_pct as f64 / 100.0));
        let mem = pick(&|s| s.memory.as_ref().map(|m| m.used_ratio));
        let gpu_mem = pick(&|s| s.memory.as_ref().map(|m| m.gpu_share_of_total));
        let cpu_t = pick(&|s| s.thermal.as_ref().map(|t| t.cpu_temp_c));
        let gpu_t = pick(&|s| s.thermal.as_ref().map(|t| t.gpu_temp_c));
        let power = pick(&|s| s.power.as_ref().map(|p| p.all_w));
        let max_power = power.iter().cloned().fold(1.0_f64, f64::max);
        let util_series = vec![
            (polyline(&cpu, 1.0), GOLD),
            (polyline(&gpu, 1.0), BLUE),
            (polyline(&gpu_dev, 1.0), GREEN),
            (polyline(&mem, 1.0), PAPER),
            (polyline(&gpu_mem, 1.0), LILAC),
        ];
        let thermal_series = vec![
            (polyline(&cpu_t, 110.0), GOLD),
            (polyline(&gpu_t, 110.0), BLUE),
            (polyline(&power, max_power), PAPER),
        ];
        let stat = |name: &str, values: &[f64], unit: &str| -> [String; 4] {
            let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let mean = values.iter().sum::<f64>() / values.len() as f64;
            [
                name.to_string(),
                format!("{min:.2}{unit}"),
                format!("{mean:.2}{unit}"),
                format!("{max:.2}{unit}"),
            ]
        };
        let pct = |v: &[f64]| v.iter().map(|x| x * 100.0).collect::<Vec<_>>();
        let rows = vec![
            stat("CPU active", &pct(&cpu), "%"),
            stat("GPU active", &pct(&gpu), "%"),
            stat("GPU device utilization", &pct(&gpu_dev), "%"),
            stat("Memory used", &pct(&mem), "%"),
            stat("GPU share of unified memory", &pct(&gpu_mem), "%"),
            stat("CPU temperature", &cpu_t, " C"),
            stat("GPU temperature", &gpu_t, " C"),
            stat("Total power", &power, " W"),
        ];
        Self {
            machine: machine_line,
            util_legend: "Gold is CPU active ratio, blue is GPU active ratio, green is GPU device utilization, grey is memory used, lilac is the GPU share of unified memory. The axis runs 0 to 100%.".into(),
            thermal_legend: format!(
                "Gold is CPU temperature and blue is GPU temperature on a 0 to 110 C axis. Grey is total power scaled to its peak of {max_power:.1} W."
            ),
            util_series,
            thermal_series,
            rows,
        }
    }
}

/// SVG points for `values` scaled by `full` onto the 880 by 220 chart.
fn polyline(values: &[f64], full: f64) -> String {
    if values.is_empty() {
        return String::new();
    }
    let plot_w = 880.0 - 16.0;
    let plot_h = 220.0 - 12.0 - 18.0;
    let full = if full <= 0.0 { 1.0 } else { full };
    let mut out = String::new();
    for (i, value) in values.iter().enumerate() {
        let x = if values.len() == 1 {
            8.0 + plot_w / 2.0
        } else {
            8.0 + plot_w * i as f64 / (values.len() - 1) as f64
        };
        let y = 12.0 + plot_h * (1.0 - (value / full).clamp(0.0, 1.0));
        if i > 0 {
            out.push(' ');
        }
        out.push_str(&format!("{x:.1},{y:.1}"));
    }
    out
}

#[cfg(test)]
mod hardware_tests {
    use super::*;

    #[test]
    fn hardware_section_reads_a_trace_and_tolerates_none() {
        let dir = std::env::temp_dir().join(format!("primness-bench-hw-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let line = r#"{"schema_version":"hardware-sample/1","ts_ms":1,"run_id":"run-a","kind":"sample","interval_ms":1000,"cpu":{"active_ratio":0.5,"scaled_ratio":0.4,"power_w":10.0,"tiers":[]},"gpu":{"active_ratio":0.2,"scaled_ratio":0.1,"freq_mhz":338,"power_w":1.0,"device_utilization_pct":30,"renderer_utilization_pct":0,"tiler_utilization_pct":0,"memory_in_use_bytes":1000,"memory_alloc_bytes":2000},"memory":{"total_bytes":4000,"used_bytes":2000,"swap_used_bytes":0,"gpu_in_use_bytes":1000,"cpu_used_bytes":1000,"used_ratio":0.5,"gpu_share_of_total":0.25,"gpu_share_of_used":0.5},"power":{"all_w":12.0,"sys_w":20.0,"cpu_w":10.0,"gpu_w":1.0,"ane_w":0.0,"ram_w":1.0},"thermal":{"cpu_temp_c":55.0,"gpu_temp_c":50.0,"fans":[]}}"#;
        std::fs::write(dir.join("run-a.jsonl"), format!("{line}
{line}
")).unwrap();
        let model = HardwareModel::load(&dir, Some("run-a"));
        assert_eq!(model.util_series.len(), 5);
        assert_eq!(model.rows.len(), 8);
        assert_eq!(model.rows[0][2], "50.00%");
        assert!(model.machine.contains("2 samples"));
        let html = render(Snapshot { runs: vec![] }, "", &dir);
        assert!(html.contains("Hardware"));
        assert!(html.contains("No run selected."));
        let none = HardwareModel::load(&dir, Some("missing"));
        assert!(none.machine.contains("No hardware trace"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
