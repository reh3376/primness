# Primness Project: History of Current States

**Metadata**
- **Created**: 2026-11-03 12:10 EST
- **Last Modified**: 2026-11-03 16:40 EST
- **Version**: 1.5
- **Tags**: current-state, history, roadmap, obstacle-D
- **Related Files**: primness_roadmap.md

## Current State Entries

### 2026-11-03 16:40 EST
- The harness parses CADO and msieve polynomial files into `instrument-polynomial/1`. `format` and `source` are closed lists. Instrument scores are not fields. A parsed file stays `source` `unspecified`.
- CADO-NFS stays outside the tree because it is LGPL-2.1. msieve is public domain. The stock Apple clang build stops because `gettimeofday` is undeclared in `common/util.c`. A checkout outside this tree builds after that file includes `<sys/time.h>`. The binary is not in this repository. Polynomial selection has not been run. YAFU is not copied in; its repository does not state a license in the README or a root LICENSE file.
- The D1 prediction is still unmeasured. The ten proved semiprimes are unchanged. The next step is one polynomial per instance from CADO-NFS and one from msieve, then the smoothness count and the Dickman comparison.

### 2026-11-03 16:36 EST
- Pull requests 16 and 17 are merged.
- The harness parses CADO and msieve polynomial files into `instrument-polynomial/1`. `format` and `source` are closed lists. Instrument scores are not fields.
- CADO-NFS stays outside the tree because it is LGPL-2.1. msieve is public domain and is being built outside the tree. YAFU is not copied in; its repository does not state a license in the README or a root LICENSE file.
- The D1 prediction is still unmeasured.

### 2026-11-03 16:24 EST
- The current-state gate reads `schemas/current-state/1.json`, named by the roadmap metadata `Schema` line (`current-state/1`).
- Paths, the section heading, the stamp shape, and which metadata labels must change are const values in that version. Enums list the closed options.
- Pull request 15 is merged. Draft pull request 16 remains open. The D1 prediction is still unmeasured.

### 2026-11-03 16:06 EST
- Pull request 15 is merged.
- Auto-PR opens or refreshes a draft only after `primness_roadmap.md` Current State changes and this file keeps the previous snapshot heading plus the new one.
- The D1 prediction is still unmeasured. The ten proved semiprimes remain in `data/d1_test_instances.md`. The next measurement step is the external polynomials and the Dickman comparison.

### 2026-11-03 15:58 EST
- Current State in `primness_roadmap.md` advanced from the 12:42 EST snapshot.
- The D1 prediction is written and unmeasured. The harness and bench remain toy-scale for smoothness.
- `data/d1_test_instances.md` now holds ten proved semiprimes from 40 through 80 digits. Factors of at most 24 digits use deterministic Miller-Rabin below ψ_13. Larger factors use one Pocklington step.
- Next measurement step is the external polynomials and the Dickman comparison.

### 2026-11-03 13:20 EST
- Living roadmap (primness_roadmap.md v1.6) added to the repository.
- Notes index and current-state history updated.

### 2026-11-03 13:05 EST
- Auto-PR workflow added to primness_dev01.

### 2026-11-03 12:51 EST
- Survey note of existing tools committed.

### 2026-11-03 12:43 EST
- Ruleset protecting primness_dev01 against deletion and force-pushes.

### 2026-11-03 12:42 EST
- PR #1 merged to main.
