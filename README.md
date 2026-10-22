# Ferrite

A fast, lightweight, terminal-native system monitor built in Rust, featuring
real-time resource metrics, process inspection, and configurable alerts.

> Monitor your system. Understand your performance. Stay in the terminal.

## Development status

**Early stage — interactive dashboard, no alerting or history yet.**
Running the executable now opens a live terminal dashboard with CPU and
memory gauges and a sortable, navigable process table. There is still no
disk or network statistics, no alerting, and no historical storage.

## Project goals

- Provide real-time CPU, memory, disk, and network metrics in the terminal.
- Offer process inspection comparable to tools like `htop` or `btop`.
- Support configurable alerts for resource thresholds.
- Stay lightweight, fast, and dependency-conscious.
- Remain fully terminal-native, with no GUI requirement.

## Features implemented now

- A validated application configuration type (`AppConfig`) with a bounded,
  documented refresh interval, used to drive the dashboard's refresh rate.
- A system metrics collector (`collector::Collector`) that reports global
  CPU usage, total/used memory, and a process list (PID, name, CPU usage,
  memory usage).
- An interactive terminal dashboard (`cargo run`) with:
  - Live CPU and memory gauges.
  - A process table sortable by CPU or memory usage, with keyboard
    navigation and a clamped, disappearance-safe selection.
  - Pause/resume of metric refreshing, and a help overlay.
  - A resize-safe layout that falls back to a message instead of
    panicking when the terminal is too small.
  - Reliable terminal restoration on quit, on I/O error, and on panic
    (see [`ui::terminal`](src/ui/terminal.rs)).
- Explicit handling of CPU-sample initialization: the dashboard shows
  "collecting…" instead of a misleading number until CPU usage reflects a
  real timed sample. See `MetricsSnapshot::cpu_metrics_initialized`.
- A library crate (`ferrite`) exposing the configuration, collector, and
  `ui` APIs.
- A unit and integration test suite covering configuration, collector, and
  dashboard state/layout behavior.
- Continuous integration running formatting, linting, and tests on every
  push and pull request.

## Planned features (not yet implemented)

- Disk and network statistics.
- Background/periodic data collection decoupled from the render loop (the
  collector currently samples synchronously, once per refresh tick).
- Configurable alerts, historical data, and data export.
- A command-line flag to override the refresh interval (it is currently
  fixed to `AppConfig::default()`).

## Technology stack

- **Language:** Rust (2024 edition)
- **Dependencies:**
  - [`sysinfo`](https://docs.rs/sysinfo) `0.39`, with only its `system`
    feature enabled (CPU, memory, and process data).
  - [`ratatui`](https://docs.rs/ratatui) `0.30`, with only its `std` and
    `crossterm` features enabled.
  - [`crossterm`](https://docs.rs/crossterm) `0.29` (terminal backend,
    input events, raw mode), at the version `ratatui`'s `crossterm`
    feature depends on.
  - Default features not listed above (disk/network/user/component
    collection, the calendar widget, serde support, alternate backends,
    etc.) are disabled, since they are out of scope for this milestone.

## Prerequisites

- A stable Rust toolchain (edition 2024 support requires Rust 1.85 or later).
  Install via [rustup](https://rustup.rs/).

## Build and run

```bash
cargo build
cargo run
```

`cargo run` launches the dashboard directly in your terminal (it needs a
real terminal — it will fail with an I/O error if stdout isn't one, such as
when piped). Controls:

| Key(s)                | Action                            |
| --------------------- | ---------------------------------- |
| `q` / `Esc` / `Ctrl+C` | Quit                                |
| `j` / `↓`              | Select next process                |
| `k` / `↑`              | Select previous process            |
| `c`                    | Sort the process table by CPU      |
| `m`                    | Sort the process table by memory   |
| `p`                    | Pause / resume metric refreshing   |
| `?`                    | Toggle the help overlay            |

Raw mode (which the dashboard enables) means the terminal no longer turns
`Ctrl+C` into a `SIGINT`; it arrives as a normal key press, which is why
it's handled explicitly rather than relying on signal delivery.

## Formatting, linting, and testing

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
```

## Architecture

The project is split into a library crate and a thin executable:

```
src/
├── main.rs            # Executable entry point: the dashboard event loop.
├── lib.rs             # Public library API surface.
├── config.rs          # Validated AppConfig type and its tests.
├── collector/         # System metrics collection.
│   ├── mod.rs         # Public module surface.
│   ├── cpu.rs         # CpuStats: global usage + core count.
│   ├── memory.rs      # MemoryStats: byte counts and unit conversions.
│   ├── process.rs     # ProcessInfo and deterministic CPU/memory sorts.
│   ├── snapshot.rs    # MetricsSnapshot bundling cpu/memory/processes.
│   └── system.rs      # Collector: the only module that calls into
│                       # sysinfo / the operating system.
└── ui/                # The interactive terminal dashboard.
    ├── mod.rs         # Public module surface.
    ├── app.rs         # App: selection, sort mode, pause/help state.
    ├── events.rs      # Key-to-Action mapping and action application.
    ├── dashboard.rs   # Layout math and widget rendering.
    └── terminal.rs    # Terminal init/restore lifecycle (ratatui).
tests/
├── config_tests.rs    # Integration tests for the public config API.
└── collector_tests.rs # Integration tests for the public collector API.
```

`src/config.rs`, every type in `src/collector/` other than
`collector::system`, and every part of `src/ui/` other than
`ui::terminal` are plain data types or pure functions with no dependency
on a real terminal or operating-system API, so they are unit tested in
isolation — application state transitions (`ui::app`), key mapping
(`ui::events`), and layout/ratio math (`ui::dashboard`) all have tests
that don't touch a real terminal or collector.

`collector::system` and `ui::terminal` are the only modules that talk to
the operating system or a real terminal. `collector::system` is covered by
integration tests that assert invariants (e.g. "used memory never exceeds
total memory") rather than exact values, since those depend on the host
machine. `ui::terminal` has no automated test, since doing so would
require driving a real terminal session; it is exercised by running the
dashboard directly.

## Contributing

Contributions are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for setup
instructions, coding conventions, and how to submit a pull request.

## Platform support

Ferrite is built on [`sysinfo`](https://docs.rs/sysinfo) for metrics and
[`ratatui`](https://docs.rs/ratatui) + [`crossterm`](https://docs.rs/crossterm)
for the terminal UI; all three support Linux, macOS, and Windows. This
milestone has been built, linted, and tested on Windows only; Linux and
macOS are expected to work (continuous integration runs on Ubuntu) but have
not been verified on real hardware as part of this milestone. The
dashboard itself was launched on Windows against a real console session
and confirmed to render live CPU, memory, and process data correctly, but
keyboard interaction could not be scripted end-to-end in the automated
environment this milestone was built in (its terminal reads console input
through the OS console API, which doesn't accept piped input) — a human
should confirm navigation, sorting, pause, and quit interactively.

## License

Ferrite is licensed under the [MIT License](LICENSE).
