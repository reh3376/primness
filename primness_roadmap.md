# Primness: Structure Beyond the Sieve

**Metadata**
- **Created**: 2026-11-03 (base document)
- **Last Modified**: 2026-11-03 17:50 EST
- **Version**: 1.16
- **Schema**: current-state/1
- **Tags**: factoring, GNFS, obstacle-D, roadmap, primness
- **Related Files**: context_convo_notes/current_state_history.md, context_convo_notes/INDEX.md
- **Repository**: https://github.com/reh3376/primness (public; development on `primness_dev01` only; PRs required to `main`)

**Active Constraints** (update only when a constraint changes)
- Primary hardware envelope: modern multi-core CPU (8–16 cores), 16–64 GB RAM, discrete GPU ≥16 GB VRAM or Apple Silicon ≥24 GB unified memory; no large distributed clusters.
- Target size: \(N \ge 10^{200}\) (≈664 bits).
- Success criterion: complete factorization (or primality proof) in < 1 hour wall-clock time.
- Preferred starting obstacle: D (missing deeper mathematical structure).
- Explicit non-goals: no reliance on fault-tolerant quantum hardware; no assumption of specialized hardware until Obstacle H is forced.
- Implementation language: core numerical kernels in Rust; Python sidecar only for ML/AI libraries as needed.
- PR authorship: auto-PR workflow on `primness_dev01` opens/updates PRs; the repository owner reviews, approves, and merges.

**Acknowledgment**

I want to begin by acknowledging Dr. Michael R. Anderson, PhD, Department of Mathematics at West Virginia State University. In 1994, as a junior at WVSU, I received a NASA research grant to investigate integer factorization with the specific aim of assessing whether RSA could be cracked. Dr. Anderson was my mentor on that project. I remember him as consistently gracious and accommodating, and I am glad to be able to recognize him at the start of this return to the problem after many years.

Looking back, I believe my fondness for algorithms, machine learning, and artificial intelligence started with Dr. Anderson. The rigorous yet patient way he guided me through the early stages of that factorization work planted the seeds for the technical path I have followed since. Therefore, I attribute my current successes in these fields to his kind guidance and his tolerance of a young fool with “lofty aspirations.”

## 1. Fully define the problem

I am looking for a deterministic or randomized algorithm \(A\) that does the following:

- Takes as input an integer \(N \ge 10^{200}\) (at least 200 decimal digits, roughly 664 bits).
- Returns the complete prime factorization of \(N\) (the multiset of prime factors with multiplicity), or a proof that \(N\) is prime.
- Runs to completion in less than one hour of wall-clock time on standard laptop hardware that includes a GPU. I am taking that to mean a modern multi-core CPU (roughly 8–16 cores), 16–64 GB of system RAM, and either a discrete GPU with at least 16 GB of VRAM or, on an Apple Silicon Mac, at least 24 GB of unified memory. I am not relying on large distributed clusters.
- Succeeds on hard instances: balanced semiprimes \(N = p \cdot q\) where \(p\) and \(q\) are primes of comparable size and have no special algebraic form that would let SNFS or other tailored methods apply. It should also handle general integers that may have smaller factors.
- Produces output that can be verified (the factors multiply back to \(N\) and are prime, or a primality certificate is given).

I prefer a solution that meets the laptop-plus-GPU constraints above. At the same time, I am prepared to accept that a correct solution for numbers of this size may require new mathematical insight together with specialized hardware that does not yet exist. If that becomes clear, the problem expands to include specifying and designing that hardware.

## 2. Why I consider the problem important to solve

- The security of RSA and related systems still rests on the practical difficulty of factoring integers in this range. An algorithm that met the constraints would make currently deployed RSA moduli insecure.
- Factoring remains a foundational problem in computational number theory. A practical advance at this scale would be a genuine step forward in our understanding of the integers.
- Factoring sits in NP (and, under standard assumptions, in NP ∩ coNP) but is not known to be NP-complete. A dramatically faster method would clarify the status of problems that appear to lie between P and the NP-complete problems.
- Being able to factor arbitrary 200-digit integers quickly, whether on existing laptops or on hardware we still need to build, would expand what can be done routinely in computational number theory.
- On a personal level, this is the same problem I worked on under the 1994 NASA grant. Returning to it now lets me pick up that thread with contemporary tools and a clearer statement of what success would require.

## 3. Known obstacles

These are the barriers I currently see. None of them is a rigorous proof that the problem is impossible under every possible hardware regime, but they are the reasons no solution exists on the primary constraints today.

- **Obstacle A (complexity-theoretic)**: No polynomial-time classical algorithm for factoring is known. The best complexities we have are still sub-exponential but super-polynomial. There is neither a proof that polynomial time is impossible nor a proof that factoring is NP-hard.
- **Obstacle B (best known algorithm)**: The General Number Field Sieve remains the asymptotically fastest published method for hard numbers of this size. Its heuristic complexity is \(L_N[1/3, (64/9)^{1/3} + o(1)]\). Even with GPU acceleration of the sieving phase, the concrete cost for a hard 200-digit semiprime still far exceeds a single laptop-hour.
- **Obstacle C (SAT reduction does not help)**: There is a polynomial-time reduction from factoring to SAT by encoding multiplication as a Boolean circuit and converting to CNF. Modern SAT solvers, whether on CPU or GPU, do not solve the resulting formulas beyond a few dozen bits in practical time.
- **Obstacle D (missing deeper mathematical structure)**: No published method improves on the GNFS exponent for hard integer factorization. Sub-claims D1–D5 examine polynomial smoothness, higher algebraic structure, lattice embeddings, relation-matrix structure, and non-random smooth clusters.
- **Obstacle E–H**: (remaining obstacles as previously defined; full text preserved in living document).

## Working notes

(Full sub-claims D1–D5, analysis selecting D1, Decision Log template, and Falsifiable Prediction template are present in the complete living document.)

## Current State

As of 2026-11-03 17:50 EST:

I am developing on `primness_dev01`. Pull request 21 is merged, and this change is the first on the branch after that merge. D1 is still my starting sub-claim under Obstacle D.

I built CADO-NFS outside this tree (`~/external/cado-nfs`, commit 692ecb7) and ran its polynomial selection on d1-01 through `scripts/cado-polyselect.sh`: 111 root-optimized degree-4 candidates, the largest MurphyE 4.749e-06, kept as `data/instruments/d1-01/cado.poly` with the score only in a comment. The harness now evaluates the homogeneous algebraic and rational values in BigInt over a coprime rectangle, counts B-smooth values by trial division over a sieved prime list, and compares the observed density with the Dickman–de Bruijn prediction, the mean of ρ(ln|F| / ln B) over the region. ρ is tabulated by Simpson's rule on the integral equation; tests pin ρ(1) = 1, ρ(2) = 1 − ln 2 and the literature values at 3, 4, 5 and 10. No Murphy α is applied, because no definition of it is recorded in this repository yet.

The first scaled-down run on d1-01 (a in [−3000, 3000], b in [1, 1500], gcd 1; 5,472,915 values; 17.6 s on 18 threads) is `data/results/d1-01.scaled-down.json`. At B = 2^20 the algebraic side is 0.05888 observed against 0.03340 predicted, a gap of +0.02548 with a binomial standard error of 0.00010; the gaps at 2^14, 2^16 and 2^18 are also positive. The rational side is below its prediction at every bound. I do not read the algebraic excess as evidence for D1: the prediction carries no root-property correction, and the protocol compares against Dickman adjusted by the measured Murphy α. Computing α in the harness, from a definition recorded here, is the next step; only after that does the rejection criterion apply. The run is `scaled-down`: the protocol's region per digit length is not fixed, and trial division does not reach the protocol's bound 2^28.

Every instrument run now records a `hardware-sample/1` trace (`docs/hardware_telemetry.md`): CPU and GPU activity, unified memory and the GPU's share of it, temperature, fans and power each second, from macmon and ioreg, validated against the schema, kept as JSON lines and in Postgres, summarized in the result file and plotted on the bench. The d1-01 run held the CPU at 0.90 active ratio on average and peaked at 77 °C; the GPU was idle; other processes held 12.5% of unified memory on the GPU throughout.

The required status check on the main ruleset is not applied yet; I add that rule in the repository settings myself.

The next measurement step is Murphy α in the harness, then the same run on d1-02 through d1-10, then a sieve over the region to reach the protocol's bounds.

Full history in `context_convo_notes/current_state_history.md`.

---

## open_questions

- Which of macmon's two CPU tiers on this M5 Max is the performance tier? macmon labels the 12-core tier `P` and the 6-core tier `S`; `sysctl` reports `hw.perflevel0.physicalcpu` = 6 and `hw.perflevel1.physicalcpu` = 12. The hardware samples keep macmon's labels as reported.

*Living base document for the Primness project.*
