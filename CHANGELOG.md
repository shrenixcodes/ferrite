# Changelog

All notable changes to Ferrite are documented in this file. The format is
based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and
this project uses [Semantic Versioning](https://semver.org/) once it
reaches 1.0 — before that, as usual for `0.x` releases, minor versions
may include breaking changes.

No version of Ferrite has been tagged or published as a GitHub release
yet. Everything below is still `[Unreleased]`; see
[RELEASING.md](RELEASING.md) for what happens when that changes.

## [Unreleased]

### Added

- Rust project foundation: validated `AppConfig`, unit/integration
  tests, CI (formatting, Clippy, tests, release build).
- A system metrics collector: CPU usage, memory, and a process list
  (PID, name, CPU%, memory), with explicit handling of `sysinfo`'s
  two-sample CPU warm-up period.
- An interactive Ratatui terminal dashboard: live CPU/memory gauges, a
  sortable/navigable process table, pause/resume, a help overlay, a
  resize-safe layout, and reliable terminal restoration on exit, error,
  or panic.
- Disk usage metrics (total/available/used, used %) and network
  interface traffic (cumulative counters plus safely computed
  bytes-per-second rates, never misleading on the first sample or after
  a counter reset).
- Bounded, in-memory-only history for CPU/memory utilization, rendered
  as dashboard sparklines.
- Configurable threshold alerts for CPU, memory, and (per-disk) storage
  usage, with activation/recovery semantics and no repeated spam while a
  threshold stays exceeded.
- Optional TOML configuration file support (refresh interval, history
  capacity, alert thresholds, default sort mode), validated on load with
  descriptive errors; see `config.example.toml`.
- A benchmark suite (`cargo bench`, via Criterion) covering process
  sorting, network rate calculation, history insertion/eviction, and
  metric transformations — see `BENCHMARKING.md`.
- Continuous integration across Linux, Windows, and macOS, and a
  tag-triggered release workflow that builds verified binaries for
  `x86_64-unknown-linux-gnu`, `x86_64-pc-windows-msvc`,
  `x86_64-apple-darwin`, and `aarch64-apple-darwin`.
- `SECURITY.md`, issue templates, and a pull request template.

### Fixed

- `Ctrl+C` now quits the dashboard explicitly, since raw mode stops the
  terminal from turning it into a `SIGINT`.
- The header's alert badge now reads "1 ALERT" instead of "1 ALERT(S)".

[Unreleased]: https://github.com/shrenixcodes/ferrite/compare/8018e57...HEAD
