# H4: the laptop's one-hour reach, measured end to end

`docs/h4_falsifiable_prediction.md` states the claim, the protocol and the criterion. It was committed (4660e12) before any factorization of 100 digits or more ran on this machine. This document records the runs and the result.

## 1. Method

**Instances.** RSA-100, RSA-110, RSA-120 and RSA-130, each the product of its published factors (`scripts/h4_summary.py --n`). CADO-NFS's reference polynomials `c100.poly` to `c130.poly` were found for these numbers. The runs do not use those polynomials.

**Runs.** `scripts/h4_reach.sh` runs each instance once, in ascending order, in an empty work directory under `hw-sample`:

```
cado-nfs.py N server.address=localhost -t 18 --workdir data/dumps/h4/rsa<d>
```

CADO-NFS (692ecb7) picks its parameter file for the size (`params.c100` to `params.c130`, sha256 in `data/results/h4/`), and nothing in it is overridden. The driver ran under Python 3.14.3 with Flask 3.1.3, the same versions as CADO-NFS's own `cado-nfs.venv`. `server.address=localhost` keeps CADO-NFS's work server on 127.0.0.1, where by default it listens on every interface. That is a network setting and changes no factoring parameter. The protocol stopped after RSA-130, the first run over one hour.

**Times.** The wall time is BSD `time` of the whole `cado-nfs.py` process, polynomial selection to square root. The log's timestamps give the phases. CADO-NFS's own CPU total gives the cores in use, since `time` does not see the sieving clients. `scripts/h4_summary.py` writes `data/results/h4_reach.{md,csv}` and `data/results/h4/hardware_by_phase.csv`.

**Check.** Every run printed exactly the two published factors.

**Conditions.** AC power, automatic power mode, not Low Power Mode. The machine was not reserved for the runs. The GPU, which CADO-NFS does not use, was active 11 to 13 percent of the time in every run, in short bursts, so other work shared the machine at about the same level throughout.

## 2. Result

| instance | wall time | L-scaled (prediction) | measured / L-scaled | cores in use (of 18) | special-q's | relations (unique) | matrix rows |
|---|---:|---:|---:|---:|---:|---:|---:|
| RSA-100 | 0:04:31 | 0:00:27 | 10.0 | 7.8 | 53,046 | 3,132,416 | 174,665 |
| RSA-110 | 0:12:38 | 0:02:17 | 5.5 | 10.9 | 75,325 | 5,982,936 | 363,780 |
| RSA-120 | 0:41:00 | 0:10:31 | 3.9 | 12.5 | 229,669 | 11,650,221 | 662,253 |
| RSA-130 | 2:08:15 | 0:45:05 | 2.8 | 13.5 | 243,487 | 23,619,434 | 1,183,823 |

The time grows 2.80 times from RSA-100 to RSA-110, 3.25 times to RSA-120 and 3.13 times to RSA-130. It doubles every 6.7, 5.9 and 6.1 digits. L_N[1/3, 1.923] says it doubles every 4.3, 4.5 and 4.8.

**The one-hour reach is d\* = 123.3 digits**, interpolated in ln T between RSA-120 (41:00) and RSA-130 (2:08:15).

**H4 is rejected.** The prediction was 132 digits, held within [127, 137]. The measured reach lies 8.7 digits below it.

**Where the time goes** (wall time, from the log):

| instance | polynomial selection | factor base, free relations | sieving | filtering | linear algebra | characters, square root |
|---|---:|---:|---:|---:|---:|---:|
| RSA-100 | 6% | 10% | 60% | 2% | 15% | 7% |
| RSA-110 | 6% | 7% | 65% | 1% | 17% | 4% |
| RSA-120 | 5% | 4% | 73% | 1% | 15% | 2% |
| RSA-130 | 5% | 3% | 77% | 1% | 13% | 1% |

Sieving includes the filtering passes that run while it goes on. At RSA-130 sieving took 1:38:22 and the linear algebra 17:09, on a matrix of 1.18 million rows.

## 3. Reading

- **The laptop's reach.** With CADO-NFS as shipped, this laptop factors a hard 123-digit number in one hour. The goal is 200 digits, which is 77 digits further on. H1 puts that at 4,300 to 4,700 times the laptop's hour.
- **The L-scaling below 200 digits.** Carried down from H1, L_N[1/3, 1.923] makes the cost fall too fast below 200 digits. The measured times are 10 to 2.8 times the scaled ones, and the gap closes with size.
  - The constant that joins one hour at 123.3 digits to 4,300 to 4,700 hours at 200 is c_eff = 1.67 to 1.68.
  - Fitted across the four runs alone, the constant is 1.41.
  - The constant rises with size, as the o(1) in L_N[1/3, c + o(1)] allows.
- **The figures above 200 digits, restated.** The prediction required these figures to be restated with c_eff, beside 1.923, from 4,300 to 4,700 laptop-hours at 200 digits:

  | digits | with 1.923 | with c_eff = 1.67 to 1.68 |
  |---:|---:|---:|
  | 240 | 2.9 × 10^5 | 1.6 to 1.8 × 10^5 |
  | 250 | 7.6 × 10^5 | 3.7 to 4.2 × 10^5 |
  | 260 | 2.0 × 10^6 | 8.4 to 9.7 × 10^5 |

  The constant has risen with size across everything measured, so the c_eff figures are the lower of the two.
- **Sustained clocks.** These runs ran at sustained clocks, lower than H1's 20-second samples.
  - During sieving the CPU was 96 to 99 percent busy at RSA-110 to RSA-130, drawing 37 to 44 W of CPU power. macmon's 6-core tier held 2.88 GHz and its 12-core tier 3.9 to 4.1 GHz.
  - H1's samples ran at 62 to 67 W, with the 12-core tier at 4.3 to 4.4 GHz and the 6-core tier at 3.8 to 4.4 GHz (`data/hardware/h1-c200-q*.jsonl`, local).
  - Measured at sustained clocks, H1's cost would be about 18 percent higher by the clocks, about 5,100 to 5,600 hours for a complete 200-digit factorization. That moves the predicted reach to about 131 digits and c_eff to 1.70 to 1.72. The verdict does not change.
- **What tuning could move.** Outside sieving the machine is partly idle: 42 to 61 percent of the CPU was active during the linear algebra, and CADO-NFS used 7.8 to 13.5 of the 18 cores on average over a whole run. The pieces a tuned run would change are:
  - the linear algebra's threads;
  - the relation target and polynomial selection effort;
  - the machine's power mode.

  At the measured growth, a run 25 percent faster at 125 digits moves the reach by about 2 digits.

## Files

| File | Role |
|---|---|
| `docs/h4_falsifiable_prediction.md` | The claim, the protocol and the criterion, committed before the runs. |
| `scripts/h4_reach.sh` | The runs under `hw-sample`, ascending until one takes more than one hour, then the summary. |
| `scripts/h4_summary.py` | The published factors (`--n`), the log filter (`--filter-log`), the check of the factors, the phases, the reach and the criterion; writes `data/results/h4_reach.{md,csv}` and `data/results/h4/hardware_by_phase.csv`. |
| `data/results/h4/` | Per instance: the `time` output, the printed factors, the log's Info lines without paths, addresses or certificate hashes, and the `hw-sample` summary. Also the parameter files' hashes and the CADO-NFS commit. |
