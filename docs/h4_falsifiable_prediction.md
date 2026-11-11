# Falsifiable Prediction under Obstacle H / H4: the laptop's one-hour reach, measured

Written on 2026-11-11, before any factorization of 100 digits or more was run on this machine. The only end-to-end run so far is CADO-NFS's own 59-digit example, which checked that the driver works.

**Scope**
H1 measured the relation collection at 200 digits on this laptop: 3,896 laptop-hours (`docs/h1_sieving_cost.md`). Every figure the roadmap gives at another size is carried from that measurement by L_N[1/3, 1.923], that is (64/9)^{1/3} with o(1) = 0. That covers the figures for 240 to 260 digits and the size at which this laptop's cost comes down to one hour. H4 measures that size directly: the largest hard semiprime this laptop factors completely, end to end, in one hour of wall-clock time, with CADO-NFS as shipped. It is the reach the success criterion counts, at the size where the machine actually meets it, and it tests the scaling the roadmap uses away from 200 digits.

**Claim (H4)**
On this machine, with CADO-NFS (commit 692ecb7), its default parameter files and 18 threads, the one-hour reach d* lies within 5 digits of 132, the value H1 carried down by L_N[1/3, 1.923] gives.

**Derivation**
- H1's 3,896 hours for the relations, divided by relation collection's share of the records' total (83 to 91 percent), give 4,300 to 4,700 hours for a complete 200-digit factorization.
- L_N[1/3, 1.923] brings that to one hour at 131.7 to 132.3 digits.
- On the same scaling, a complete factorization takes 0.5 minutes at 100 digits, 2.3 at 110, 10.5 at 120, 45 at 130 and 3.0 hours at 140.
- The time doubles every 4.9 digits near 130, so 5 digits is a factor of 2 in time, the tolerance H1 had.

**The instances**
RSA-100, RSA-110, RSA-120, RSA-130 and, if the protocol reaches it, RSA-140. They are RSA challenge numbers, balanced semiprimes with no special form, whose factorizations were published between 1991 and 1999. CADO-NFS's reference polynomials `parameters/polynomials/c100.poly` to `c140.poly` were found for these numbers, so its parameter files were tuned on them. The runs do not use those polynomials: CADO-NFS selects its own, and the time counts that selection.

**Measurement protocol**
1. Each instance runs once as `cado-nfs.py N -t 18` in an empty work directory, under `hw-sample`. CADO-NFS picks the parameter file for the size (`params.c100` and so on), and nothing in it is overridden. The run includes everything CADO-NFS does: polynomial selection, the factor base and free relations, sieving, filtering, linear algebra and the square root.
2. Order: ascending in steps of 10 digits from RSA-100. The protocol stops after the first run that takes more than one hour. No other measurement runs meanwhile.
3. The time is the wall-clock time of the whole `cado-nfs.py` process. CADO-NFS's own total and the timestamps in its log give the phases.
4. The reach d* is interpolated linearly in ln T against the digit count, between the last run under one hour and the first run over it.
5. A run counts only if the factors it prints multiply to N and equal the published factors.

**Rejection criterion**
H4 holds if d* lies in [127, 137]. Otherwise H4 is rejected. In that case the cost carried from 200 digits by L_N[1/3, 1.923] misplaces the laptop's own one-hour reach by more than a factor of 2 in time. The roadmap's figures away from 200 digits are then restated with the effective constant this measurement gives, beside the ones with 1.923.

**Reported beside the verdict, not part of it**
- Each phase's wall time and share, per instance.
- The effective constant that takes the measured reach to H1's 200-digit cost: c_eff = ln(T_200 / 1 h) / (f(200) − f(d*)), where T_200 is 4,300 to 4,700 hours and f is the L_N[1/3] exponent (ln N)^{1/3} (ln ln N)^{2/3}. Also the slope of ln T in f across the runs.
- Peak memory, power and temperature from the hardware samples.
- Tuned settings (relation target, threads in the linear algebra, polynomial selection effort) are a later step and do not enter this verdict.

**Hardware envelope**
This machine: Apple M5 Max, 18 CPU cores, 40 GPU cores, 128 GB unified memory. CADO-NFS uses only the CPU.

**Relation to the rest of Obstacle H**
H1 measured the cost at the goal's size, and H2 measured what the GPU adds. H4 measures the size at which the goal's one hour is met today. The distance from d* to 200 digits is the gap in the units the success criterion uses.
