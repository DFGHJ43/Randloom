# Golden fixture provenance

These files were generated from the original C implementation (available on
`master`) with a temporary harness on Windows using MinGW GCC. The harness
included the C reference's `src/random.c` directly so it could call the static
engine helpers (`next_u64`, `next_double`, `next_double_open`) as well as the
public distribution functions. The harness is preserved at commit `2ac8793`
under `legacy/c/tools/dump_sequences.c`; regeneration commands are in
`docs/rust-rewrite/migration.md`.

The files are plain whitespace-separated numeric values (one per line),
committed with LF line endings via `.gitattributes`.

## Parity policy

- `engine_seed42.golden`, `f64_seed42.golden`, `uniform_seed42.golden`,
  `bernoulli_seed42.golden`, and the `poisson_*.golden` files must match the
  Rust implementation exactly (bit-for-bit for floats).
- `normal_seed42.golden` is compared with a small tolerance because Box-Muller
  uses the platform math library (`ln`, `cos`); the RNG draw sequence itself is
  still exact.
- The continuous uniform sampler has no separate fixture: with `[0, 1)` bounds
  it is exactly `next_f64`, so `tests/distributions.rs` checks it bit-for-bit
  against `f64_seed42.golden`.
- Poisson vectors cover both Knuth (`lambda <= 10`) and PTRS (`lambda > 10`)
  branches, including a large `lambda` and a value just above the branch
  boundary.
- If a golden fails on one CI platform, treat it as a bug rather than silently
  loosening the test. Regenerate only after confirming the C reference behaves
  the same way on that platform.
