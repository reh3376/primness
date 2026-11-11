# Primness: Structure Beyond the Sieve

**Metadata**
- **Created**: 2026-11-03 (base document)
- **Last Modified**: 2026-11-10 19:30 EST
- **Version**: 1.26
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

As of 2026-11-10 19:30 EST:

I am developing on `primness_dev01`. Pull request 28 is merged, and this change follows it. It closes the D measurements and starts Obstacle H.

I closed the D measurements and adopted the D1 verdict, which rejects the claim (`docs/d1_falsifiable_prediction.md`). `docs/obstacle_d_verdict.md` records the verdict on Obstacle D. D1, D5 and D2 part 1 were each rejected under a criterion written before measuring.

A closing pass over all the data looked for anything those criteria could miss (`scripts/d_checks_run_all.sh`, binary `d-checks`, `data/results/d_checks.md`). It found nothing:
- The 2,160 per-instance statistics show no shift or tail of the kind a hidden excess would leave.
- Smoothness at fixed cofactor does not depend on the small-prime part, the other side, or (a, b) mod 2 or 3.
- Neighbouring values are not smooth together more often than independence says.
- The prediction holds up to u ≈ 5, which covers the u at which a 200-digit run sieves.
- The trend in digits allows at most +0.47% at 200 digits at 2^28 (95% upper bound).
- Five toy quartics with Galois groups A4, S4, C4, D4 and V4 follow the model whatever their group.

The values of the number field sieve are as smooth as their sizes and root counts make them, and nothing more. What is left is smaller values, another source of relations, or more throughput.

Obstacle H is forced in the sense the README gives it. Its first measurement, H1, was written down and committed before the run (`docs/h1_falsifiable_prediction.md`). It is the relation collection for RSA-200 on this laptop, with CADO-NFS's own 200-digit polynomial and parameters, sampled over six special-q ranges and integrated to CADO-NFS's relation target. It takes 3,896 laptop-hours (`docs/h1_sieving_cost.md`, `data/results/h1_cost.md`). H1 holds: the estimate scaled from the records was 6,300, and the measurement is 0.62 of it. Sieving takes 50 to 64 percent of the time and cofactorization 25 to 37 percent, so neither alone decides the speed. The gap at 200 digits is about 3,900 times for the relations, or 4,300 to 4,700 times with the linear algebra.

Next is H2. The laptop's GPU is part of the success criterion and has not been measured, so H2 measures what it adds to the two dominant kernels and states what a machine must do per second to collect 200-digit relations in an hour. The linear algebra at 200 digits and the laptop's own one-hour reach with tuned CADO-NFS settings follow. New mathematics enters when there is a construction the harness can price. The required status check on the main ruleset is still to be applied in the repository settings.

Full history in `context_convo_notes/current_state_history.md`.

---

## open_questions

- Does the closure hold at a larger scale? These are 40–80 digit numbers with B up to 2^28 and a region with b ≤ 1500. The closing checks narrowed it (`docs/obstacle_d_verdict.md` section 2.3). The small bounds already put the values at the u of a 200-digit run, and the model holds there within the counting error. The trend in digits allows at most +0.47% at 200 digits at 2^28. What a larger instance would still test is a larger B at the same u.
- Why do the rational side's sectors scatter less than counting noise when its tiles do not? The χ² per sector is 0.79 to 0.87 at 2^16 through 2^28, and 0.35 for the largest sectors of d1-02 (`docs/d5_smooth_clusters.md`). The closing checks add that D5's held-out lifts are negative on both sides and that the rational side's D1 χ² is 3.3 on ten at 2^28. Smooth values are slightly anti-clustered. The likely reason is that a region holds an almost fixed number of each prime's multiples, since they form a lattice.
- The five toy quartics sit +0.10% ± 0.03% above the model at 2^24 and 2^28, whatever their Galois group, while the forty NFS polynomials do not. Is that the neglected second-order term, or roots of small height that tiny coefficients produce? It is too small to matter for the verdict.
- What does the laptop's GPU add to the two kernels that dominate the 200-digit sieve, the bucket sieve and the ECM cofactorization? H1 measured the CPU alone (`docs/h1_sieving_cost.md`).
- 240 to 260 digits, the range of the latest records, is a stretch beyond the 200-digit target. At H1's measured rate it needs about 3 × 10^5, 8 × 10^5 and 2 × 10^6 laptop-hours at 240, 250 and 260 digits. By L_N[1/3] that calls for a constant below the special number field sieve's 1.526 on general numbers.
- Which of macmon's two CPU tiers on this M5 Max is the performance tier? macmon labels the 12-core tier `P` and the 6-core tier `S`; `sysctl` reports `hw.perflevel0.physicalcpu` = 6 and `hw.perflevel1.physicalcpu` = 12. The hardware samples keep macmon's labels as reported.

*Living base document for the Primness project.*
