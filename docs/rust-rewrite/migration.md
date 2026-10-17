# Migration notes

> Status (2026-10-17): phases 0–5 are implemented and verified by
> `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and the full
> test suite on the local GNU toolchain. Phase 6 is intentionally pending the
> interactive TUI smoke test in a real terminal (the automated environment is
> non-interactive); `legacy/c/` is retained until that check passes. CI covers
> Windows/MSVC, Linux and macOS builds and tests.

## Phase plan

1. **Scaffold + legacy move** — C sources moved to `legacy/c/`, Cargo package,
   toolchain/config files, CI, docs and shared module interfaces created.
2. **Core** — xoshiro256**, distributions, parameters/fields, statistics,
   `%g` formatting and CSV export; golden vectors generated from C.
3. **Render model** — `Frame`, glyph sets, layout, chrome, controls, results
   and headless snapshot tests.
4. **Graphs** — half-block canvas, axes, and the four distribution renderers;
   snapshots at 60x22/80x24 in Unicode and ASCII.
5. **TUI** — crossterm terminal layer, event loop, resize/too-small handling,
   signal/panic restoration; manual smoke test.
6. **Cleanup** — remove `legacy/`, stale C-specific ignore entries and CMake
   references; update the screenshot if desired; tag the parity commit.

Each phase is an independent commit so `git revert`/`git checkout` can return
to a known state. Do not delete `legacy/c/` until the parity checklist and
manual smoke test are complete.

## Golden vector regeneration

The generator includes the C implementation directly so static engine helpers
are callable:

```bash
cd legacy/c/tools
gcc -O2 -std=c99 -o dump_sequences dump_sequences.c -lm
```

PowerShell example (ASCII/CRLF-safe):

```powershell
.\dump_sequences.exe engine 42 20 |
    Set-Content -Encoding ascii tests\fixtures\engine_seed42.golden
.\dump_sequences.exe uniform 42 20 0 100 |
    Set-Content -Encoding ascii tests\fixtures\uniform_seed42.golden
.\dump_sequences.exe normal 42 10 0 1 |
    Set-Content -Encoding ascii tests\fixtures\normal_seed42.golden
.\dump_sequences.exe bernoulli 42 20 0.5 |
    Set-Content -Encoding ascii tests\fixtures\bernoulli_seed42.golden
.\dump_sequences.exe poisson 42 20 1 |
    Set-Content -Encoding ascii tests\fixtures\poisson_seed42_lambda1.golden
.\dump_sequences.exe poisson 42 20 10 |
    Set-Content -Encoding ascii tests\fixtures\poisson_seed42_lambda10.golden
.\dump_sequences.exe poisson 42 20 10.0001 |
    Set-Content -Encoding ascii tests\fixtures\poisson_seed42_lambda10_0001.golden
.\dump_sequences.exe poisson 42 20 100 |
    Set-Content -Encoding ascii tests\fixtures\poisson_seed42_lambda100.golden
.\dump_sequences.exe poisson 42 10 1000000 |
    Set-Content -Encoding ascii tests\fixtures\poisson_seed42_lambda1000000.golden
```

From `legacy/c/tools`, the `.exe` paths are local to that directory; adjust
them when running from the repository root. Unix shells can use `>` directly.

See `tests/fixtures/README.md` for the parity policy.

## Build and test

The committed `rust-toolchain.toml` selects the stable channel and CI covers
Windows (MSVC), Linux and macOS. Rust 1.85 is the declared MSRV.

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

If a Windows machine has MinGW but no MSVC linker, install the GNU host
toolchain (`rustup toolchain install stable-x86_64-pc-windows-gnu`) and run
Cargo with `RUSTUP_TOOLCHAIN=stable-x86_64-pc-windows-gnu`; CI still verifies
the MSVC target.

Snapshots:

```bash
cargo test --test render_snapshots
cargo insta review
```

## Legacy removal gate

Delete `legacy/` only after all of the following:

- [ ] All automated tests pass on the three CI platforms.
- [ ] Golden vectors and snapshots are committed.
- [ ] Manual smoke test in `parity-checklist.md` passes.
- [ ] README build/platform/structure sections are updated.
- [ ] `.gitignore` no longer needs C build entries.
- [ ] No tracked file references `legacy/c` except historical docs.

## Rollback

- Before the deletion commit: revert individual phase commits or check out the
  previous commit; `legacy/c` is still present.
- After deletion: restore the C tree from the branch history (the move was a
  rename, so history is preserved) or from `master`.
