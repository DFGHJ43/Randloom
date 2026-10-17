# Golden fixture provenance

These files were generated from the C reference implementation with
`legacy/c/tools/dump_sequences.c` on Windows using MinGW GCC. The harness
includes `legacy/c/src/random.c` directly so it can call the static engine
helpers (`next_u64`, `next_double`, `next_double_open`) as well as the public
distribution functions.

Regeneration commands are in `docs/rust-rewrite/migration.md`. The files are
plain whitespace-separated numeric values (one per line), committed with LF
line endings via `.gitattributes`.

## Parity policy

- `engine_seed42.golden`, `f64_seed42.golden`, `uniform_seed42.golden`,
  `bernoulli_seed42.golden`, and the `poisson_*.golden` files must match the
  Rust implementation exactly (bit-for-bit for floats).
- `normal_seed42.golden` is compared with a small tolerance because Box-Muller
  uses the platform math library (`ln`, `cos`); the RNG draw sequence itself is
  still exact.
- Poisson vectors cover both Knuth (`lambda <= 10`) and PTRS (`lambda > 10`)
  branches, including a large `lambda` and a value just above the branch
  boundary.
- If a golden fails on one CI platform, treat it as a bug rather than silently
  loosening the test. Regenerate only after confirming the C reference behaves
  the same way on that platform.
