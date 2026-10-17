# Randloom

Randloom is an interactive terminal-based random number generator written in
Rust. It preserves the original C application's three-panel TUI, distribution
graphs, seed reproducibility and CSV export while replacing the platform layer
with [crossterm](https://crates.io/crates/crossterm) and the timestamp handling
with [chrono](https://crates.io/crates/chrono). The random number engine is a
self-contained xoshiro256** implementation; the crate contains no `unsafe` code.

## Screenshot

![Randloom TUI Screenshot](screenshot.png)

## Features

- Three-column TUI layout: controls | graph | results
- Uniform distribution: integer mode (the default, matching the C app) samples
  integers in [min, max]; continuous mode samples real numbers in [min, max).
  Press `C` while Uniform is selected to toggle; integer mode truncates decimal
  Min/Max values toward zero.
- Normal distribution: Box-Muller transform with configurable mean/stddev
- Bernoulli distribution: 0/1 outcomes with configurable probability p
- Poisson distribution: Knuth's algorithm (λ <= 10) and PTRS rejection
  sampling (λ > 10) with configurable rate λ
- Real-time graphs of the probability density / mass functions, drawn with
  Unicode block elements (█, ▀, ▄) and box-drawing axes, so curves and bars
  come out solid and smooth instead of stair-stepped
- Result numbers follow the distribution: Normal and continuous uniform values
  use exactly six decimal places, switching to scientific notation for non-zero
  values below 1e-6 (e.g. `1.234568e-07`); integer uniform, Bernoulli and
  Poisson results keep the compact C `%g` format
- Export results to CSV with timestamp filenames; the CSV uses the same
  per-distribution number style as the result list
- Reproducible results: set a fixed Seed (0 = auto, uses current time)

### Graph rendering

The plot is a half-block canvas: every character cell holds two vertically
stacked half-cells, rendered with `█` (full block), `▀` (upper half) and `▄`
(lower half). That gives a 1x2 sub-cell grid — twice the vertical resolution of
plain `#` art — while keeping the fill solid instead of dotted. Axes use
box-drawing characters (`│`, `─`, `└`, `→`) and the frame uses the
`┌ ┬ ┐ ├ ┤ └ ┴ ┘` family; labels and the value readout stay ASCII.

Where Unicode cannot be used, set `RANDLOOM_ASCII=1` to render the frame with
ASCII `+ - |` and graph cells with `#`.

## Build

Requires Rust 1.85 or newer (edition 2024).

```bash
cargo build --release
```

Run the binary:

```bash
target/release/randloom        # Linux / macOS
target\release\randloom.exe    # Windows
```

## Development

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

UI snapshots are managed with [insta](https://crates.io/crates/insta). After
changing rendering, review and update snapshots with:

```bash
cargo install cargo-insta   # once
cargo insta review
```

Golden vectors in `tests/fixtures/` were generated from the C reference
implementation; see `tests/fixtures/README.md` for the exact commands.

On Windows, either the MSVC build tools or the GNU toolchain can build the
project. The CI matrix uses the stable MSVC toolchain on `windows-latest`. If
the machine has MinGW but no MSVC linker:

```powershell
rustup toolchain install stable-x86_64-pc-windows-gnu
$env:RUSTUP_TOOLCHAIN = "stable-x86_64-pc-windows-gnu"
cargo run --release
```

Run the TUI from a real interactive terminal; redirected stdin/stdout is
rejected by design.

## Usage

```bash
./randloom
```

### Key Bindings

| Key            | Action                                 |
| -------------- | -------------------------------------- |
| `G`            | Generate random numbers                |
| `E`            | Export results to CSV                  |
| `U`            | Switch to uniform distribution         |
| `N`            | Switch to normal distribution          |
| `B`            | Switch to Bernoulli distribution       |
| `P`            | Switch to Poisson distribution         |
| `C`            | Toggle uniform integer/continuous mode |
| `Tab`          | Cycle focus between input fields       |
| `Up` `Down`    | Scroll results                         |
| `Left` `Right` | Move cursor in input fields            |
| `Backspace`    | Delete character in input field        |
| `Q` / `Esc`    | Quit                                   |
| `Ctrl+C`       | Quit (also restores the terminal)      |

### Parameter Fields

| Field          | Meaning                                                        |
| -------------- | -------------------------------------------------------------- |
| Count          | How many numbers to generate (1–1000)                          |
| Seed           | 0 = auto (time-based); non-zero = fixed, sequence reproducible |
| Min / Max      | Integer `[min, max]` (decimals truncated) or real `[min, max)` |
| Mean / StdDev  | Normal mean and standard deviation                             |
| Prob           | Bernoulli probability p in [0, 1]                              |
| Lambda         | Poisson rate λ                                                 |

> Reproducibility: the same Seed with the same distribution, parameters and
> Count yields the same sequence. The normal distribution uses the platform
> `f64` math library, so its last bits may differ between operating systems;
> integer uniform, Bernoulli and Poisson results are exact across platforms,
> and continuous uniform values are reproducible from the same Seed and Min/Max
> bounds. The status bar shows the seed actually used in auto mode.

## Project Structure

```text
src/
├── main.rs                     Thin binary entry point
├── lib.rs                      Public module map, #![forbid(unsafe_code)]
├── run.rs                      Terminal bootstrap, event loop, resize handling
├── app.rs                      Application state machine and key actions
├── config.rs                   Parameters, fields, parsing and validation
├── input.rs                    Terminal-independent Key / InputEvent / Flow
├── rng.rs                      xoshiro256** + SplitMix64 engine
├── stats.rs                    min / max / mean / population stddev
├── fmt.rs                      %g and six-decimal/scientific formatting
├── export.rs                   Timestamped single-column CSV export
├── dist/                       Uniform (int/real) / normal / Bernoulli / Poisson
├── term/                       crossterm terminal layer, input mapping, signals
└── ui/                         Headless Frame model, layout, panels and graphs
    ├── frame.rs, glyphs.rs, layout.rs
    ├── chrome.rs, controls.rs, results.rs
    └── graph/                  Half-block canvas, axes, per-distribution plots
tests/
├── fixtures/                   Golden vectors from the C reference
├── snapshots/                  insta UI snapshots
└── *.rs                        Parity, statistics, config, app and export tests
docs/rust-rewrite/              Architecture, parity checklist, migration notes
```

The original C implementation remains available on the `master` branch; the
Rust rewrite tree contains no C sources.

## Platform Notes

- **Windows 10 / 11**: crossterm enables VT processing in the console; works in
  Windows Terminal and the classic console host. Windows 7/8 are not supported
  by the Rust toolchain baseline (Rust 1.78+); use the C version on `master` if
  those systems are required.
- **Linux / macOS**: ANSI terminal output as usual.
- The TUI requires an interactive stdin/stdout; redirected input is rejected
  with a clear error.

## Dependencies

| Crate | Purpose | License |
| --- | --- | --- |
| `crossterm` | Cross-platform terminal I/O and events | MIT |
| `chrono` | Local timestamp for export filenames | MIT OR Apache-2.0 |
| `signal-hook` (Unix only) | Graceful SIGINT/SIGTERM handling | MIT OR Apache-2.0 |
| `insta` (dev only) | UI snapshot tests | Apache-2.0 |

The random number engine and all distributions are implemented in this
repository; there is no dependency on a random-number crate. `src/` contains no
`unsafe` code (`unsafe_code = "forbid"`).

## License

MIT
