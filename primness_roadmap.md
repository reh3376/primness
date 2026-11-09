# Primness: Structure Beyond the Sieve

**Metadata**
- **Created**: 2026-11-03 (base document)
- **Last Modified**: 2026-11-09 10:05 EST
- **Version**: 1.20
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

As of 2026-11-09 10:05 EST:

I am developing on `primness_dev01`. Pull request 24 is merged, and this change is the first on the branch after that merge. D1 is still my starting sub-claim under Obstacle D.

The baseline is now measured, not assumed. `d1-baseline` sieves consecutive integers near 101 sizes from 5 × 10^8 to 10^52, 16.8 million at each, with the same line sieve as the instrument runs, and compares their smoothness at the protocol's bounds with three Dickman forms. Integers without structure follow the de Bruijn local form ρ(u) + γρ′(u)/ln B to a fraction of a percent from 2^16 up (1.7% at 2^14); ρ(u) itself overstates their smoothness by 6 to 13 percent at these bounds, and the derivative form the harness carried as the local density understates it by about as much. `docs/baseline_calibration.md` records the derivation and the run.

Against that baseline, re-applied per u bin by `scripts/d1_calibrate.py`, the algebraic side of all ten instances sits above the α-adjusted prediction at every bound, +44% at 2^14 falling to +4.2% at 2^28, and the sign no longer changes with u. The shape of what remains is a second-order term of the α heuristic itself: α fixes the mean of the log small-prime part and not its spread, and ρ is convex, so the spread raises the smoothness by about α₂ ρ″/(2ρ ln² B), with α₂ the excess variance of that log computed from the polynomial by the same root lifting as α (`src/alpha.rs`, `d1-alpha`, `alpha-moments/1`). The variance each instance's gap implies is nearly the same at all six bounds and close to its computed α₂ (d1-01: 5.5 to 7.4 against 6.3). With α₂ applied the mean gap is +11% at 2^14, +2.8% at 2^20, +1.2% at 2^24 and +0.5% at 2^28, two standard deviations of the spread across instances at 2^28; on the rational side, where the model should be exact, it is +0.11%, +0.04% and −0.01% at the protocol bounds. The criterion rejects at no bound under either weighting, and I do not read that as evidence for D1: three quarters to seven eighths of the gap after α was the heuristic's own second-order term.

The criterion's weighting is proposed in `docs/baseline_calibration.md` section 6 (relative gap, de Bruijn baseline with α and α₂, noise the larger of Poisson and the instance spread) and left for me to adopt in the prediction document.

The next step replaces the expansion altogether: compute the exact small-prime part of every value and compare its smoothness with the baseline's conditional density for cofactors of that size, which `d1-baseline` can tabulate from the same consecutive integers. A gap that survives that is structure at primes between the α bound and B and the first evidence for D1; a gap that vanishes closes the α chapter.

The required status check on the main ruleset is still to be applied in the repository settings.

Full history in `context_convo_notes/current_state_history.md`.

---

## open_questions

- What carries the gap that remains after the measured baseline and α₂: +1.2% at 2^24 and +0.5% at 2^28 on the algebraic side, positive for eight of ten instances, with implied variances 10 to 50 percent above the computed α₂ for most instances? The candidates are the third moment of the small-prime part, the independence and random-cofactor approximations inside α, and structure at primes between the α bound and B. The exact small-prime measurement in `docs/baseline_calibration.md` section 5 separates them.
- Does the residual after α and α₂ track the Galois group of the polynomial on real selected polynomials, as the toy quartics suggested for α alone (A4 least, V4 most)? The toy ladder has not been re-read against the measured baseline, and the reducible x^n − 1 family is outside the second-order expansion's regime.
- Which of macmon's two CPU tiers on this M5 Max is the performance tier? macmon labels the 12-core tier `P` and the 6-core tier `S`; `sysctl` reports `hw.perflevel0.physicalcpu` = 6 and `hw.perflevel1.physicalcpu` = 12. The hardware samples keep macmon's labels as reported.

*Living base document for the Primness project.*
