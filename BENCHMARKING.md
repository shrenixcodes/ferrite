# Benchmarking Ferrite

This documents the benchmark suite under `benches/core.rs`, how to run it,
and the most recent measured results. These are **microbenchmarks** of
isolated, pure functions — they do not measure the whole running
application (its real-world CPU/memory overhead), which is a separate,
coarser measurement described at the bottom of this document.

## What's measured

| Group | Function(s) | Why it matters |
| --- | --- | --- |
| `process_sorting` | `sort_processes_by_cpu_desc`, `sort_processes_by_memory_desc` | Runs once per refresh tick over the full process list; cost scales with process count. |
| `network_rate_calculation` | `bytes_per_second` | Runs once per network interface per refresh tick. |
| `history` | `History::push` (below capacity, and with eviction at capacity) | Runs twice per refresh tick (CPU and memory history). |
| `metric_transformations` | `CpuStats::utilization_fraction`, `MemoryStats::used_fraction` | Runs on every render frame (gauges) and every refresh (history, alerts). |

All inputs are deterministic (generated from a fixed formula, not an RNG),
so results are repeatable across runs on the same machine. These are
*not* intended to be compared against other system monitors — no such
comparison is made or implied anywhere in this document.

## Running the benchmarks

```bash
cargo bench
```

This uses [Criterion](https://docs.rs/criterion), which warms up, collects
~100 samples per benchmark, and reports a 95%-confidence time range. A
full run takes a few minutes because of this statistical sampling — it is
intentionally not part of ordinary CI, since results vary by machine and a
fixed pass/fail threshold would be brittle.

To run a single group: `cargo bench -- process_sorting`.

## Measured results

Measured on the hardware/toolchain below. These numbers will differ on
other machines; they are a snapshot, not a guarantee.

**Environment:**
- CPU: 12th Gen Intel(R) Core(TM) i5-12450H (8 cores / 12 logical processors)
- OS: Windows 11 Home Single Language 10.0.26200
- Rust: `rustc 1.98.0 (88d9e12ae 2026-08-18)`, release profile
- Command: `cargo bench`

| Benchmark | Input | Time (median) |
| --- | --- | --- |
| `sort_by_cpu_desc` | 100 processes | 4.16 µs |
| `sort_by_cpu_desc` | 1,000 processes | 43.2 µs |
| `sort_by_cpu_desc` | 10,000 processes | 631 µs |
| `sort_by_memory_desc` | 100 processes | 3.97 µs |
| `sort_by_memory_desc` | 1,000 processes | 37.4 µs |
| `sort_by_memory_desc` | 10,000 processes | 371 µs |
| `bytes_per_second` (typical) | — | 339 ps |
| `bytes_per_second` (counter reset) | — | 329 ps |
| `History::push` (below capacity) | 1,000 pushes into capacity 10,000 | 9.58 µs total (~9.6 ns/push) |
| `History::push` (at capacity, evicting) | 1,000 pushes into capacity 120 | 6.84 µs total (~6.8 ns/push) |
| `CpuStats::utilization_fraction` | — | 2.59 ns |
| `MemoryStats::used_fraction` | — | 330 ps |

**Reading these honestly:**
- The sub-nanosecond numbers (`bytes_per_second`, `used_fraction`) are at
  the edge of what Criterion can resolve for a single arithmetic
  expression with constant-ish inputs; the compiler may have partially
  optimized the benchmark loop. Treat them as "negligible," not as a
  precise absolute cost — in real use these run once or twice per
  refresh tick, not in a hot loop.
- Even at a typical host's process count (dozens to a few hundred),
  sorting costs tens of microseconds — far below the default 1-second
  refresh interval, and not a bottleneck.
- `History::push` with eviction measured *faster* per-operation than the
  no-eviction case here, most likely because the smaller, capacity-120
  `VecDeque` stays cache-resident while the capacity-10,000 one does not.
  This is a plausible explanation, not a verified root cause — it was not
  profiled further.

These results have only been measured on the Windows machine described
above. They have not been measured on Linux or macOS; if you run this
suite on another platform, numbers (and relative rankings) may differ.

## Whole-application overhead (not yet measured here)

CPU/memory overhead of the *running dashboard itself* (not the
microbenchmarks above) is a different measurement — it requires watching
the compiled binary with an OS-level process monitor (e.g. Task Manager,
`htop`, `ps`, or `/usr/bin/time -v`) while it runs, rather than Criterion.
This has not been done as part of this milestone; the commands below are
provided so it can be measured, rather than claiming a number that was
never actually recorded:

```bash
# Linux/macOS: run Ferrite, then in another terminal:
ps -o %cpu,rss,etime -p "$(pgrep ferrite)"

# Windows (PowerShell):
Get-Process ferrite | Select-Object CPU, WorkingSet, StartTime
```
