# Primness: Structure Beyond the Sieve

**Metadata**
- **Created**: 2026-11-03 (base document)
- **Last Modified**: 2026-11-11 20:45 EST
- **Version**: 1.30
- **Schema**: current-state/1
- **Tags**: factoring, GNFS, obstacle-D, obstacle-H, roadmap, primness
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

As of 2026-11-11 20:45 EST:

I am developing on `primness_dev01`. Pull request 31 is merged, and this change follows it. It measures the linear algebra at 200 digits.

Obstacle D is closed (`docs/obstacle_d_verdict.md`). The values of the number field sieve are as smooth as their sizes and root counts make them, and nothing more. Obstacle H is forced on the primary hardware envelope:
- H1: relation collection for RSA-200 takes 3,896 laptop-hours on the CPU (`docs/h1_sieving_cost.md`).
- H2: the GPU is 8.2 times the CPU on the bucket sieve and 19.6 times on 96-bit Montgomery multiplication. That still leaves at least 189 laptop-hours (`docs/h2_gpu_kernels.md`).
- H4: with CADO-NFS as shipped, the laptop factors a 123.3-digit number in one hour (`docs/h4_one_hour_reach.md`). The gap to the goal is 77 digits.
- H5: the sieve proper is 65 percent of a complete factorization at that reach, so a sieve that cost nothing would buy at most 2.87 times (`docs/h5_sieve_share.md`). From 130 to 200 digits the cost per sieve position grows 1.45 times, and the number of positions 2,015 times.

H6 measures the linear algebra at 200 digits, the part a perfect relation collection would leave. I wrote the prediction and committed it before measuring (`docs/h6_falsifiable_prediction.md`). Scaled from H4's linear algebra, it is 46 laptop-hours within a factor of 2. The roadmap's stand-in, taken from the records' share, was about 400.

A real 200-digit matrix would need H1's 3,896 hours of relations, so its size is estimated from H4's matrices.
- Re-merging RSA-130's relations at c200's density gives κ = 0.849. Scaling by the large-prime ideal count then gives 14.1 million rows and 2.4 × 10^9 nonzeros.
- A new generator, `h6-matrix`, builds a synthetic matrix of that size with the column profile of a number field sieve matrix.
- At RSA-130's shape the synthetic matrix runs within 11.5 percent of the real one (`scripts/h6_linalg.sh`, `docs/h6_linalg_200.md`, `data/results/h6_linalg.md`).

Block Wiedemann at 200 digits takes 47.9 laptop-hours, so H6 holds.
- The parts: 661,289 matrix products at 0.24 s each (44 hours), lingen 0.95 hours, and 6 percent overhead.
- Other work on the machine slowed some intervals. Counting only clean ones gives 55.8 hours, and the top of H4's range of matrix sizes gives 62.5. The linear algebra is therefore 48 to 63 laptop-hours, a sixth to an eighth of the stand-in.
- The products are memory-bound. Once the 113 MB vector leaves the caches, the cost per nonzero is 1.2 to 1.4 times RSA-130's.

The linear algebra is 1.2 to 1.6 percent of relation collection and linear algebra together. A complete 200-digit factorization is therefore about 3,950 laptop-hours plus polynomial selection. A relation collection that cost nothing would make it at most 63 to 82 times faster. If polynomial selection kept the 5 to 6 percent share it had in H4, that ceiling would be about 15 times.

Next is polynomial selection at 200 digits, the one part of the cost not yet measured, and then the one-hour reach with tuned settings. H3 is not started.

Full history in `context_convo_notes/current_state_history.md`.

---

## open_questions

- Does the closure hold at a larger scale? These are 40–80 digit numbers with B up to 2^28 and a region with b ≤ 1500. The closing checks narrowed it (`docs/obstacle_d_verdict.md` section 2.3). The small bounds already put the values at the u of a 200-digit run, and the model holds there within the counting error. The trend in digits allows at most +0.47% at 200 digits at 2^28. What a larger instance would still test is a larger B at the same u.
- Why do the rational side's sectors scatter less than counting noise when its tiles do not? The χ² per sector is 0.79 to 0.87 at 2^16 through 2^28, and 0.35 for the largest sectors of d1-02 (`docs/d5_smooth_clusters.md`). The closing checks add that D5's held-out lifts are negative on both sides and that the rational side's D1 χ² is 3.3 on ten at 2^28. Smooth values are slightly anti-clustered. The likely reason is that a region holds an almost fixed number of each prime's multiples, since they form a lattice.
- The five toy quartics sit +0.10% ± 0.03% above the model at 2^24 and 2^28, whatever their Galois group, while the forty NFS polynomials do not. Is that the neglected second-order term, or roots of small height that tiny coefficients produce? It is too small to matter for the verdict.
- How far do tuned implementations move H2's ratios (8.2 for the sieve, 19.6 for the multiplication)? A tuned 96-bit multiplication on the CPU, a full ECM on the GPU, and a two-dimensional lattice siever with small-prime sieving and survivor rescans would each change them. None would move the bound of 189 hours by its own size.
- 240 to 260 digits, the range of the latest records, is a stretch beyond the 200-digit target. From H1's 200-digit cost, L_N[1/3, 1.923] gives about 3 × 10^5, 8 × 10^5 and 2 × 10^6 laptop-hours at 240, 250 and 260 digits. H4 measured an effective constant of 1.67 to 1.68 between the laptop's one-hour reach and 200 digits. With that constant the figures are 1.6–1.8 × 10^5, 3.7–4.2 × 10^5 and 8.4–9.7 × 10^5. The constant rose with size across everything measured, so the second set is the lower one. By L_N[1/3] that calls for a constant below the special number field sieve's 1.526 on general numbers.
- How long does polynomial selection take at 200 digits with CADO-NFS's c200 parameters? It is the only part of a 200-digit factorization no measurement here has covered. At the 5 to 6 percent share it had in H4's runs it would be 200 to 250 laptop-hours, four times the linear algebra (`docs/h6_linalg_200.md`). It also sets the ceiling on what a perfect relation collection could buy.
- How much slower is a sustained 200-digit run than H1's samples? H1's 20-second samples ran with the CPU at 62 to 67 W. H4's sustained sieving drew 37 to 44 W, with macmon's 6-core tier at 2.88 GHz (`docs/h4_one_hour_reach.md`). By the clocks a sustained run is about 18 percent slower, which would put a complete 200-digit factorization at about 5,100 to 5,600 laptop-hours. A sustained run over one special-q range would measure it.
- Which of macmon's two CPU tiers on this M5 Max is the performance tier? macmon labels the 12-core tier `P` and the 6-core tier `S`; `sysctl` reports `hw.perflevel0.physicalcpu` = 6 and `hw.perflevel1.physicalcpu` = 12. The hardware samples keep macmon's labels as reported.

*Living base document for the Primness project.*
