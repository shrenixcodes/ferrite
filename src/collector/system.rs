//! The operating-system-backed metrics collector.

use std::time::Instant;

use sysinfo::{Disks, Networks, ProcessesToUpdate, System};

use super::cpu::CpuStats;
use super::disk::DiskInfo;
use super::memory::MemoryStats;
use super::network::{NetworkInterfaceStats, NetworkRateTracker};
use super::process::{ProcessInfo, sort_processes_by_cpu_desc};
use super::snapshot::MetricsSnapshot;

/// Collects system metrics using the host operating system's reporting
/// facilities.
///
/// `sysinfo` computes CPU usage as a delta between two samples, so a
/// freshly constructed `Collector` has no prior sample to diff against.
/// [`Collector::refresh`] reports, via
/// [`MetricsSnapshot::cpu_metrics_initialized`], whether enough wall-clock
/// time has elapsed since the previous sample
/// ([`sysinfo::MINIMUM_CPU_UPDATE_INTERVAL`]) for the CPU figures in that
/// snapshot to be meaningful. Memory and the process list are reliable
/// immediately, from the first snapshot.
///
/// Platform note: available metrics depend on `sysinfo`'s platform
/// backend. CPU usage, memory totals, per-process (PID, name, CPU,
/// memory), disk capacity, and network interface counters are all read
/// through `sysinfo`'s cross-platform APIs, which it supports on every
/// platform it builds for (Linux, macOS, Windows). Which specific disks
/// or network interfaces are visible — and how fine-grained their
/// reported counters are — is still ultimately up to the OS and `sysinfo`'s
/// platform backend, not something this crate controls.
pub struct Collector {
    system: System,
    disks: Disks,
    networks: Networks,
    network_rates: NetworkRateTracker,
    last_refresh: Option<Instant>,
}

impl Collector {
    /// Creates a new collector and takes an initial, unreliable CPU
    /// baseline sample.
    ///
    /// The snapshot returned by the first call to [`Self::refresh`] can
    /// report reliable CPU usage, provided enough time
    /// ([`sysinfo::MINIMUM_CPU_UPDATE_INTERVAL`]) has elapsed since this
    /// call returned. Network rates follow the same rule, independently:
    /// the first call to [`Self::refresh`] has no prior counter sample for
    /// any interface, so its network rates are `None` (see
    /// [`NetworkInterfaceStats`]).
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_cpu_usage();
        system.refresh_memory();
        system.refresh_processes(ProcessesToUpdate::All, true);

        Self {
            system,
            disks: Disks::new_with_refreshed_list(),
            networks: Networks::new_with_refreshed_list(),
            network_rates: NetworkRateTracker::new(),
            last_refresh: Some(Instant::now()),
        }
    }

    /// Takes a new sample and returns a [`MetricsSnapshot`].
    ///
    /// Processes are sorted by CPU usage, highest first, with ties broken
    /// by ascending PID (see [`sort_processes_by_cpu_desc`]). A process
    /// that exits during collection simply does not appear in the
    /// resulting list; collection never panics because of it. Disks that
    /// are no longer mounted are dropped the same way.
    ///
    /// Disk usage is sampled from the OS's cached filesystem metadata
    /// (statvfs-style calls), not a directory scan, so calling this once
    /// per refresh tick (rather than per rendered frame, which the
    /// dashboard already avoids) is cheap.
    pub fn refresh(&mut self) -> MetricsSnapshot {
        let cpu_metrics_initialized = self
            .last_refresh
            .is_some_and(|instant| instant.elapsed() >= sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);

        self.system.refresh_cpu_usage();
        self.system.refresh_memory();
        self.system.refresh_processes(ProcessesToUpdate::All, true);
        self.disks.refresh(true);
        self.networks.refresh(true);
        let now = Instant::now();
        self.last_refresh = Some(now);

        let cpu = CpuStats::new(self.system.global_cpu_usage(), self.system.cpus().len());
        let memory = MemoryStats::new(self.system.total_memory(), self.system.used_memory());

        let disks: Vec<DiskInfo> = self
            .disks
            .list()
            .iter()
            .map(|disk| {
                DiskInfo::new(
                    disk.name().to_string_lossy().into_owned(),
                    disk.mount_point().to_string_lossy().into_owned(),
                    disk.total_space(),
                    disk.available_space(),
                )
            })
            .collect();

        let current_network_totals: Vec<(String, u64, u64)> = self
            .networks
            .iter()
            .map(|(name, data)| {
                (
                    name.clone(),
                    data.total_received(),
                    data.total_transmitted(),
                )
            })
            .collect();
        let networks: Vec<NetworkInterfaceStats> =
            self.network_rates.update(&current_network_totals, now);

        let mut processes: Vec<ProcessInfo> = self
            .system
            .processes()
            .values()
            .map(|process| {
                ProcessInfo::new(
                    process.pid().as_u32(),
                    process.name().to_string_lossy().into_owned(),
                    process.cpu_usage(),
                    process.memory(),
                )
            })
            .collect();
        sort_processes_by_cpu_desc(&mut processes);

        MetricsSnapshot::new(
            cpu,
            memory,
            disks,
            networks,
            processes,
            cpu_metrics_initialized,
        )
    }
}

impl Default for Collector {
    fn default() -> Self {
        Self::new()
    }
}
