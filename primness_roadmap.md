# Primness: Structure Beyond the Sieve

**Metadata**
- **Created**: 2026-11-03 (base document)
- **Last Modified**: 2026-11-10 05:15 EST
- **Version**: 1.22
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

As of 2026-11-10 05:15 EST:

I am developing on `primness_dev01`. Pull request 25 is merged, and this change follows the conditional-baseline table on the branch. D1 is still my starting sub-claim under Obstacle D.

The measurement that replaces the α expansion is done, and it finds no excess. `d1-baseline` tabulates Q_B(c), the density of B-smooth integers among integers with no prime factor up to 2000, by the size of c, from 1.69 × 10^9 consecutive integers (`d1-baseline/2`). `d1-measure` divides every polynomial value by its part made of the primes up to 2000 and bins the cofactor the same way (`d1-measurement/4`). `scripts/d1_conditional.py` sums Q_B over the values of each instance. The prediction uses no α, no α₂ and no Dickman function. Sections 7 and 8 of `docs/baseline_calibration.md` record the method, and `data/results/summary.conditional.md` the numbers.

The rational side is the control, because a linear polynomial has one root at every prime. Its mean gap over the ten instances is +0.02% at 2^24 and −0.02% at 2^28, inside the noise at every bound. On the algebraic side the mean gap is +0.57% ± 0.64% at 2^20, +0.35% ± 0.42% at 2^24 and +0.16% ± 0.24% at 2^28, the error being the spread across instances. Most of the +1.2% and +0.5% left after α and α₂ was the expansion's truncation.

Single instances are not at zero: d1-08 is +1.72% at 2^28, d1-09 is −1.18%, each more than six standard errors. Those deviations follow the tail of α over the primes between 2000 and B, which is zero on average over polynomials: the correlation across instances is 0.93 to 0.98 at every bound, and removing the tail's first-order prediction cuts the spread at 2^28 from 0.76% to 0.29% and leaves a mean of +0.10% ± 0.09%. The first-order prediction is too small by a factor of 1.5 at 2^28 and 3.4 at 2^14. I expect that to be the linearization, since one prime near B changes the cofactor by an amount of the order of ln B, but I have not computed it.

I read this as no evidence for D1 on these ten instances at this scale. The rejection threshold in the prediction document is met at every bound under the relative weighting proposed in section 6 of the calibration document; that weighting and the verdict are still mine to adopt in `docs/d1_falsifiable_prediction.md`.

Two choices in the build mattered. The bins are an eighth of a bit wide so that every power-of-two bound is a bin edge, because Q jumps at c = B; the first table, in bins of 0.25, put −0.11% on the control at 2^28. And α's root enumeration now uses the roots mod p, which made α at 2^24 a four-minute computation; α at 2^28 is not computed and the tail at 2^24 stands in for it.

The next step is a decision: compute the exact first-order term of the primes between 2000 and B from the root counts and close D1 at this scale, or move to another sub-claim. The required status check on the main ruleset is still to be applied in the repository settings.

Full history in `context_convo_notes/current_state_history.md`.

---

## open_questions

- Why is the α tail's first-order prediction too small, by 1.5 at 2^28 rising to 3.4 at 2^14, when it orders the instances almost perfectly? The candidate is the linearization: the exact first-order term is Σ (n_p − 1)/p × (Q_B(c/p)/Q_B(c) − 1) over the primes between 2000 and B. It needs the root counts per prime and the table, and it would say whether anything is left per polynomial.
- Is the mean gap after the tail real? It is positive at five of six bounds by about one standard error each (+0.10% ± 0.09% at 2^28), on values the bounds share. More instances, or a larger region per instance, would decide it; the exact first-order term might remove it.
- Does the residual track the Galois group of the polynomial on real selected polynomials, as the toy quartics suggested for α alone (A4 least, V4 most)? The α tail is a sum over primes of root-count fluctuations, whose variance the Galois group fixes, so the question now has a concrete form.
- Which of macmon's two CPU tiers on this M5 Max is the performance tier? macmon labels the 12-core tier `P` and the 6-core tier `S`; `sysctl` reports `hw.perflevel0.physicalcpu` = 6 and `hw.perflevel1.physicalcpu` = 12. The hardware samples keep macmon's labels as reported.

*Living base document for the Primness project.*
