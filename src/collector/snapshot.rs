//! The combined metrics snapshot returned by the collector.

use super::cpu::CpuStats;
use super::disk::DiskInfo;
use super::memory::MemoryStats;
use super::network::NetworkInterfaceStats;
use super::process::ProcessInfo;

/// A single point-in-time snapshot of system metrics.
///
/// Produced by [`crate::collector::Collector::refresh`]. See
/// [`MetricsSnapshot::cpu_metrics_initialized`] before trusting any CPU
/// usage figures, including per-process ones.
#[derive(Debug, Clone, PartialEq)]
pub struct MetricsSnapshot {
    cpu: CpuStats,
    memory: MemoryStats,
    disks: Vec<DiskInfo>,
    networks: Vec<NetworkInterfaceStats>,
    processes: Vec<ProcessInfo>,
    cpu_metrics_initialized: bool,
}

impl MetricsSnapshot {
    /// Creates a new [`MetricsSnapshot`] from already-collected data.
    ///
    /// `cpu_metrics_initialized` must be `false` until the collector has
    /// taken at least two timed samples; see
    /// [`crate::collector::Collector`] for details.
    pub fn new(
        cpu: CpuStats,
        memory: MemoryStats,
        disks: Vec<DiskInfo>,
        networks: Vec<NetworkInterfaceStats>,
        processes: Vec<ProcessInfo>,
        cpu_metrics_initialized: bool,
    ) -> Self {
        Self {
            cpu,
            memory,
            disks,
            networks,
            processes,
            cpu_metrics_initialized,
        }
    }

    /// System-wide CPU usage.
    pub fn cpu(&self) -> &CpuStats {
        &self.cpu
    }

    /// System memory usage.
    pub fn memory(&self) -> &MemoryStats {
        &self.memory
    }

    /// Storage usage for every detected disk/mount. May be empty if the
    /// platform reports none.
    pub fn disks(&self) -> &[DiskInfo] {
        &self.disks
    }

    /// Traffic statistics for every detected network interface. May be
    /// empty if the platform reports none. Rates are `None` for any
    /// interface seen for the first time this run; see
    /// [`NetworkInterfaceStats`].
    pub fn networks(&self) -> &[NetworkInterfaceStats] {
        &self.networks
    }

    /// The process list, sorted by CPU usage (highest first).
    pub fn processes(&self) -> &[ProcessInfo] {
        &self.processes
    }

    /// Whether CPU usage figures in this snapshot (both [`Self::cpu`] and
    /// each process's `cpu_usage_percent`) reflect a real timed sample.
    ///
    /// `sysinfo` computes CPU usage as a delta between two samples. The
    /// first sample after construction has no prior sample to diff against,
    /// so its CPU usage reads as `0.0` and does not reflect real load.
    /// Callers must not treat CPU usage as a reliable measurement while
    /// this returns `false` — refresh again (after
    /// [`sysinfo::MINIMUM_CPU_UPDATE_INTERVAL`] has elapsed) and use the
    /// next snapshot instead. Memory and process-list membership are not
    /// affected and are reliable from the first snapshot.
    pub fn cpu_metrics_initialized(&self) -> bool {
        self.cpu_metrics_initialized
    }

    /// Consumes the snapshot, returning its fields by value.
    ///
    /// For a caller (such as [`crate::ui::App::apply_snapshot`]) that is
    /// about to store every field and drop the snapshot anyway, this
    /// avoids cloning `disks`, `networks`, and `processes` (and the
    /// `String`s inside them) just to copy out of a borrow first.
    pub fn into_parts(
        self,
    ) -> (
        CpuStats,
        MemoryStats,
        Vec<DiskInfo>,
        Vec<NetworkInterfaceStats>,
        Vec<ProcessInfo>,
        bool,
    ) {
        (
            self.cpu,
            self.memory,
            self.disks,
            self.networks,
            self.processes,
            self.cpu_metrics_initialized,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_the_values_it_was_constructed_with() {
        let cpu = CpuStats::new(12.5, 4);
        let memory = MemoryStats::new(1_000, 500);
        let disks = vec![DiskInfo::new("disk0", "/", 1_000, 500)];
        let networks = vec![NetworkInterfaceStats::new(
            "eth0",
            100,
            200,
            Some(10),
            Some(20),
        )];
        let processes = vec![ProcessInfo::new(1, "init", 0.0, 100)];

        let snapshot = MetricsSnapshot::new(
            cpu,
            memory,
            disks.clone(),
            networks.clone(),
            processes.clone(),
            true,
        );

        assert_eq!(snapshot.cpu(), &cpu);
        assert_eq!(snapshot.memory(), &memory);
        assert_eq!(snapshot.disks(), disks.as_slice());
        assert_eq!(snapshot.networks(), networks.as_slice());
        assert_eq!(snapshot.processes(), processes.as_slice());
        assert!(snapshot.cpu_metrics_initialized());
    }

    #[test]
    fn empty_process_and_disk_lists_are_a_valid_snapshot() {
        let snapshot = MetricsSnapshot::new(
            CpuStats::new(0.0, 1),
            MemoryStats::new(0, 0),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            false,
        );

        assert!(snapshot.disks().is_empty());
        assert!(snapshot.networks().is_empty());
        assert!(snapshot.processes().is_empty());
        assert!(!snapshot.cpu_metrics_initialized());
    }

    #[test]
    fn into_parts_returns_the_same_values_as_the_borrowing_getters() {
        let cpu = CpuStats::new(12.5, 4);
        let memory = MemoryStats::new(1_000, 500);
        let disks = vec![DiskInfo::new("disk0", "/", 1_000, 500)];
        let networks = vec![NetworkInterfaceStats::new(
            "eth0",
            100,
            200,
            Some(10),
            Some(20),
        )];
        let processes = vec![ProcessInfo::new(1, "init", 0.0, 100)];

        let snapshot = MetricsSnapshot::new(
            cpu,
            memory,
            disks.clone(),
            networks.clone(),
            processes.clone(),
            true,
        );

        let (out_cpu, out_memory, out_disks, out_networks, out_processes, out_initialized) =
            snapshot.into_parts();

        assert_eq!(out_cpu, cpu);
        assert_eq!(out_memory, memory);
        assert_eq!(out_disks, disks);
        assert_eq!(out_networks, networks);
        assert_eq!(out_processes, processes);
        assert!(out_initialized);
    }
}
