// d1_generate_instances: produce candidate hard balanced semiprimes for the D1 protocol.
// The products below are a scaled-down sketch, not the 40–80 digit test set.

use num_bigint::BigUint;
use primness::metrics::{self, Event, Sink};
use std::env;
use std::time::Instant;

fn main() {
    println!("D1 instance generator");
    let _target: usize = env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(40);

    let pairs = [
        (1000000007u64, 1000000009u64),
        (1000000021u64, 1000000033u64),
        (1000000087u64, 1000000093u64),
    ];

    let started = Instant::now();
    let run_id = format!("d1g-{}-{}", std::process::id(), metrics::now_ms());
    let mut sink = Sink::from_env(&run_id);
    sink.emit(&Event {
        ts_ms: metrics::now_ms(),
        run_id: run_id.clone(),
        source: "d1-generate".into(),
        kind: "start".into(),
        scale: "toy".into(),
        instance: "placeholder-pairs".into(),
        b: 0,
        evaluated: 0,
        smooth: 0,
        elapsed_ms: 0,
        rate_per_s: 0.0,
        residues: String::new(),
    });

    println!("ID | Digits | N");
    for (i, (p, q)) in pairs.iter().enumerate() {
        let bp = BigUint::from(*p);
        let bq = BigUint::from(*q);
        let n = &bp * &bq;
        let digits = n.to_string().len();
        let id = format!("d1-{:02}", i + 1);
        println!("{id} | {digits} | {n}");
        let elapsed_ms = started.elapsed().as_millis() as u64;
        let produced = (i as u64) + 1;
        let rate = if elapsed_ms == 0 {
            0.0
        } else {
            produced as f64 / (elapsed_ms as f64 / 1000.0)
        };
        sink.emit(&Event {
            ts_ms: metrics::now_ms(),
            run_id: run_id.clone(),
            source: "d1-generate".into(),
            kind: "sample".into(),
            scale: "toy".into(),
            instance: id,
            b: 0,
            evaluated: produced,
            smooth: 0,
            elapsed_ms,
            rate_per_s: rate,
            residues: String::new(),
        });
    }
    let elapsed_ms = started.elapsed().as_millis() as u64;
    sink.emit(&Event {
        ts_ms: metrics::now_ms(),
        run_id,
        source: "d1-generate".into(),
        kind: "finish".into(),
        scale: "toy".into(),
        instance: "placeholder-pairs".into(),
        b: 0,
        evaluated: pairs.len() as u64,
        smooth: 0,
        elapsed_ms,
        rate_per_s: if elapsed_ms == 0 {
            0.0
        } else {
            pairs.len() as f64 / (elapsed_ms as f64 / 1000.0)
        },
        residues: String::new(),
    });
    println!("(Next: replace with properly generated and proven primes in the 40-80 digit range)");
}
