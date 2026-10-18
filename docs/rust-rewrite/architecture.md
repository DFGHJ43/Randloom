# Rust rewrite architecture

## Goals

- Keep the original TUI behavior and visual geometry: three panels, four
  distributions, key bindings, seed reproducibility and CSV export.
- Separate the terminal-free domain/rendering logic from the crossterm
  terminal layer so most of the app can be tested headlessly.
- Keep `src/` free of `unsafe` (`#![forbid(unsafe_code)]`).
- Preserve the ability to compare against and regenerate data from the original
  C implementation, which remains available on the `master` branch.

## Package layout

The project is one Cargo package with a library and a binary:

- `src/lib.rs` — public modules and `run()`; `src/main.rs` only calls it.
- `src/run.rs` — terminal bootstrap, event loop, resize and too-small handling.
- `src/app.rs` — `AppState`; consumes `Key` events, owns parameters/results.
- `src/config.rs` — `DistType`, `UniformMode`, `FieldId`, field buffers, parsing
  and clamps.
- `src/rng.rs` — xoshiro256** state, SplitMix64 seeding and `f64` helpers.
- `src/dist/` — one module per distribution plus the dispatch in `mod.rs`.
- `src/ui/` — headless `Frame`/`Layout`/glyph model, panels and graph renderers.
- `src/term/` — the only crossterm user: raw mode, alternate screen, event
  mapping, ANSI painting and signal flags.
- `src/stats.rs`, `src/fmt.rs`, `src/export.rs` — library helpers used by the
  app/tests and available for future non-interactive modes.

## Dependency direction

```text
main -> run -> { app, ui, term }
app  -> { config, dist -> rng, export -> fmt, input }
ui   -> { app, config, fmt }
term -> { input, ui::Frame }
run  -> { input, term, ui, app }
```

Rules:

- Only `term` may use `crossterm`.
- Only `export` may use `chrono`.
- `ui` never does terminal I/O; it writes into an in-memory `Frame`.
- `app` never draws or reads keys from the OS.
- `rng` has no global mutable state; `AppState::generate` creates a fresh
  `Rng` from the effective seed.

## Data flow

1. `run` checks TTY, size and installs the `Terminal` guard, panic hook and
   signal handlers.
2. Each loop iteration checks interrupt/size, renders `ui::render(&state, ...)`
   into a `Frame`, and sends it to `term::paint`.
3. `term::poll_event` maps crossterm key/resize events to `InputEvent`; `run`
   forwards `InputEvent::Key` to `AppState::handle_key`.
4. `AppState` parses fields, generates results (`dist::generate`), exports CSV,
   switches distributions, toggles the uniform mode and updates the status line.
5. On any exit path (`q`/Esc/Ctrl+C, SIGINT/SIGTERM, error, panic, Drop), the
   terminal guard restores the cursor, line wrap, alternate screen and raw mode.

## Terminal lifecycle

- `enable_raw_mode` + alternate screen + hidden cursor + `DisableLineWrap`.
- `KeyEventKind::Press` and `Repeat` are accepted; `Release` is ignored.
- Ctrl+C is a normal key event in raw mode; Unix SIGINT/SIGTERM set an atomic
  flag via `signal-hook`, which the loop polls.
- A panic hook restores the terminal before the default panic output.
- `RANDLOOM_ASCII=1` selects the ASCII glyph set; otherwise Unicode is used.
- A full frame is encoded into one reusable `Vec<u8>` and written/flushed once,
  avoiding per-cell terminal calls.

## UI model

- `Layout::compute(cols, rows)` mirrors the C formulas exactly, including the
  `60x22` minimum and the `gh`/`gpw` floors.
- `Frame` stores `Cell { ch, style }`; all glyphs are one terminal cell wide.
- `ui::render` draws the box/titles, controls, results, graph area and status
  line in the same order as C.
- Graph rendering uses a half-block `Canvas` (two half-cells per character
  cell), `axes`, and one renderer per distribution. ASCII mode emits `#` for a
  touched cell and `+ - | >` for the frame/axes. The uniform renderer branches
  on the mode: integer mode keeps the C panel; continuous mode uses `%g` bounds
  and a `%g` density legend for the PDF value `1 / (max - min)`.
- The controls panel shows a mode checkbox row one blank row below the last
  parameter field (between the fields and the action buttons) only for Uniform,
  aligned with the field labels: `Continuous(C): [ ]` / `Continuous(C): [x]`.
  The 18-cell row includes the `(C)` key hint only when the left panel is wide
  enough; narrower panels fall back to `Continuous: [ ]`/`[x]` rather than
  clipping. `C` toggles the mode only while Uniform is selected.
- `ui::render_to_string` is the headless snapshot entry point; styles are
  tested separately through `Frame` cells.

## RNG and distributions

- `Rng::next_u64` is a direct xoshiro256** port; `next_f64` uses the top 53
  bits and `next_f64_open` adds the half-step used by PTRS.
- Seeding uses SplitMix64, so a zero seed also produces a valid non-zero state;
  the application resolves seed 0 to the clock before seeding.
- Uniform has two modes selected by `UniformMode`. Integer mode (the default)
  truncates real Min/Max toward zero and uses 64-bit rejection sampling
  (unbiased over the full `i32` range). Continuous mode maps the 53-bit
  `next_f64` output linearly onto `[min, max)`; reversed bounds are swapped and
  a zero-width span returns the bound. Both modes consume the same
  xoshiro256** stream, so a fixed seed is reproducible in either mode.
- Normal uses Box-Muller with the same draw order as C.
- Bernoulli is `next_f64() < p` after clamping.
- Poisson uses Knuth for `lambda <= 10` and PTRS otherwise. PTRS needs
  `ln(n!)`; since Rust's `f64` has no `lgamma`, `dist/poisson.rs` uses a direct
  sum for small `n` and the Stirling series for large `n`.

## Configuration and export

- Field buffers are ASCII-only and capped at `FIELD_BUF_LEN - 1` characters;
  numeric fields accept digits, `-` and `.`, so Min/Max can hold fractional
  bounds.
- Parsing is lenient like C: empty/invalid values become 0; overflow saturates
  (`i32` clamps, `u64` saturates); then `Params::validate` applies the C clamps.
- `Params` stores `min`/`max` as `f64`. Integer uniform truncates them toward
  zero via `uniform_i32_bounds`; continuous uniform sorts the real endpoints via
  `uniform_real_bounds`, swapping reversed bounds. The continuous sampler steps
  to the next representable value below `max` if rounding would land exactly on
  the excluded upper bound (adjacent `f64` bounds).
- `AppState` captures the current `value_style` into `result_style` when it
  generates results; the result list and CSV export use that stored style, so
  switching distributions gives existing results a new graph but not a new
  number format.
- Export writes `randloom_export_YYYYMMDD_HHMMSS.csv`, one value per line and no
  header, using local time from `chrono`. The CSV uses the same `NumberStyle`
  as the result list: compact `%g` for integer-valued distributions,
  `Decimal6` for Normal and continuous uniform.
- `fmt::g` reproduces C `%g` (six significant digits, exponent rules, trailing
  zero removal) so integer results and golden output stay stable.
  `fmt::decimal6` renders exactly six decimals and falls back to scientific
  notation for non-zero values below `1e-6`, so tiny values never collapse to
  `0.000000`.

## Invariants

- `#![forbid(unsafe_code)]` in both the library and binary.
- No terminal I/O in `ui` or the core modules.
- No global mutable application/RNG state (only signal flags are process-wide).
- Every glyph used by the UI is a single terminal cell.
- Resizing and too-small terminals must never leave a stale or garbled frame.
