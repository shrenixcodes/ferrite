//! Per-process metrics and deterministic sorting.

use std::cmp::Ordering;

/// A snapshot of a single process's resource usage.
///
/// `cpu_usage_percent` follows the same reliability rules as
/// [`crate::collector::CpuStats`]: until the collector has taken two timed
/// samples, the value is not yet meaningful. See
/// [`crate::collector::MetricsSnapshot::cpu_metrics_initialized`].
#[derive(Debug, Clone, PartialEq)]
pub struct ProcessInfo {
    /// The operating system process identifier.
    pub pid: u32,
    /// The process name, as reported by the operating system.
    ///
    /// Process names are converted losslessly from the OS-native
    /// representation; non-UTF-8 bytes are replaced rather than causing an
    /// error, since display is the only intended use.
    pub name: String,
    /// CPU usage as a percentage (0.0–100.0 per core, so a multi-core
    /// process may exceed 100.0).
    pub cpu_usage_percent: f32,
    /// Resident memory usage, in bytes.
    pub memory_bytes: u64,
}

impl ProcessInfo {
    /// Creates a new [`ProcessInfo`] from its component fields.
    pub fn new(
        pid: u32,
        name: impl Into<String>,
        cpu_usage_percent: f32,
        memory_bytes: u64,
    ) -> Self {
        Self {
            pid,
            name: name.into(),
            cpu_usage_percent,
            memory_bytes,
        }
    }
}

/// Sorts `processes` by CPU usage, highest first, breaking ties by
/// ascending PID so the ordering is deterministic even when two processes
/// report identical (or NaN) CPU usage.
///
/// This is a pure function over already-collected data, so it can be tested
/// without depending on the host's actual process list.
pub fn sort_processes_by_cpu_desc(processes: &mut [ProcessInfo]) {
    processes.sort_by(|a, b| {
        b.cpu_usage_percent
            .partial_cmp(&a.cpu_usage_percent)
            .unwrap_or(Ordering::Equal)
            .then_with(|| a.pid.cmp(&b.pid))
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn process(pid: u32, cpu: f32, memory: u64) -> ProcessInfo {
        ProcessInfo::new(pid, format!("proc-{pid}"), cpu, memory)
    }

    #[test]
    fn sorts_by_cpu_usage_descending() {
        let mut processes = vec![
            process(1, 10.0, 100),
            process(2, 50.0, 200),
            process(3, 25.0, 300),
        ];

        sort_processes_by_cpu_desc(&mut processes);

        let pids: Vec<u32> = processes.iter().map(|p| p.pid).collect();
        assert_eq!(pids, vec![2, 3, 1]);
    }

    #[test]
    fn breaks_ties_by_ascending_pid() {
        let mut processes = vec![
            process(30, 5.0, 0),
            process(10, 5.0, 0),
            process(20, 5.0, 0),
        ];

        sort_processes_by_cpu_desc(&mut processes);

        let pids: Vec<u32> = processes.iter().map(|p| p.pid).collect();
        assert_eq!(pids, vec![10, 20, 30]);
    }

    #[test]
    fn empty_list_sorts_without_panicking() {
        let mut processes: Vec<ProcessInfo> = Vec::new();
        sort_processes_by_cpu_desc(&mut processes);
        assert!(processes.is_empty());
    }

    #[test]
    fn single_item_list_is_unchanged() {
        let mut processes = vec![process(7, 99.0, 1_024)];
        sort_processes_by_cpu_desc(&mut processes);
        assert_eq!(processes[0].pid, 7);
    }

    #[test]
    fn nan_cpu_usage_does_not_panic_during_sort() {
        let mut processes = vec![
            process(2, f32::NAN, 0),
            process(1, f32::NAN, 0),
            process(3, 10.0, 0),
        ];

        sort_processes_by_cpu_desc(&mut processes);

        let pids: Vec<u32> = processes.iter().map(|p| p.pid).collect();
        assert_eq!(pids.len(), 3);
        assert!(pids.contains(&1) && pids.contains(&2) && pids.contains(&3));
    }

    #[test]
    fn nan_cpu_usage_entries_break_ties_by_ascending_pid() {
        let mut processes = vec![process(2, f32::NAN, 0), process(1, f32::NAN, 0)];

        sort_processes_by_cpu_desc(&mut processes);

        let pids: Vec<u32> = processes.iter().map(|p| p.pid).collect();
        assert_eq!(pids, vec![1, 2]);
    }
}
