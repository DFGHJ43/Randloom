# Migration notes

> Status (2026-10-17): rewrite complete. Phases 0–6 are done; the automated test
> suite and the manual TUI smoke test passed. `legacy/c/` has been removed. The
> original C implementation remains on `master`; the rewrite snapshot that still
> contains the C tree and the temporary golden harness is commit `2ac8793`.

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
6. **Cleanup** — removed `legacy/`, stale C-specific ignore entries and CMake
   references.

Each phase was committed so `git revert`/`git checkout` can return to a known
state.

## Golden vector regeneration

The committed fixtures are stable and do not normally need regeneration. They
were produced from the C implementation on `master` with a temporary harness.
The harness is preserved at commit `2ac8793` under
`legacy/c/tools/dump_sequences.c`.

```bash
git worktree add ../randloom-c master
mkdir -p ../randloom-c/tools
git show 2ac8793:legacy/c/tools/dump_sequences.c > ../randloom-c/tools/dump_sequences.c
cd ../randloom-c/tools
gcc -O2 -std=c99 -o dump_sequences dump_sequences.c -lm
```

The harness includes `../src/random.c` from the C worktree, so it can call the
static engine helpers (`next_u64`, `next_double`, `next_double_open`) as well as
the public distribution functions. Run the modes from the Rust repository root
and write the output into `tests/fixtures/` (PowerShell shown; adjust the
worktree path as needed):

```powershell
$dump = "..\randloom-c\tools\dump_sequences.exe"
& $dump engine 42 20 | Set-Content -Encoding ascii tests\fixtures\engine_seed42.golden
& $dump uniform 42 20 0 100 | Set-Content -Encoding ascii tests\fixtures\uniform_seed42.golden
& $dump normal 42 10 0 1 | Set-Content -Encoding ascii tests\fixtures\normal_seed42.golden
& $dump bernoulli 42 20 0.5 | Set-Content -Encoding ascii tests\fixtures\bernoulli_seed42.golden
& $dump poisson 42 20 1 | Set-Content -Encoding ascii tests\fixtures\poisson_seed42_lambda1.golden
& $dump poisson 42 20 10 | Set-Content -Encoding ascii tests\fixtures\poisson_seed42_lambda10.golden
& $dump poisson 42 20 10.0001 | Set-Content -Encoding ascii tests\fixtures\poisson_seed42_lambda10_0001.golden
& $dump poisson 42 20 100 | Set-Content -Encoding ascii tests\fixtures\poisson_seed42_lambda100.golden
& $dump poisson 42 10 1000000 | Set-Content -Encoding ascii tests\fixtures\poisson_seed42_lambda1000000.golden
```

Unix shells can use `>` directly. See `tests/fixtures/README.md` for the parity
policy.

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

## Cleanup completion

- [x] All automated tests pass.
- [x] Golden vectors and snapshots are committed.
- [x] Manual smoke test in `parity-checklist.md` passed.
- [x] README build/platform/structure sections are updated.
- [x] `.gitignore` no longer needs C build entries.
- [x] No tracked file depends on `legacy/c`.

The original C implementation remains available on `master`. To recover the
pre-cleanup tree exactly as it was during the rewrite, check out commit
`2ac8793`.

## Rollback

- To inspect the rewrite with the C reference tree: `git worktree add ../randloom-c 2ac8793`.
- To return to the original C application: switch to `master`.
