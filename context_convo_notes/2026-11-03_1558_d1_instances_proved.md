# Proved D1 instances (2026-11-03 15:58 EST)

## Done
- Replaced the placeholders in `data/d1_test_instances.md` with ten semiprimes of 40, 44, 48, 52, 56, 60, 64, 68, 72, and 80 digits.
- Each factor of at most 24 digits is proved by Miller-Rabin on the first 13 prime bases, which is deterministic below ψ_13 (Sorenson and Webster, 2017).
- Each larger factor is proved by one Pocklington step whose prime factor of p−1 is itself in that Miller-Rabin range.
- `cargo test` rechecks the committed certificates.

## Not done
- No SNFS-structure test is defined in the repository, so none was applied.
- Polynomials from CADO-NFS and msieve are not attached.
- The Dickman comparison has not been run.

## Next
Obtain one polynomial per instance from CADO-NFS and one from msieve, then run the independent evaluation inside `d1_measure`.
