# Primness: Structure Beyond the Sieve

**Metadata**
- **Created**: 2026-11-03 (base document)
- **Last Modified**: 2026-11-10 13:45 EST
- **Version**: 1.24
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

As of 2026-11-10 13:45 EST:

I am developing on `primness_dev01`. Pull request 27 is merged. D1 found nothing at this scale, so this change tests D5, the sub-claim that smooth values cluster in regions that can be located, on the same ten instances. It also drafts the D1 verdict for my approval.

I wrote the D5 prediction before measuring (`docs/d5_falsifiable_prediction.md`). Every value gets the smooth probability D1 ended with: the conditional baseline at its own cofactor plus the first-order root-count term. The region is cut into tilings from 2 × 2 to 50 × 50 and into 64 sectors, and the values are split into two halves by a hash of (a, b). To be supported, regions picked for an excess in one half must show an excess in the other. That excess must exceed three errors, and the rational control's lift by three combined errors, at one of the protocol's bounds.

D5 is rejected on these instances. The largest held-out lift is +1.5 errors (5 × 5 tiles at 2^24), and eleven of the eighteen are negative. At 2^28 the lift on 50 × 50 tiles is −0.11% ± 0.15%. The tiles scatter as counting noise says (χ² per tile 0.98 to 1.00 on the 25 × 25 and 50 × 50 tiles), and the split-half correlations are at zero. The method and result are in `scripts/d5_clusters.py` and `docs/d5_smooth_clusters.md`. `d1-measure --dump` now carries each value's small-prime part.

One thing in the control is unexplained. The rational side's 64 sectors scatter less than counting noise (χ² per sector 0.79 to 0.87 at 2^16 through 2^28), while its tiles and the algebraic sectors do not. Less scatter is the opposite of clustering, so it does not bear on D5. It does hint that counting noise overstates the noise of large regions. That would make D1's per-instance errors upper bounds. The D1 verdict uses the spread across instances, which does not depend on that noise.

`docs/d1_falsifiable_prediction.md` now ends with a draft verdict for my approval: under the relative weighting the claim is rejected at 2^20, 2^24 and 2^28. It lists the protocol's departures.

The next step is mine. I need to adopt or amend the D1 verdict. Then I choose among D2 (higher algebraic structure), D3 (lattice embeddings) and D4 (relation-matrix structure), which need new machinery, or a test of the closure at a larger scale. The required status check on the main ruleset is still to be applied in the repository settings.

Full history in `context_convo_notes/current_state_history.md`.

---

## open_questions

- Does the closure hold at a larger scale? These are 40–80 digit numbers with B up to 2^28 and a region with b ≤ 1500. At the sizes the success criterion needs, the region reaches past the primes it sieves and the special roots of section 9 stop mattering, but the first-order term's convexity grows with u. One larger instance at the protocol's bounds would show whether the residual stays at the noise.
- Why do the rational side's sectors scatter less than counting noise when its tiles do not? The χ² per sector is 0.79 to 0.87 at 2^16 through 2^28, and 0.35 for the largest sectors of d1-02 (`docs/d5_smooth_clusters.md`). Independent draws let the number of values each smaller prime divides vary, and a region fixes those numbers. The answer would also say how far counting noise overstates the noise of a whole region, and so D1's per-instance errors.
- Does the spread of the α tail across polynomials follow the Galois group, as the toy quartics suggested for α alone (A4 least, V4 most)? It is now the whole of what a polynomial does beyond the small primes, so the question is about root statistics and no longer a candidate for D1.
- Which of macmon's two CPU tiers on this M5 Max is the performance tier? macmon labels the 12-core tier `P` and the 6-core tier `S`; `sysctl` reports `hw.perflevel0.physicalcpu` = 6 and `hw.perflevel1.physicalcpu` = 12. The hardware samples keep macmon's labels as reported.

*Living base document for the Primness project.*
