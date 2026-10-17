# Legacy C implementation (reference only)

This directory contains the original C implementation of Randloom, kept for
behavioral comparison while the Rust rewrite is in progress. It is **not**
part of the CI build and will be deleted once the Rust implementation reaches
parity (see `docs/rust-rewrite/migration.md`).

Build it from this directory if a side-by-side reference binary is needed:

```bash
cmake -S . -B build
cmake --build build
```

`tools/dump_sequences.c` includes `src/random.c` directly and emits the golden
vectors used by `tests/fixtures/`. See `docs/rust-rewrite/migration.md` for the
regeneration commands.
