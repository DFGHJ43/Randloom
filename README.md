# Random Number Generator (RNG)

An interactive terminal-based random number generator written in C.
Zero external dependencies — uses only ANSI escape codes and the C standard library.

## Screenshot

![RNG TUI Screenshot](screenshot.png)

## Features

- Three-column TUI layout: controls | graph | results
- Uniform distribution: random integers in [min, max]
- Normal distribution: Box-Muller transform with configurable mean/stddev
- Bernoulli distribution: 0/1 outcomes with configurable probability p
- Poisson distribution: Knuth's algorithm with configurable rate λ
- Real-time ASCII graphs of probability density functions
- Export results to CSV with timestamp filenames
- Reproducible results: set a fixed Seed (0 = auto, uses current time)

## Build

```bash
gcc -Wall -Wextra -std=c99 -pedantic -Isrc -o rng-tui \
    src/tui.c src/term.c src/controls.c src/random.c \
    src/stats.c src/output.c \
    src/graph/graph.c src/graph/graph_uniform.c \
    src/graph/graph_normal.c src/graph/graph_bernoulli.c \
    src/graph/graph_poisson.c -lm
```

> Or `cmake` if you have it installed (uses the included CMakeLists.txt).

## Usage

```bash
./rng-tui
```

### Key Bindings

| Key            | Action                           |
| -------------- | -------------------------------- |
| `G`            | Generate random numbers          |
| `E`            | Export results to CSV            |
| `U`            | Switch to uniform distribution   |
| `N`            | Switch to normal distribution    |
| `B`            | Switch to Bernoulli distribution |
| `P`            | Switch to Poisson distribution  |
| `Tab`          | Cycle focus between input fields |
| `Up` `Down`    | Scroll results                   |
| `Left` `Right` | Move cursor in input fields      |
| `Backspace`    | Delete character in input field  |
| `Q` / `Esc`    | Quit                             |

### Parameter Fields

| Field          | Meaning                                                        |
| -------------- | -------------------------------------------------------------- |
| Count          | How many numbers to generate (1–1000)                          |
| Seed           | 0 = auto (time-based); non-zero = fixed, sequence reproducible |
| Min / Max      | Uniform range `[min, max]`                                     |
| Mean / StdDev  | Normal mean and standard deviation                             |
| Prob           | Bernoulli probability p in [0, 1]                              |
| Lambda         | Poisson rate λ                                                 |

> Reproducibility: the same Seed with the same distribution, parameters,
> and Count yields the identical sequence on every run and platform.

## Project Structure

```
src/
├── tui.c/h                     Main event loop, frame rendering, results panel
├── term.c/h                    Terminal control (ANSI) + keyboard input
├── controls.c/h                Left panel: parameter fields, field editing
├── random.c/h                  Random number generation (xoshiro256** + SplitMix64)
├── stats.c/h                   Statistics (min, max, mean, stddev)
├── output.c/h                  Output to console / CSV file
└── graph/
    ├── graph.c/h               Graph area dispatch (clear + route)
    ├── graph_uniform.c/h       Uniform distribution PDF graph
    ├── graph_normal.c/h        Normal distribution PDF bell curve
    ├── graph_bernoulli.c/h     Bernoulli distribution PMF dual-bar
    └── graph_poisson.c/h       Poisson distribution PMF bar chart
```

## License

MIT
