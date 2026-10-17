# C → Rust parity checklist

This checklist tracks the behavior the Rust rewrite preserves from the original
C implementation (available on `master`), plus the small number of intentional
deviations.

> Manual TUI smoke test: completed by the maintainer on 2026-10-17 on
> Windows 10/11 before the legacy C tree was removed.

## Module mapping

| C | Rust |
| --- | --- |
| `tui.c/h` | `run.rs`, `app.rs`, `config.rs`, `input.rs` |
| `term.c/h` | `term/{mod,keys,paint,signals}.rs` (crossterm) |
| `chars.h` | `ui/glyphs.rs` |
| `controls.c/h` | `ui/controls.rs`, `config.rs`, `app.rs` |
| `random.c/h` | `rng.rs`, `dist/{uniform,normal,bernoulli,poisson}.rs` |
| `stats.c/h` | `stats.rs` |
| `output.c/h` | `export.rs`, `fmt.rs` |
| `graph/graph.c/h` | `ui/graph/mod.rs` |
| `graph/graph_axes.h` | `ui/graph/axes.rs` |
| `graph/block.c/h` | `ui/graph/canvas.rs`, `ui/frame.rs` |
| `graph/graph_*.c/h` | `ui/graph/{uniform,normal,bernoulli,poisson}.rs` |
| `CMakeLists.txt` | `Cargo.toml` (C reference on `master`) |

## Behavior checklist

- [x] Layout: content width split `22/32/remainder`, `gy=4`,
      `gh=max(rows-14,6)`, `gpx=mx+2`, `gpw=max(mw-4,10)`,
      `status_y=rows-2`, `result_max_show=max(rows-14,1)`.
- [x] Minimum terminal 60x22; too-small startup message; runtime overlay and
      wait-for-resize.
- [x] Box, titles, status bar, focus reverse video and bold status style.
- [x] Distribution selector labels/hints and radio state.
- [x] Eight fields in Tab order Count → Seed → visible distribution parameters;
      invisible fields skipped and cleared.
- [x] Field labels, bracket geometry, value width `max(lw-12,1)`, 31-char cap.
- [x] Seed accepts digits only; numeric fields accept digits, `-`, `.`.
- [x] Parsing/clamps: count 1..1000, prob 0..1, stddev default 1, lambda default
      1 and max 1e9, seed saturation.
- [x] Results: clear rows, `(no results)`, `%3d: <number>`, scroll clamp,
      `(N total, arrows)` hint truncated to panel width.
- [x] Number display: Normal and continuous uniform results use six decimals
      (scientific for non-zero values below `1e-6`); integer uniform, Bernoulli
      and Poisson keep C `%g`; the style is stored with the results at
      generation time, and CSV uses that same style.
- [x] Graph area clearing, axes, titles, value legends and x-axis labels.
- [x] Uniform integer mode (default): inclusive `[min, max]`, fractional
      Min/Max truncated toward zero, reversed bounds swapped.
- [x] Uniform continuous mode: real `[min, max)` using the 53-bit `next_f64`
      stream; reversed bounds swapped; zero-width range returns the bound;
      values stay strictly below `max` even for adjacent `f64` bounds.
- [x] `C` toggles the uniform mode only for Uniform; controls checkbox row
      `[ ]`/`[x] Continuous (C)` (the `(C)` hint is omitted, never clipped,
      when the left panel cannot fit the 18-cell row); status messages
      `Uniform mode: ...` and `Gen N continuous uniform numbers. Seed ...`.
- [x] Uniform: flat filled rectangle; integer labels min/max and two-decimal
      PDF legend; continuous `%g` title/labels and a `%g` legend for the PDF
      value `1 / (max - min)`.
- [x] Normal: bell curve sampled per half-cell column; mu/sigma title and
      labels; PDF legend.
- [x] Bernoulli: dual bars with C proportions, labels and probability values.
- [x] Poisson: PMF bars, 64-entry PMF cap, sampling step when bars overflow,
      `k_max`/lambda labels, PMF legend.
- [x] Unicode block/box glyphs and ASCII fallback (`#`, `+ - | >`).
- [x] Keys: G/E, U/N/B/P, C (uniform mode), Tab, Up/Down, Left/Right,
      Backspace, Q/Esc; Ctrl+C quits and restores.
- [x] Generate: parse fields, count clamp, auto seed, status line with actual
      seed and `(auto)`, reset scroll.
- [x] Export: empty-result message, timestamped filename, one formatted value
      per line (`%g` or six-decimal style), failure message.
- [x] Engine and integer distributions match C golden vectors exactly;
      continuous uniform with `[0, 1)` matches the `f64_seed42` engine vector.
- [x] Normal golden vectors match within floating-point tolerance.
- [x] Terminal restore on normal exit, Ctrl+C, Unix SIGINT/SIGTERM, panic,
      and error paths.
- [x] TTY check (stdin/stdout) before entering raw mode.
- [x] Terminal resize while idle and while the too-small overlay is shown.

## Intentional deviations

| Deviation | Reason |
| --- | --- |
| Windows 10+ only | Rust 1.78+ baseline; Windows 7/8 users keep the `master` C build. |
| Runtime dependencies `crossterm`/`chrono` (plus `signal-hook` on Unix) | Replaces platform ANSI/Win32/termios and local-time code. README no longer claims zero dependencies; RNG remains dependency-free. |
| Alternate screen | Exiting returns to the previous shell content instead of clearing the main screen. Running UI is unchanged. |
| Unicode always used unless `RANDLOOM_ASCII=1` | Rust/crossterm writes Unicode through the console; Windows code-page probing is no longer needed. The ASCII renderer remains for compatibility/tests. |
| Normal values are tolerance-matched, not bit-matched, across platforms | `ln`/`cos` come from the platform math library. Integer distributions remain exact. |
| Uniform continuous mode | New extension not present in the C reference; adds a checkbox row under the Uniform selector. Integer mode remains the default, so sampling output and golden vectors are unchanged. |
| Normal and continuous uniform results use six decimals (scientific for non-zero values below `1e-6`) | C used `%g` for every result; stable decimal places keep continuous values readable. Integer-valued distributions retain `%g`. |
| CSV follows the per-distribution result style | C wrote `%g` for every value; matching the TUI avoids a second formatting rule. |
| Result lines are clipped to the panel width | Prevents long values from overwriting the border at 60 columns. |
| CSV write errors are reported | C ignored write errors after a successful open. |
| `i32`/`u64` parsing clamps/saturates explicitly | Avoids C implementation-defined overflow conversions while preserving normal input behavior. |
| C `output_console`/`output_stats` helpers are not ported | They were unused by the TUI; `stats` remains a library API for future non-interactive use. |
| No CLI subcommands | Out of scope for the parity rewrite. |

## Automated verification

- `tests/rng_golden.rs`: raw `u64` and `f64` bit sequences from C.
- `tests/distributions.rs`: exact uniform/Bernoulli/Poisson golden vectors,
  continuous uniform via the `f64_seed42` engine vector, normal tolerance, and
  statistical sanity checks.
- `tests/fmt.rs`: `%g` cases captured from C `printf`, plus six-decimal and
  scientific display cases.
- `tests/config.rs`, `tests/app_keys.rs`, `tests/export.rs`: parsing,
  real-valued uniform bounds, visibility, editing, key actions including the
  uniform mode, scrolling, filenames and both CSV number styles.
- `tests/render_snapshots.rs`: 80x24 and 60x22 layouts for all distributions,
  continuous uniform layouts, Unicode and ASCII, plus focus/empty/scroll
  states.
- `src/ui/{layout,frame,graph/canvas}.rs` unit tests: geometry and half-cell
  bounds.

## Manual smoke test

1. Windows 10/11 (Windows Terminal and classic console) and Linux/macOS.
2. Generate all four distributions; toggle continuous uniform with `C` and
   verify its graph and number display. Compare the integer-mode graph and
   sampling against the C build from `master` at 80x24 and 60x22.
3. Fixed seed 42 twice → identical integer results; seed 0 twice → status shows
   different auto seeds and results differ; Tab order/edit/backspace work.
4. Resize to below 60x22 and back; the overlay appears/disappears and no stale
   frame remains.
5. Export CSV, verify contents; remove write permission/target a directory and
   verify the "Export failed!" status.
6. Quit with Q, Esc, Ctrl+C and SIGTERM; verify cursor/raw mode/alternate
   screen are restored. Trigger a panic only in a throwaway build to verify the
   restore hook.
