use leptos::prelude::*;
use primness::metrics::{density, RunView, Snapshot};
use primness::schema::V1;

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

pub fn render(snapshot: Snapshot, selected: &str) -> String {
    let selected = selected.to_string();
    let body =
        Owner::new().with(|| view! { <Page snapshot=snapshot selected=selected/> }.to_html());
    format!("<!DOCTYPE html>\n{body}")
}

#[component]
fn Page(snapshot: Snapshot, selected: String) -> impl IntoView {
    let model = PageModel::from_snapshot(&snapshot, &selected);
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
        let html = render(Snapshot { runs: vec![] }, "");
        assert!(html.contains("Primness bench"));
        assert!(html.contains(V1));
        assert!(html.contains("No measurements yet."));
    }

    #[test]
    fn run_page_shows_rate_density_and_schema() {
        let html = render(Snapshot { runs: vec![run()] }, "run-a");
        assert!(html.contains("800,000 values a second"));
        assert!(html.contains("#f2c14e"));
        assert!(html.contains("#6ea8c9"));
        assert!(html.contains(V1));
        assert!(html.contains("toy"));
        assert!(html.contains("362,000/s"));
    }
}
