# Ferrite

A fast, lightweight, terminal-native system monitor built in Rust, featuring
real-time resource metrics, process inspection, and configurable alerts.

> Monitor your system. Understand your performance. Stay in the terminal.

## Development status

**Early stage — no terminal dashboard yet.** Ferrite can now collect CPU,
memory, and process metrics through its library API, but nothing displays
them yet: there is no terminal dashboard, no disk or network statistics, no
alerting, and no historical storage.

Running the executable still prints a short startup message and exits — it
does not display live system metrics.

## Project goals

- Provide real-time CPU, memory, disk, and network metrics in the terminal.
- Offer process inspection comparable to tools like `htop` or `btop`.
- Support configurable alerts for resource thresholds.
- Stay lightweight, fast, and dependency-conscious.
- Remain fully terminal-native, with no GUI requirement.

## Features implemented now

- A validated application configuration type (`AppConfig`) with a bounded,
  documented refresh interval.
- A system metrics collector (`collector::Collector`) that reports:
  - Global CPU usage and logical core count.
  - Total and used memory, in bytes (with mebibyte conversion helpers).
  - A process list (PID, name, CPU usage, memory usage), deterministically
    sorted by CPU usage with an ascending-PID tie-break.
- Explicit handling of CPU-sample initialization: a snapshot reports
  whether its CPU figures reflect a real timed sample, since the first
  sample after startup cannot yet be meaningful. See the doc comments on
  `MetricsSnapshot::cpu_metrics_initialized` for details.
- A library crate (`ferrite`) exposing the configuration and collector
  APIs.
- A minimal executable that reports its own early-development status (it
  does not yet call the collector or display metrics).
- A unit and integration test suite covering configuration and collector
  behavior.
- Continuous integration running formatting, linting, and tests on every
  push and pull request.

## Planned features (not yet implemented)

- A `ratatui`-based terminal dashboard that displays the collected metrics.
- Disk and network statistics.
- Background/periodic data collection (the collector currently samples
  on demand, once per call).
- Configurable alerts, historical data, and data export.

## Technology stack

- **Language:** Rust (2024 edition)
- **Dependencies:** [`sysinfo`](https://docs.rs/sysinfo) `0.39`, with only
  its `system` feature enabled (CPU, memory, and process data). Disk,
  network, user, and component features are disabled since they are out of
  scope for this milestone.
- A planned future milestone is expected to introduce a terminal UI crate
  (such as `ratatui`) for the dashboard.

## Prerequisites

- A stable Rust toolchain (edition 2024 support requires Rust 1.85 or later).
  Install via [rustup](https://rustup.rs/).

## Build and run

```bash
cargo build
cargo run
```

## Formatting, linting, and testing

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
```

## Architecture

The project is currently split into a library crate and a thin executable:

```
src/
├── main.rs            # Executable entry point; prints a startup message.
├── lib.rs             # Public library API surface.
├── config.rs          # Validated AppConfig type and its tests.
└── collector/         # System metrics collection.
    ├── mod.rs         # Public module surface.
    ├── cpu.rs         # CpuStats: global usage + core count.
    ├── memory.rs      # MemoryStats: byte counts and unit conversions.
    ├── process.rs     # ProcessInfo and deterministic CPU-descending sort.
    ├── snapshot.rs    # MetricsSnapshot bundling cpu/memory/processes.
    └── system.rs      # Collector: the only module that calls into
                        # sysinfo / the operating system.
tests/
├── config_tests.rs    # Integration tests for the public config API.
└── collector_tests.rs # Integration tests for the public collector API.
```

`src/config.rs` and every type in `src/collector/` other than
`collector::system` are plain data types or pure functions with no
dependency on terminal rendering or operating-system APIs, so they can be
tested in isolation. `collector::system` is the sole module that talks to
the operating system (via `sysinfo`); it is covered by integration tests
that assert invariants (e.g. "used memory never exceeds total memory")
rather than exact values, since those depend on the host machine. Future
milestones are expected to add process inspection detail, alerting, and the
terminal dashboard without requiring changes to this layer.

## Contributing

Contributions are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for setup
instructions, coding conventions, and how to submit a pull request.

## Platform support

Ferrite's metrics collector is built on [`sysinfo`](https://docs.rs/sysinfo),
which supports Linux, macOS, and Windows. This milestone has been built,
linted, and tested on Windows only; Linux and macOS are expected to work
(continuous integration runs on Ubuntu) but have not been manually verified
on real hardware as part of this milestone.

## License

Ferrite is licensed under the [MIT License](LICENSE).
