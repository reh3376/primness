# Tools Integration Scaffolding (2026-11-03)

## Added
- `tools/README.md`: instructions for using CADO-NFS, msieve, and YAFU solely as external measurement instruments. Shared relation format documented. No code vendored.
- `Cargo.toml` and `src/bin/d1_measure.rs`: initial Rust workspace and stub binary for the independent D1 smoothness measurement harness.

## Rule restated
Existing tools supply candidate polynomials and relations. All smoothness statistics, Dickman–de Bruijn comparisons, and falsifiable claims are computed by the Primness Rust code. Their published constraints remain hypotheses.

## Next
Implement the first measurement protocol (10 hard 40–80 digit semiprimes, independent B-smooth density counts) inside `d1_measure`.
