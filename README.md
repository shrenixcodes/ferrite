# Ferrite

A fast, lightweight, terminal-native system monitor built in Rust, featuring
real-time resource metrics, process inspection, and configurable alerts.

> Monitor your system. Understand your performance. Stay in the terminal.

## Development status

**Pre-1.0, active development (current version: see `Cargo.toml`) — core
metrics, history, and alerting are in place; export and remote telemetry
are not.** Running the executable opens a live terminal dashboard with
CPU, memory, disk, and network panels, bounded in-memory history
sparklines, a sortable/navigable process table, and configurable
threshold alerts. Configuration can come from an optional TOML file.
There is still no data export, no background collection decoupled from
the render loop, and no formal GitHub release has been published yet
(see [CHANGELOG.md](CHANGELOG.md) and [RELEASING.md](RELEASING.md)).

No screenshot or terminal recording is included yet — none has actually
been captured for this repository, and a fabricated one would be
misleading. Contributions adding a genuine one are welcome.

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
- **Dev-only dependency:** [`criterion`](https://docs.rs/criterion) `0.8`
  (minimal features), for the benchmark suite under `benches/`. This
  never compiles into `cargo build --release`'s output — only into
  `cargo bench`/`cargo test`.
- No `cargo-audit`/`cargo-deny` style automated vulnerability scan has
  been run against this dependency tree (the tooling wasn't available in
  the environment this was prepared in); dependencies were reviewed
  manually via `cargo tree` instead. See
  [CONTRIBUTING.md](CONTRIBUTING.md) for the project's dependency policy.

## Prerequisites

- A stable Rust toolchain, 1.85 or later (edition 2024 requires it).
  Install via [rustup](https://rustup.rs/).
- A real terminal to run the dashboard in. Platform notes:
  - **Windows:** Windows Terminal or the Windows 10/11 Console both
    work. Legacy `cmd.exe` in older Windows versions may render the
    Unicode symbols (`⚠ ↓ ↑ —`) and sparkline bars poorly — this is a
    terminal-capability limitation, not something Ferrite detects or
    works around.
  - **Linux:** any modern terminal emulator (GNOME Terminal, Konsole,
    Alacritty, etc.) over a UTF-8 locale.
  - **macOS:** Terminal.app or iTerm2.
  - No elevated/administrator privileges are required — Ferrite only
    reads metrics an unprivileged process can already see via `sysinfo`.

## Installing

**From a release binary:** once a release has been published (see
[RELEASING.md](RELEASING.md) — none has yet), download the binary for
your platform from the
[Releases page](https://github.com/shrenixcodes/ferrite/releases) and
run it directly. No installer or elevated privileges are needed.

**From source with Cargo** (works today):

```bash
git clone https://github.com/shrenixcodes/ferrite.git
cd ferrite
cargo install --path .
```

This builds a release binary and copies it to Cargo's bin directory
(`~/.cargo/bin` by default, or `%USERPROFILE%\.cargo\bin` on Windows —
make sure that directory is on your `PATH`, which the Rust installer
normally arranges for you). Run it with `ferrite` from anywhere
afterward.

To uninstall a binary installed this way:

```bash
cargo uninstall ferrite
```

**Without installing**, just build and run in place from a clone:

```bash
cargo build --release
cargo run --release
```

## Running Ferrite

```bash
cargo run
```

(or just `ferrite` if installed via `cargo install`). The dashboard needs
a real terminal — it will fail with an I/O error if stdout isn't one,
such as when piped or redirected to a file. Controls:

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

## Troubleshooting

- **"ferrite: <some I/O error>" and it exits immediately.** Ferrite
  needs a real terminal attached to stdout. This happens if you pipe its
  output (`ferrite | less`), redirect it to a file, or run it in a
  context with no attached console (some CI runners, certain IDE "run"
  panels). Run it directly in an interactive terminal instead.
- **"ferrite: invalid configuration: ...".** Your configuration file
  (see below) has a malformed value, a typo'd field name, or isn't valid
  TOML. The error message names the specific field and problem. Fix the
  file or delete it to fall back to defaults — Ferrite never fixes or
  overwrites it for you.
- **Permission errors reading metrics.** Ferrite only reads what an
  unprivileged process can already see. On most systems this is
  everything needed for CPU/memory/disk/network totals and your own
  processes; some systems restrict visibility into *other users'*
  process details (name/command line), which `sysinfo` — and therefore
  Ferrite — cannot see around. This is expected OS-level sandboxing, not
  a bug.
- **Garbled symbols (boxes, `?`, or missing sparkline bars).** Your
  terminal or font doesn't support the Unicode characters Ferrite uses
  for the alert badge (`⚠`), network arrows (`↓↑`), and sparklines. Try a
  terminal with better Unicode coverage (see Prerequisites above);
  Ferrite does not currently offer an ASCII-only rendering mode.
- **The terminal looks broken after Ferrite exits (no prompt, garbled
  cursor).** This shouldn't happen on a normal `q`/`Esc`/`Ctrl+C` exit or
  a panic — both restore the terminal (see `src/ui/terminal.rs`). It can
  still happen if the process is killed outright (`kill -9`, Task
  Manager "End Task", a host/VM crash), since no cleanup code runs on a
  non-unwinding kill. Run `reset` (Linux/macOS) or close and reopen the
  terminal tab (Windows) to recover; this is a limitation of any raw-mode
  terminal application, not specific to Ferrite.

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

## Benchmarking

```bash
cargo bench
```

Runs the [Criterion](https://docs.rs/criterion) microbenchmark suite
under `benches/core.rs` (process sorting, network rate calculation,
history insertion/eviction, and metric transformations). See
[BENCHMARKING.md](BENCHMARKING.md) for what's covered, how to read the
output, and the actual last-measured results with full hardware/toolchain
disclosure. These are not run in CI, since results are machine-dependent
and a fixed threshold would be brittle.

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
[CHANGELOG.md](CHANGELOG.md) tracks notable changes, and
[RELEASING.md](RELEASING.md) documents how (and whether) a version gets
tagged and published.

## Security

See [SECURITY.md](SECURITY.md) for how to report a vulnerability. There
is no dedicated security team or private mailing list yet — reports are
handled by whoever is maintaining the project.

## Platform support

Ferrite is built on [`sysinfo`](https://docs.rs/sysinfo) for metrics and
[`ratatui`](https://docs.rs/ratatui) + [`crossterm`](https://docs.rs/crossterm)
for the terminal UI; all three support Linux, macOS, and Windows.

**What's actually been verified, as of this writing:**

- **Local development and manual testing:** Windows only. The dashboard
  has been launched on Windows against a real console session and
  confirmed to render live CPU, memory, disk, network, history, and
  alert-badge data correctly, including the first-sample "no rate yet"
  network display. Keyboard interaction (navigation, sorting, pause,
  quit) could not be scripted end-to-end in the automated environment
  this was built in, since its terminal reads console input through the
  OS console API, which doesn't accept piped input — a human should
  confirm those interactively.
- **Continuous integration:** `.github/workflows/ci.yml` is configured
  to build, lint, and run the full test suite on `ubuntu-latest`,
  `windows-latest`, and `macos-latest` for every push and pull request.
  **This configuration has not yet executed on GitHub** as of the commit
  that introduced it — it will start running on the next push to this
  repository. Until a run has actually completed, treat Linux/macOS
  support as "expected to work, not yet confirmed," not as verified.
- **Release builds:** `.github/workflows/release.yml` is configured to
  cross-compile for `x86_64-unknown-linux-gnu`,
  `x86_64-pc-windows-msvc`, `x86_64-apple-darwin`, and
  `aarch64-apple-darwin` when a version tag is pushed, re-running the
  full check suite per target first. No tag has been pushed yet, so no
  release binary has actually been produced by this workflow.

Compiling for a target is not the same as having run and exercised the
binary on it — this README does not claim the latter for anything beyond
the Windows manual testing described above.

Disk and network interface visibility depends entirely on what
`sysinfo`'s platform backend reports; Ferrite does not filter or
reinterpret that list beyond what's described above.

## License

Ferrite is licensed under the [MIT License](LICENSE).
