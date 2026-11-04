# Primness Agent Onboarding

**Purpose.** This is the zero-context handoff for a new coding agent. Read this file, then the linked documents. When an older note and `primness_roadmap.md` disagree, the roadmap **Current State** section is the later record. Do not invent a section, a formula, or a test that those files do not contain.

**Last updated:** 2026-11-03 19:28 EST

## What this project is

Primness is integer-factorization research. The target, in [primness_roadmap.md](../primness_roadmap.md), is a verified factorization or primality proof of \(N \ge 10^{200}\) in under one hour on a laptop with a GPU. The hard instances are balanced semiprimes with no special algebraic form. The preferred obstacle is D. The success criterion is that 200-digit hour. The 40–80 digit work is the first measurement, not the target.

The crate license field in [Cargo.toml](../Cargo.toml) is `MIT OR Apache-2.0`. There is no separate `LICENSE` file.

Active constraints live at the top of the roadmap: Rust for numerical kernels, a Python sidecar only where an ML library is required, no quantum hardware, and no specialized hardware unless obstacle H is forced. The roadmap names obstacles A–D in full. Obstacle D names sub-claims D1–D5 in one sentence. The full D1–D5 text and the choice of D1 are in [context_convo_notes/2026-11-03_1205_obstacle_D_subclaims.md](../context_convo_notes/2026-11-03_1205_obstacle_D_subclaims.md). The line for obstacles E–H is a placeholder. The roadmap has an `open_questions` section, and that list is empty. Add a question there only when it is actually open. The roadmap has no Decision Log. Do not invent one.

## Read these first

- [primness_roadmap.md](../primness_roadmap.md) — problem, active constraints, obstacles A–D, `## Current State`, and `## open_questions`.
- [context_convo_notes/current_state_history.md](../context_convo_notes/current_state_history.md) — earlier Current State snapshots. Newest `###` heading first. Leave every previous heading in place.
- [README.md](../README.md) — landing page. Update it in the same change as Current State.
- [CONTRIBUTING.md](../CONTRIBUTING.md) — branch and pull-request process.
- [docs/d1_falsifiable_prediction.md](d1_falsifiable_prediction.md) — the D1 claim, the 10-instance protocol, and the rejection rule (mean gap smaller than two standard deviations of measurement noise).
- [data/d1_test_instances.md](../data/d1_test_instances.md) — the ten proved semiprimes and their certificates.
- [docs/hardware.md](hardware.md) — names the machine as a Mac M5 Max and leaves installed RAM unconfirmed. Docker and the bench are allowed. An independent D1 measurement must meet the roadmap's one-hour criterion on this machine or be marked scaled-down.
- [docs/dashboard.md](dashboard.md) — local bench. It does not gate CI and it does not decide the D1 claim.
- [tools/README.md](../tools/README.md) — CADO-NFS, msieve, and YAFU as instruments, including which trees may be copied into this repository.
- [context_convo_notes/INDEX.md](../context_convo_notes/INDEX.md) — note index. Its table was last modified 2026-11-03 15:58 EST and lists only the 12:05, 12:51, and 15:58 notes plus the history file. The directory also contains note files from 13:06 through 14:44 that the table does not list. Read the directory. Do not invent summaries to fill the table.

## Current state a new agent must not miss

Current State is stamped 2026-11-03 19:28 EST. The scientific facts below are carried forward from the 16:48 EST snapshot:

- Development is on `primness_dev01`. Pull request 23 is merged.
- D1 is the starting sub-claim. d1-01 is measured at 2^14 to 2^20 (`data/results/d1-01.scaled-down.json`) and at the protocol's 2^20, 2^24, 2^28 (`data/results/d1-01.scaled-down.b28.json`) against raw, α-adjusted and local-density Dickman predictions. Murphy \(\alpha\) is defined in `docs/murphy_alpha.md` and computed in `src/alpha.rs`; do not read the instrument's α comment. All ten instances are measured (`data/results/summary.all-bounds.csv`, `scripts/d1_summary.py`). The residual after α changes sign with u, which is the baseline's finite-B error; do not read the split criterion verdict as a result. `data/results/patterns.scaled-down.csv` and `data/results/fixtures/` hold the structure ladder (x^n − 1 and five Galois-group quartics) on the same region.
- The ten semiprimes in `data/d1_test_instances.md` are unchanged. Digit counts are 40, 44, 48, 52, 56, 60, 64, 68, 72, and 80. Their bit lengths, from those decimals, are 131 through 264. d1-10 is 264 bits and \(\log_{10}(N)\) is about 79.37.
- The harness parses CADO and msieve polynomial files into `instrument-polynomial/1` and measures them (see the harness section below). `data/instruments/d1-01/cado.poly` is the only polynomial so far.
- CADO-NFS is built outside this tree at `~/external/cado-nfs` (commit 692ecb7). [tools/README.md](../tools/README.md) records it as LGPL-2.1, so its source stays outside this tree and the scripts call its binaries. The same file records msieve as public domain and records that YAFU’s README and repository root do not state a license, so YAFU is not copied in.
- msieve 1.54, in a checkout outside this tree, enters NFS polynomial selection only when `mp_bits(n)` is greater than 264. The degree-4 parameter table starts at \(\log_{10}(N) = 80\). Every proved semiprime here is at or below 264 bits, so that binary takes the quadratic-sieve path for them. The stock Apple clang build stops because `gettimeofday` is undeclared in `common/util.c`. The checkout builds after that file includes `<sys/time.h>`. The binary is not in this repository. Polynomial selection has not been run.
- [docs/d1_falsifiable_prediction.md](d1_falsifiable_prediction.md) step 2 still says to take one polynomial from CADO-NFS and one from msieve. That file was not revised when the size gate was recorded. Follow Current State for the next run: one NFS polynomial per instance from CADO-NFS, outside this tree, then the smoothness count and the Dickman comparison in the Rust harness. Do not rewrite the prediction into a new protocol.

The repository does not define an SNFS-structure test. The instance file says the factors are random primes of equal length.

## How to land a change

1. Work on `primness_dev01`, or a short-lived branch from it. Do not commit to `main`.
2. In the same change, update `## Current State` in `primness_roadmap.md` (first person), add a new `###` snapshot at the top of `context_convo_notes/current_state_history.md` without removing older headings, and update `README.md`. Bump `Last Modified` on the roadmap and the history file. The head stamp in Current State must be a `###` heading in the history file, and the previous stamp’s heading must remain.
3. The roadmap metadata line `- **Schema**: current-state/1` names [schemas/current-state/1.json](../schemas/current-state/1.json). The checker reads the requirements from that schema. Paths, the section heading, the stamp shape, and which metadata labels must change are `const` values. Enums are the closed options. A later contract is a new file.
4. Push `primness_dev01`. [.github/workflows/auto-pr.yml](../.github/workflows/auto-pr.yml) opens or refreshes one draft pull request as `github-actions[bot]` only after `sh scripts/check-current-state.sh origin/main HEAD` passes. Do not run `gh pr create`. A pull request opened from a developer shell is authored by that shell, and GitHub then blocks the owner’s self-approval.
5. The owner reviews, approves, and merges. Do not self-approve or merge.
6. After the merge, fast-forward `primness_dev01` to `origin/main` and push. Auto-PR exits without opening a pull request when `origin/main..HEAD` is empty.

[.github/workflows/ci.yml](../.github/workflows/ci.yml) runs `cargo check --workspace --all-targets` and `cargo test --workspace --all-targets` on every push. There is no `pull_request` trigger: the check recorded on the head commit is what the pull request shows. The main ruleset is to require that `Rust check and test` check; the owner adds the rule in the repository settings. The workspace members are the root crate and `bench`, so the dashboard binary is in that test run. [.github/workflows/sync-readme.yml](../.github/workflows/sync-readme.yml) only prints that README updates are still manual.

Before a push, run `sh scripts/check-current-state.sh origin/main HEAD`. The gate compares git commits, so the Current State edit has to be committed first.

## Code map

Workspace [Cargo.toml](../Cargo.toml), edition 2021, resolver 2. Release profile uses LTO and one codegen unit. The dashboard image build turns LTO off.

| Path | Role |
| --- | --- |
| [src/lib.rs](../src/lib.rs) | Modules `db`, `dickman`, `eval`, `hardware`, `metrics`, `poly`, `primes`, `schema`, `sieve`. |
| [src/dickman.rs](../src/dickman.rs) | Dickman–de Bruijn ρ(u): exact on [0, 2], Simpson grid above (4096 steps per unit to u = 40). Tests pin ρ(1) = 1, ρ(2) = 1 − ln 2, and the literature values at 3, 4, 5, 10. No α. |
| [src/eval.rs](../src/eval.rs) | Homogeneous BigInt evaluation F(a, b) = Σ c_i a^i b^(d−i), and `coprime`. |
| [src/sieve.rs](../src/sieve.rs) | Primes up to B by a bit sieve; `largest_prime_factor_if_smooth` by trial division (BigUint, then u128, then u64 phases). Used by `--method trial` and by α. |
| [src/alpha.rs](../src/alpha.rs) | Murphy α from [docs/murphy_alpha.md](murphy_alpha.md): expected p-adic valuation over coprime pairs, exact lifting. |
| [src/fp.rs](../src/fp.rs) | Roots of f mod p (gcd with x^p − x, Cantor–Zassenhaus). |
| [src/linesieve.rs](../src/linesieve.rs) | Line sieve with resieve verification ([docs/line_sieve.md](line_sieve.md)); the default smoothness method. |
| [scripts/d1_run_all.sh](../scripts/d1_run_all.sh), [scripts/d1_summary.py](../scripts/d1_summary.py), [scripts/d1_report.py](../scripts/d1_report.py) | All instances end to end; the criterion across instances; the per-run report. |
| [src/hardware.rs](../src/hardware.rs) | `hardware-sample/1` telemetry: macmon + ioreg + sysctl, JSONL trace, Postgres rows, summary. See [docs/hardware_telemetry.md](hardware_telemetry.md). |
| [src/bin/hw_sample.rs](../src/bin/hw_sample.rs) | Binary `hw-sample`: records a trace while a command runs or for N samples. |
| [scripts/cado-polyselect.sh](../scripts/cado-polyselect.sh), [scripts/cado-best-poly.py](../scripts/cado-best-poly.py) | CADO-NFS polynomial selection outside the tree, under the sampler; best candidate by MurphyE into `data/instruments/<id>/cado.poly`. |
| [src/primes.rs](../src/primes.rs) | Primality proofs for the instance generator. |
| [src/poly.rs](../src/poly.rs) | Parser for external polynomial files. Version constant `instrument-polynomial/1`. `MAX_DEGREE` is 12. |
| [src/schema.rs](../src/schema.rs) | Loads every `schemas/**/*.json`. Measurement events use `$id` `measurement-event/1`. |
| [src/metrics.rs](../src/metrics.rs) | Validates an event, then appends JSONL, optionally POSTs it, and inserts it when Postgres connects. |
| [src/db.rs](../src/db.rs) | Postgres store. Inserts use `ON CONFLICT (run_id, schema_version, kind, b, evaluated, elapsed_ms) DO NOTHING`. |
| [src/bin/d1_measure.rs](../src/bin/d1_measure.rs) | Binary name `d1-measure` (hyphen). |
| [src/bin/d1_generate_instances.rs](../src/bin/d1_generate_instances.rs) | Binary name `d1_generate_instances` (underscore). |
| [bench/](../bench/) | Package `primness-bench`, binary `primness-dashboard`. Leptos 0.8.23 server-rendered HTML. The measurement crate does not compile this UI. |

### Polynomial files

`d1-measure` tries `primness::poly::parse_instrument` before the toy path.

- Success runs the instrument path: F(a, b) and G(a, b) in BigInt over a in [−A, A], b in [1, B_MAX], gcd(a, b) = 1 (defaults 500 and 250; `--a-max`, `--b-max`), smoothness by trial division at every `--bounds` value (default 2^14, 2^16, 2^18, 2^20) from one largest-prime-factor pass, and the Dickman prediction as the mean of ρ(ln|F| / ln B). Events carry `scale` `scaled-down`. The result is a `d1-measurement/1` file (`--out`, default `data/results/<instance>-<run_id>.json`) with both sides, the gap, the binomial standard error, mean u, the implied size shift, and the hardware summary, plus CSV tables beside it (`tables` in the result: bounds, u_bins, rows, lpf_hist) and, with `--dump FILE`, one CSV row per value (gitignored). `scripts/d1_report.py <result.json>` writes `<stem>.report.md` and `<stem>.hardware.csv`. `--source` names the instrument only when the caller knows it; the file itself never does. `full` is not yet a reachable label: the protocol's region per digit length is not fixed.
- `Error::NotInstrument` falls through to the toy path. [examples/demo.poly](../examples/demo.poly) is three integer lines (`1`, `-1`, `1`) and takes that path.
- Any other parse error prints to stderr and exits 1. A file that has instrument keys plus a score key such as `MurphyE` is an error. The schema has no score field.

[schemas/instrument-polynomial/2.json](../schemas/instrument-polynomial/2.json) (version 1 stays beside it) is draft 2020-12 with `additionalProperties: false`. `format` is `cado` or `msieve-fb`. `source` is `cado-nfs`, `msieve`, `yafu`, or `unspecified`. `degree` is an integer from 1 through 12 in version 2 (version 1 stopped at 6). The D1 claim still wants an optimized degree-4 or degree-5 polynomial. Parsing a degree outside that pair does not make it the claim. Coefficients are decimal strings. `skew` is optional text. Synthetic fixtures, not polynomials for the ten instances: [examples/instrument_cado.poly](../examples/instrument_cado.poly), [examples/instrument_msieve.fb](../examples/instrument_msieve.fb).

### Toy harness

The toy path prints `scale: toy (not the 40–80 digit protocol)`. It evaluates an `i64` coefficient list in `i128`, clamps the absolute value into `u64`, and trial-divides by every integer up to `B`. The default region is `a` in 1..=10, `b` in 1..=5, and `B` in 10, 30, 100. `--region N` clamps `N` to 2..=2000 and uses `B` in 32, 128, 512. Events use `schema_version` `measurement-event/1` and `scale` `toy`. That is the schema enum value. It is not `scaled-down` and it is not `full`.

An empty coefficient list becomes `1, -1, 1`. Do not treat that fallback as a selected polynomial.

The prediction’s example smoothness bounds are \(2^{20}\), \(2^{24}\), and \(2^{28}\). The toy bounds are not those.

### Instances and proofs

`d1_generate_instances` requires `--write <path>`. It draws from `/dev/urandom` and replaces the file. Do not point it at `data/d1_test_instances.md`. A new draw would replace the committed semiprimes. `primes::tests::committed_instance_file_rechecks` rechecks that file.

Proofs in [src/primes.rs](../src/primes.rs):

- Deterministic Miller-Rabin only for a prime strictly below \(\psi_{13}\) = `3317044064679887385961981`, using the first 13 primes as bases (2 through 41). The generator uses this path for primes of at most 24 digits.
- A larger prime uses one Pocklington step. The certificate line is `id role decimal mr` or `id role decimal pocklington F witness`.
- d1-01 through d1-03 use Miller-Rabin on the factors. d1-04 through d1-10 use Pocklington. Witnesses in the committed file are 2.

Do not call a random-base Miller-Rabin test, an OpenSSL prime, or a Baillie-PSW test a proof. The proof code does not call sympy, gmpy2, or PARI. Python in this repository is `scripts/check_current_state.py` and the schema upsert inside `scripts/apply-migrations.sh`. There is no ML sidecar.

### Schemas and collection

Three schema files:

- [schemas/measurement-event/1.json](../schemas/measurement-event/1.json) — `$id` `measurement-event/1`. `kind` is `start`, `sample`, or `finish`. `scale` is `toy`, `scaled-down`, or `full`.
- [schemas/instrument-polynomial/1.json](../schemas/instrument-polynomial/1.json) — not a measurement event.
- [schemas/current-state/1.json](../schemas/current-state/1.json) — the Auto-PR gate, not a measurement event.

`Registry::load` walks every JSON file under `schemas/`. A new version is a new file. Old versions stay. `cargo test` loads them; an invalid document fails the schema tests.

`Sink::emit` drops an event when the registry is missing or the JSON does not match. That skip does not stop the numeric loop. When the event is valid, the sink appends a JSONL line under `PRIMNESS_METRICS_DIR` (default `data/metrics`), POSTs it when `PRIMNESS_METRICS_URL` is set, and inserts it when `PRIMNESS_DATABASE_URL` connects. `data/metrics/*.jsonl` is gitignored. The bench reads Postgres when that URL is set and the query works. If the query fails, it reads the JSONL directory. It does not merge the two.

Leave `PRIMNESS_METRICS_URL` unset in a process that already has `PRIMNESS_DATABASE_URL`. Otherwise the same event is inserted and posted. Do not source `.env`. It holds `GITHUB_TOKEN` and the Postgres password. Read the one key you need. The key names are in [.env.example](../.env.example). That example also lists an Ollama host and `CHROME_CONTROL=unavailable`. This repository has no Ollama client and no Chrome client.

Postgres is `postgres:16-alpine`, container `primness-db`, volume `primness-pg`, database and user `primness`, host port `127.0.0.1:5455`, in-network host `db:5432`. `scripts/apply-migrations.sh` applies `db/migrations/*.sql` once, then upserts every schema `$id` into `schema_documents`. Local access is `docker exec primness-db psql -U primness -d primness`. Compose fails if `POSTGRES_PASSWORD` is unset.

The bench container is `primness-bench`, image built from [Dockerfile](../Dockerfile), which copies `schemas/` at build time. A schema file added later is inside the container only after `scripts/up-dashboard.sh` rebuilds the image. The host migration script registers the host `schemas/` tree without that rebuild. The page is `http://127.0.0.1:9477/` by default, loopback only, and it reloads every 2 seconds. The binary binds `PRIMNESS_DASHBOARD_HOST`, default `127.0.0.1`; the container sets `0.0.0.0` behind its loopback port mapping. `scripts/up-dashboard.sh` refuses a port that another process holds.

## Immediate next work

1. Calibrate the baseline in this harness: sample random integers with the same size distribution as the values of each run, count their smoothness at the same bounds (trial division is fine at the sample sizes needed), and compare the count against ρ, the α-shifted ρ and the local-density form by u. Record the calibration as its own schema and note.
2. State in [docs/d1_falsifiable_prediction.md](d1_falsifiable_prediction.md) whether the criterion's mean gap is absolute or relative to the prediction, with a reason, then re-run `scripts/d1_summary.py` against the calibrated baseline.
3. Test the A4 lever on one instance: select polynomials with square discriminant and measure the residual after α against the generic selection.
4. Fix the protocol's region per digit length so a run can be labeled `full`.

Do not start all ten instances at \(B = 2^{28}\) before one small external polynomial has been parsed. Do not run msieve NFS polynomial selection on these ten numbers with the 1.54 binary described above and expect an NFS polynomial.

## Handoff checklist

- [ ] Read `## Current State` and `## open_questions` in `primness_roadmap.md`, and the newest heading in `context_convo_notes/current_state_history.md`.
- [ ] Read `docs/d1_falsifiable_prediction.md`, `data/d1_test_instances.md`, `docs/hardware.md`, `docs/dashboard.md`, and `tools/README.md`.
- [ ] Confirm the local branch is `primness_dev01` and matches the remote you intend to continue.
- [ ] Treat `context_convo_notes/INDEX.md` as incomplete. List the directory.
- [ ] Do not regenerate `data/d1_test_instances.md`.
- [ ] Push only through the Auto-PR path above.
