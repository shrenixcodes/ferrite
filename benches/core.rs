//! Microbenchmarks for Ferrite's pure, performance-sensitive functions.
//!
//! These measure isolated functions in this process, not the whole
//! application (CPU/memory overhead of the running dashboard is not
//! measured here — see `BENCHMARKING.md` for how to measure that
//! separately with an OS-level process monitor). Inputs are deterministic
//! (no RNG dependency) so results are repeatable across runs on the same
//! machine.
//!
//! Run with:
//!
//! ```sh
//! cargo bench
//! ```
//!
//! See `BENCHMARKING.md` for the actual hardware/toolchain this was last
//! measured on, and how to read the results.

use std::time::{Duration, Instant};

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use ferrite::collector::{
    CpuStats, MemoryStats, ProcessInfo, bytes_per_second, sort_processes_by_cpu_desc,
    sort_processes_by_memory_desc,
};
use ferrite::history::History;

/// Builds `count` processes with deterministic, non-monotonic CPU/memory
/// values, so the sort under test cannot rely on data that is already
/// sorted (or reverse-sorted) by coincidence.
fn sample_processes(count: usize) -> Vec<ProcessInfo> {
    (0..count)
        .map(|i| {
            let cpu = ((i * 37 + 13) % 10_000) as f32 / 100.0;
            let memory = ((i * 97 + 7) % 1_000_000) as u64;
            ProcessInfo::new(i as u32 + 1, format!("process-{i}"), cpu, memory)
        })
        .collect()
}

fn bench_process_sorting(c: &mut Criterion) {
    let mut group = c.benchmark_group("process_sorting");
    for &count in &[100usize, 1_000, 10_000] {
        let input = sample_processes(count);

        group.bench_with_input(
            BenchmarkId::new("sort_by_cpu_desc", count),
            &input,
            |b, input| {
                b.iter_batched(
                    || input.clone(),
                    |mut processes| sort_processes_by_cpu_desc(&mut processes),
                    criterion::BatchSize::SmallInput,
                );
            },
        );

        group.bench_with_input(
            BenchmarkId::new("sort_by_memory_desc", count),
            &input,
            |b, input| {
                b.iter_batched(
                    || input.clone(),
                    |mut processes| sort_processes_by_memory_desc(&mut processes),
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

fn bench_network_rate_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("network_rate_calculation");

    group.bench_function("bytes_per_second_typical", |b| {
        b.iter(|| bytes_per_second(1_000_000, 2_500_000, Duration::from_secs(1)));
    });

    group.bench_function("bytes_per_second_counter_reset", |b| {
        b.iter(|| bytes_per_second(5_000_000, 1_000, Duration::from_secs(1)));
    });

    group.finish();
}

fn bench_history(c: &mut Criterion) {
    let mut group = c.benchmark_group("history");

    group.bench_function("push_below_capacity", |b| {
        b.iter_batched(
            || History::new(10_000),
            |mut history| {
                let t0 = Instant::now();
                for i in 0..1_000u64 {
                    history.push(t0 + Duration::from_millis(i), i as f64 / 1_000.0);
                }
                history
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.bench_function("push_with_eviction_at_capacity", |b| {
        b.iter_batched(
            || {
                let mut history = History::new(120);
                let t0 = Instant::now();
                for i in 0..120u64 {
                    history.push(t0 + Duration::from_millis(i), i as f64 / 120.0);
                }
                (history, t0)
            },
            |(mut history, t0)| {
                for i in 120..1_120u64 {
                    history.push(t0 + Duration::from_millis(i), i as f64 / 1_120.0);
                }
                history
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

fn bench_metric_transformations(c: &mut Criterion) {
    let mut group = c.benchmark_group("metric_transformations");

    let cpu = CpuStats::new(437.5, 8);
    group.bench_function("cpu_utilization_fraction", |b| {
        b.iter(|| cpu.utilization_fraction());
    });

    let memory = MemoryStats::new(16_000_000_000, 11_234_567_890);
    group.bench_function("memory_used_fraction", |b| {
        b.iter(|| memory.used_fraction());
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_process_sorting,
    bench_network_rate_calculation,
    bench_history,
    bench_metric_transformations
);
criterion_main!(benches);
