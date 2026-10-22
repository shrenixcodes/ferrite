# Ferrite

A fast, lightweight, terminal-native system monitor built in Rust, featuring
real-time resource metrics, process inspection, and configurable alerts.

> Monitor your system. Understand your performance. Stay in the terminal.

## Development status

**Early stage — foundation only.** Ferrite is not yet a functioning system
monitor. This milestone establishes the Rust project foundation: a validated
configuration system, a test suite, continuous integration, and project
documentation. It does **not** yet collect CPU, memory, disk, network, or
process data, and it has no terminal dashboard.

Running the executable prints a short startup message and exits — it does
not display live system metrics.

## Project goals

- Provide real-time CPU, memory, disk, and network metrics in the terminal.
- Offer process inspection comparable to tools like `htop` or `btop`.
- Support configurable alerts for resource thresholds.
- Stay lightweight, fast, and dependency-conscious.
- Remain fully terminal-native, with no GUI requirement.

## Features implemented now

- A validated application configuration type (`AppConfig`) with a bounded,
  documented refresh interval.
- A library crate (`ferrite`) exposing the configuration API.
- A minimal executable that reports its own early-development status.
- A unit and integration test suite covering configuration behavior.
- Continuous integration running formatting, linting, and tests on every
  push and pull request.

## Planned features (not yet implemented)

- CPU and memory usage collection.
- Process listing and inspection.
- Disk and network statistics.
- A `ratatui`-based terminal dashboard.
- Background data collectors.
- Configurable alerts, historical data, and data export.

## Technology stack

- **Language:** Rust (2024 edition)
- **Dependencies:** none at this milestone — the standard library only.
- Planned future milestones are expected to introduce a terminal UI crate
  (such as `ratatui`) and a system-information crate (such as `sysinfo`).

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
├── main.rs    # Executable entry point; prints a startup message.
├── lib.rs     # Public library API surface.
└── config.rs  # Validated AppConfig type and its tests.
tests/
└── config_tests.rs  # Integration tests for the public config API.
```

`src/config.rs` has no dependency on terminal rendering or operating-system
APIs, so it can be tested in isolation. Future milestones are expected to add
modules for metrics collection, process inspection, and the terminal
dashboard without requiring changes to the configuration layer.

## Contributing

Contributions are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for setup
instructions, coding conventions, and how to submit a pull request.

## Platform support

Ferrite is developed and tested on a stable Rust toolchain. No platform-
specific system APIs have been introduced yet, since this milestone performs
no system data collection. Platform-specific behavior will be documented as
later milestones add it.

## License

Ferrite is licensed under the [MIT License](LICENSE).
