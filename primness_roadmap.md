# Primness: Structure Beyond the Sieve

**Metadata**
- **Created**: 2026-11-03 (base document)
- **Last Modified**: 2026-11-03 19:28 EST
- **Version**: 1.19
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

As of 2026-11-03 19:28 EST:

I am developing on `primness_dev01`. Pull request 23 is merged, and this change is the first on the branch after that merge. D1 is still my starting sub-claim under Obstacle D.

All ten instances are measured. Each has a CADO-NFS degree-4 polynomial selected outside this tree, a Murphy α computed here, and two runs on the same region: 2^14 to 2^20 and the protocol's 2^20, 2^24, 2^28, with the line sieve and the hardware trace. The residual after α, as a share of the prediction, averages +21.7% at 2^14, +12.9% at 2^16, +6.7% at 2^18, +3.3% at 2^20, -0.3% at 2^24, -1.7% at 2^28. Its sign is set by u, not by the instance: small instances sit below the α-adjusted prediction, large ones above it, and every instance drifts the same way as B grows. `scripts/d1_summary.py` applies the written criterion per bound and gets a split verdict, rejected at 2^20 and not at the others, with the absolute-gap weighting that the prediction document does not specify. I do not read either verdict as evidence for or against D1 yet: a residual that depends only on u is the finite-B error of the Dickman baseline, which at these bounds is of the same size as the residual.

I also ran a ladder of structured polynomials on the same region. The reducible x^n − 1 family shows the excess that survives α, up to +83% for x^6 − 1, which at the same mean u as d1-01 is smooth eleven times as often; that is the special number field sieve's advantage and is closed to a hard N because a reducible polynomial factors N. Five irreducible quartics with different Galois groups all fall below the α-adjusted prediction, roughly in the order of how many primes can divide their values. The harness now tells three shapes apart, and D1 needs an irreducible polynomial with the first. None of the ten has it.

The next step is to calibrate the baseline inside this harness, by counting the smoothness of random integers of the same sizes at the same bounds, and then to re-apply the criterion with relative gaps and a stated weighting. After that, the A4 lever: polynomials with square discriminant for one instance.

The required status check on the main ruleset is still to be applied in the repository settings.

Full history in `context_convo_notes/current_state_history.md`.

---

## open_questions

- Which Dickman approximation is the right baseline for the D1 comparison at u between 2 and 5? Across ten instances the residual after α changes sign with u, from about −3% at u near 3 to about +20% at u near 5, between the plain ρ(u + α/ln B) and the local-density form. The rejection criterion needs a baseline whose own error is below the measurement noise, which points to an empirical calibration in this harness.
- Does the criterion in `docs/d1_falsifiable_prediction.md` mean the absolute gap, which weights the dense small instances most, or the relative gap? The verdict differs.
- Does the residual after α track the Galois group of the polynomial on real selected polynomials, as it does on the toy quartics (A4 least, V4 most)?
- Which of macmon's two CPU tiers on this M5 Max is the performance tier? macmon labels the 12-core tier `P` and the 6-core tier `S`; `sysctl` reports `hw.perflevel0.physicalcpu` = 6 and `hw.perflevel1.physicalcpu` = 12. The hardware samples keep macmon's labels as reported.

*Living base document for the Primness project.*
