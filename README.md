# Ferrite

A fast, lightweight, terminal-native system monitor built in Rust, featuring
real-time resource metrics, process inspection, and configurable alerts.

> Monitor your system. Understand your performance. Stay in the terminal.

## Development status

**Active development — core metrics, history, and alerting are in place;
export and remote telemetry are not.** Running the executable opens a live
terminal dashboard with CPU, memory, disk, and network panels, bounded
in-memory history sparklines, a sortable/navigable process table, and
configurable threshold alerts. Configuration can come from an optional
TOML file. There is still no data export and no background collection
decoupled from the render loop.

## Project goals

- Provide real-time CPU, memory, disk, and network metrics in the terminal.
- Offer process inspection comparable to tools like `htop` or `btop`.
- Support configurable alerts for resource thresholds.
- Stay lightweight, fast, and dependency-conscious.
- Remain fully terminal-native, with no GUI requirement.

## Features implemented now

- A validated application configuration type (`AppConfig`), covering the
  refresh interval, metric history capacity, CPU/memory/disk alert
  thresholds, and the default process sort mode — see
  [Configuration](#configuration).
- A system metrics collector (`collector::Collector`) that reports:
  - Global CPU usage and a process list (PID, name, CPU usage, memory
    usage).
  - Total/used memory.
  - Per-disk capacity and usage (total, available, used, used %).
  - Per-network-interface traffic: cumulative byte counters plus
    received/transmitted bytes-per-second, computed safely from
    successive samples (never negative, `None` rather than a misleading
    number on the first sample or after a counter reset — see
    `collector::bytes_per_second`).
- Bounded, in-memory-only history (`history::History`) for CPU and memory
  utilization, rendered as compact sparklines. History is **not**
  persisted anywhere and resets when Ferrite exits.
- Configurable threshold alerts (`alerts::AlertEngine`) for CPU, memory,
  and (independently, per mount point) disk usage, with clear
  activation/recovery semantics, no repeated spam while a threshold stays
  exceeded, and no false alerts from unavailable readings. Alerts only
  drive a dashboard indicator — they never run a command or send a
  notification.
- An interactive terminal dashboard (`cargo run`) with:
  - Live CPU and memory gauges, disk and network summary panels, and
    CPU/memory history sparklines.
  - A header alert badge showing the count of currently active alerts.
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
- A library crate (`ferrite`) exposing the configuration, collector,
  history, alerts, and `ui` APIs.
- A unit and integration test suite covering configuration, collector,
  history, alerts, and dashboard state/layout behavior.
- Continuous integration running formatting, linting, and tests on every
  push and pull request.

## Planned features (not yet implemented)

- Data export (e.g. to a file or another process).
- Background/periodic data collection decoupled from the render loop (the
  collector currently samples synchronously, once per refresh tick).
- A command-line flag to override configuration (configuration currently
  comes only from the file described below, or documented defaults).

## Technology stack

- **Language:** Rust (2024 edition)
- **Dependencies:**
  - [`sysinfo`](https://docs.rs/sysinfo) `0.39`, with its `system`,
    `disk`, and `network` features enabled.
  - [`ratatui`](https://docs.rs/ratatui) `0.30`, with only its `std` and
    `crossterm` features enabled.
  - [`crossterm`](https://docs.rs/crossterm) `0.29` (terminal backend,
    input events, raw mode), at the version `ratatui`'s `crossterm`
    feature depends on.
  - [`serde`](https://docs.rs/serde) `1.0` (with `derive`) and
    [`toml`](https://docs.rs/toml) `1.1`, for parsing the optional
    configuration file.
  - Default features not listed above (user/component collection, the
    calendar widget, alternate terminal backends, etc.) are disabled,
    since they are out of scope for this project.

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

## Configuration

Ferrite reads an **optional** TOML configuration file. If no file is
found, or any field is omitted, Ferrite uses the documented defaults
below — there is no requirement to create a file at all.

**Search path** (exactly one location is checked; there is no precedence
chain across multiple candidate paths):

| Platform | Path |
| --- | --- |
| Windows | `%APPDATA%\ferrite\config.toml` |
| macOS | `$HOME/Library/Application Support/ferrite/config.toml` |
| Linux / other Unix | `$XDG_CONFIG_HOME/ferrite/config.toml`, or `$HOME/.config/ferrite/config.toml` if `XDG_CONFIG_HOME` is unset |

Ferrite only ever *reads* this file — it never creates, writes, or
overwrites it, so an existing file is never at risk of being clobbered.
A missing file is not an error (defaults are used); a file that exists
but is malformed, has an unknown field, or has an out-of-range value *is*
an error, and Ferrite reports it and exits rather than silently ignoring
the mistake.

See [`config.example.toml`](config.example.toml) for a documented example
with every field at its default value. Copy it to the path above and
edit it to override any setting:

| Field | Default | Meaning |
| --- | --- | --- |
| `refresh_interval_ms` | `1000` | Dashboard refresh interval, in milliseconds (100-60000). |
| `history_capacity` | `120` | Samples retained per metric history (1-3600). |
| `cpu_alert_threshold_percent` | `90.0` | CPU usage % at or above which the CPU alert activates (0-100). |
| `memory_alert_threshold_percent` | `90.0` | Memory usage % at or above which the memory alert activates (0-100). |
| `disk_alert_threshold_percent` | `90.0` | Usage % at or above which a disk's alert activates, applied independently per disk (0-100). |
| `default_sort_mode` | `"cpu"` | Process table's sort column at startup: `"cpu"` or `"memory"`. |

## Formatting, linting, and testing

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
```

## Architecture

The project is split into a library crate and a thin executable, with
collection, transformation, history, alerting, configuration, application
state, and rendering kept in separate modules:

```
src/
├── main.rs            # Executable entry point: loads config, runs the
│                       # dashboard event loop.
├── lib.rs             # Public library API surface.
├── history.rs         # History: generic bounded, timestamped samples.
├── alerts.rs          # Alert/AlertEngine: threshold activation and
│                       # recovery, independent of rendering.
├── config/            # Configuration parsing and validation.
│   ├── mod.rs         # AppConfig, AppConfigBuilder, ConfigError.
│   └── file.rs        # Optional TOML file loading and path resolution.
├── collector/         # System metrics collection.
│   ├── mod.rs         # Public module surface.
│   ├── cpu.rs         # CpuStats: usage, core count, utilization_fraction.
│   ├── memory.rs      # MemoryStats: byte counts and unit conversions.
│   ├── disk.rs        # DiskInfo: per-disk capacity and usage.
│   ├── network.rs     # NetworkInterfaceStats + the pure
│   │                   # bytes_per_second rate calculation and the
│   │                   # NetworkRateTracker that applies it per
│   │                   # interface, across refreshes.
│   ├── process.rs     # ProcessInfo and deterministic CPU/memory sorts.
│   ├── snapshot.rs    # MetricsSnapshot bundling everything above.
│   └── system.rs      # Collector: the only module that calls into
│                       # sysinfo / the operating system.
└── ui/                # The interactive terminal dashboard.
    ├── mod.rs         # Public module surface.
    ├── app.rs         # App: metrics, history, alerts, selection, sort
    │                   # mode, pause/help state.
    ├── events.rs      # Key-to-Action mapping and action application.
    ├── dashboard.rs   # Layout math, summary formatting, and widget
    │                   # rendering.
    └── terminal.rs    # Terminal init/restore lifecycle (ratatui).
tests/
├── config_tests.rs    # Integration tests for the public config API.
└── collector_tests.rs # Integration tests for the public collector API.
```

Every type in `src/history.rs`, `src/alerts.rs`, `src/config/` (parsing
and validation, not file I/O), every type in `src/collector/` other than
`collector::system`, and every part of `src/ui/` other than `ui::terminal`
are plain data types or pure functions with no dependency on a real
terminal or operating-system API, so they are unit tested in isolation:
application state transitions (`ui::app`), key mapping (`ui::events`),
layout/summary-formatting math (`ui::dashboard`), network rate math
(`collector::network`), history eviction (`history`), and alert
activation/recovery (`alerts`) all have tests that don't touch a real
terminal, filesystem, or collector.

`collector::system`, `config::file` (the parts that touch the filesystem
and environment variables), and `ui::terminal` are the only modules that
talk to the operating system. `collector::system` and `config::file` are
covered by integration tests asserting invariants (e.g. "used memory
never exceeds total memory", "the shipped example config file parses")
rather than exact values, since those depend on the host machine.
`ui::terminal` has no automated test, since doing so would require
driving a real terminal session; it is exercised by running the
dashboard directly.

## Contributing

Contributions are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for setup
instructions, coding conventions, and how to submit a pull request.

## Platform support

Ferrite is built on [`sysinfo`](https://docs.rs/sysinfo) for metrics and
[`ratatui`](https://docs.rs/ratatui) + [`crossterm`](https://docs.rs/crossterm)
for the terminal UI; all three support Linux, macOS, and Windows. This
project has been built, linted, and tested on Windows only; Linux and
macOS are expected to work (continuous integration runs on Ubuntu, and the
Linux/macOS branches of the configuration file's path resolution have
their own unit tests) but have not been verified on real hardware.

The dashboard has been launched on Windows against a real console session
and confirmed to render live CPU, memory, disk, network, history, and
alert-badge data correctly — including the disk and network panels
introduced in this milestone, and the first-sample "no rate yet" network
display. Keyboard interaction (navigation, sorting, pause, quit) could
not be scripted end-to-end in the automated environment this project was
built in, since its terminal reads console input through the OS console
API, which doesn't accept piped input — a human should confirm those
interactively.

Disk and network interface visibility depends entirely on what
`sysinfo`'s platform backend reports; Ferrite does not filter or
reinterpret that list beyond what's described above.

## License

Ferrite is licensed under the [MIT License](LICENSE).
