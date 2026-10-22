//! Dashboard application state.
//!
//! `App` holds everything the dashboard needs to render a frame: the most
//! recent metrics, bounded CPU/memory history, active alerts, the current
//! sort mode, the selected process, and pause/help flags. It never talks
//! to the operating system or a terminal — it only transforms state, so
//! every transition here can be unit tested without a real
//! [`crate::collector::Collector`] or terminal.

use std::time::Instant;

use crate::alerts::{ActiveAlert, AlertEngine};
use crate::collector::{
    CpuStats, DiskInfo, MemoryStats, MetricsSnapshot, NetworkInterfaceStats, ProcessInfo,
    sort_processes_by_cpu_desc, sort_processes_by_memory_desc,
};
use crate::history::History;

/// The alert threshold used by [`App::new`]'s simple constructor for every
/// metric, matching [`crate::config`]'s own defaults.
const DEFAULT_ALERT_THRESHOLD_PERCENT: f32 = 90.0;

/// How the process table is currently sorted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    /// Highest CPU usage first.
    Cpu,
    /// Highest memory usage first.
    Memory,
}

/// The settings [`App::with_options`] needs, gathered in one place so
/// constructing a fully-configured `App` does not require a long
/// positional argument list.
///
/// `App` does not depend on [`crate::config::AppConfig`] directly, so the
/// caller (`main`) is expected to build this from one.
#[derive(Debug, Clone, Copy)]
pub struct AppOptions {
    /// The configured refresh interval, in milliseconds (for display in
    /// the footer).
    pub refresh_interval_ms: u32,
    /// The maximum number of samples retained in the CPU and memory
    /// history.
    pub history_capacity: usize,
    /// The CPU usage percentage at or above which the CPU alert activates.
    pub cpu_alert_threshold_percent: f32,
    /// The memory usage percentage at or above which the memory alert
    /// activates.
    pub memory_alert_threshold_percent: f32,
    /// The disk usage percentage at or above which a disk's alert
    /// activates.
    pub disk_alert_threshold_percent: f32,
    /// The process table's sort mode at startup.
    pub default_sort_mode: SortMode,
}

/// Dashboard application state.
///
/// Construct with [`App::new`] (sensible defaults besides the refresh
/// interval) or [`App::with_options`] (every setting explicit), feed it
/// metrics with [`App::apply_snapshot`] on every refresh, and mutate it via
/// the navigation/sort/pause methods in response to user input (see
/// [`crate::ui::events`]).
#[derive(Debug, Clone)]
pub struct App {
    cpu: CpuStats,
    memory: MemoryStats,
    disks: Vec<DiskInfo>,
    networks: Vec<NetworkInterfaceStats>,
    cpu_metrics_initialized: bool,
    processes: Vec<ProcessInfo>,
    selected_pid: Option<u32>,
    sort_mode: SortMode,
    paused: bool,
    show_help: bool,
    refresh_interval_ms: u32,
    cpu_history: History,
    memory_history: History,
    alerts: AlertEngine,
}

impl App {
    /// Creates a fresh `App` with no metrics yet, using `refresh_interval_ms`
    /// (from [`crate::config::AppConfig`]) for display in the footer, and
    /// Ferrite's documented defaults for everything [`AppOptions`] would
    /// otherwise let a caller configure.
    pub fn new(refresh_interval_ms: u32) -> Self {
        Self::with_options(AppOptions {
            refresh_interval_ms,
            history_capacity: crate::history::DEFAULT_HISTORY_CAPACITY,
            cpu_alert_threshold_percent: DEFAULT_ALERT_THRESHOLD_PERCENT,
            memory_alert_threshold_percent: DEFAULT_ALERT_THRESHOLD_PERCENT,
            disk_alert_threshold_percent: DEFAULT_ALERT_THRESHOLD_PERCENT,
            default_sort_mode: SortMode::Cpu,
        })
    }

    /// Creates a fresh `App` with no metrics yet, from a fully-specified
    /// [`AppOptions`].
    ///
    /// # Panics
    ///
    /// Panics if any alert threshold in `options` is not a finite
    /// percentage in `0.0..=100.0`. Build `options` from a validated
    /// [`crate::config::AppConfig`] (whose own fields are validated in the
    /// same range) to guarantee this never happens.
    pub fn with_options(options: AppOptions) -> Self {
        let alerts = AlertEngine::new(
            options.cpu_alert_threshold_percent,
            options.memory_alert_threshold_percent,
            options.disk_alert_threshold_percent,
        )
        .expect("AppOptions thresholds must already be validated, e.g. by AppConfig");

        Self {
            cpu: CpuStats::new(0.0, 0),
            memory: MemoryStats::new(0, 0),
            disks: Vec::new(),
            networks: Vec::new(),
            cpu_metrics_initialized: false,
            processes: Vec::new(),
            selected_pid: None,
            sort_mode: options.default_sort_mode,
            paused: false,
            show_help: false,
            refresh_interval_ms: options.refresh_interval_ms,
            cpu_history: History::new(options.history_capacity),
            memory_history: History::new(options.history_capacity),
            alerts,
        }
    }

    /// Replaces the current metrics with a freshly collected snapshot,
    /// re-sorting the process list according to the active [`SortMode`],
    /// preserving the selection where possible, recording a CPU/memory
    /// history sample, and re-evaluating alerts.
    ///
    /// If the previously selected process is still present (even at a
    /// different index, due to re-sorting), it stays selected. Otherwise the
    /// selection falls back to the same row index as before, clamped to the
    /// new (possibly shorter) list; if the list is empty, nothing is
    /// selected.
    pub fn apply_snapshot(&mut self, snapshot: MetricsSnapshot) {
        let previous_index = self.selected_index();

        self.alerts.evaluate(&snapshot);

        self.cpu = *snapshot.cpu();
        self.memory = *snapshot.memory();
        self.disks = snapshot.disks().to_vec();
        self.networks = snapshot.networks().to_vec();
        self.cpu_metrics_initialized = snapshot.cpu_metrics_initialized();
        self.processes = snapshot.processes().to_vec();
        self.sort_processes();
        self.reconcile_selection(previous_index);

        let now = Instant::now();
        if self.cpu_metrics_initialized {
            self.cpu_history.push(now, self.cpu.utilization_fraction());
        }
        self.memory_history.push(now, self.memory.used_fraction());
    }

    fn sort_processes(&mut self) {
        match self.sort_mode {
            SortMode::Cpu => sort_processes_by_cpu_desc(&mut self.processes),
            SortMode::Memory => sort_processes_by_memory_desc(&mut self.processes),
        }
    }

    fn reconcile_selection(&mut self, previous_index: Option<usize>) {
        if self.processes.is_empty() {
            self.selected_pid = None;
            return;
        }

        let still_present = self
            .selected_pid
            .is_some_and(|pid| self.processes.iter().any(|p| p.pid == pid));
        if still_present {
            return;
        }

        let fallback_index = previous_index.unwrap_or(0).min(self.processes.len() - 1);
        self.selected_pid = Some(self.processes[fallback_index].pid);
    }

    /// The most recent CPU usage.
    pub fn cpu(&self) -> &CpuStats {
        &self.cpu
    }

    /// The most recent memory usage.
    pub fn memory(&self) -> &MemoryStats {
        &self.memory
    }

    /// Storage usage for every detected disk/mount. May be empty if the
    /// platform reports none.
    pub fn disks(&self) -> &[DiskInfo] {
        &self.disks
    }

    /// Traffic statistics for every detected network interface. May be
    /// empty if the platform reports none.
    pub fn networks(&self) -> &[NetworkInterfaceStats] {
        &self.networks
    }

    /// Bounded history of CPU utilization (0.0..=1.0 fraction samples).
    /// Only receives a new sample on refreshes where
    /// [`Self::cpu_metrics_initialized`] is true.
    pub fn cpu_history(&self) -> &History {
        &self.cpu_history
    }

    /// Bounded history of memory utilization (0.0..=1.0 fraction samples).
    pub fn memory_history(&self) -> &History {
        &self.memory_history
    }

    /// Every alert currently active (CPU, memory, and per-disk).
    pub fn active_alerts(&self) -> Vec<ActiveAlert> {
        self.alerts.active_alerts()
    }

    /// Whether any alert is currently active.
    pub fn has_active_alerts(&self) -> bool {
        self.alerts.has_active_alerts()
    }

    /// Whether [`Self::cpu`] (and each process's CPU usage) reflects a real
    /// timed sample yet. See
    /// [`crate::collector::MetricsSnapshot::cpu_metrics_initialized`].
    pub fn cpu_metrics_initialized(&self) -> bool {
        self.cpu_metrics_initialized
    }

    /// The process list, sorted according to the current [`SortMode`].
    pub fn processes(&self) -> &[ProcessInfo] {
        &self.processes
    }

    /// The current sort mode.
    pub fn sort_mode(&self) -> SortMode {
        self.sort_mode
    }

    /// Sets the sort mode and immediately re-sorts the process list.
    ///
    /// The selection (tracked by PID) is unaffected; its row index simply
    /// follows the process to its new position.
    pub fn set_sort_mode(&mut self, mode: SortMode) {
        if self.sort_mode != mode {
            self.sort_mode = mode;
            self.sort_processes();
        }
    }

    /// The index of the selected process within [`Self::processes`], or
    /// `None` if there is no selection (always the case for an empty list).
    pub fn selected_index(&self) -> Option<usize> {
        self.selected_pid
            .and_then(|pid| self.processes.iter().position(|p| p.pid == pid))
    }

    /// The selected process, or `None` if there is no selection.
    pub fn selected_process(&self) -> Option<&ProcessInfo> {
        self.selected_index().map(|index| &self.processes[index])
    }

    /// Moves the selection one row down, clamped to the last row. A no-op
    /// on an empty list.
    pub fn select_next(&mut self) {
        if self.processes.is_empty() {
            return;
        }
        let next_index = match self.selected_index() {
            Some(index) => (index + 1).min(self.processes.len() - 1),
            None => 0,
        };
        self.selected_pid = Some(self.processes[next_index].pid);
    }

    /// Moves the selection one row up, clamped to the first row. A no-op on
    /// an empty list.
    pub fn select_previous(&mut self) {
        if self.processes.is_empty() {
            return;
        }
        let previous_index = match self.selected_index() {
            Some(index) => index.saturating_sub(1),
            None => 0,
        };
        self.selected_pid = Some(self.processes[previous_index].pid);
    }

    /// Whether metric refreshing is currently paused.
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Toggles pause state and returns the new value.
    pub fn toggle_pause(&mut self) -> bool {
        self.paused = !self.paused;
        self.paused
    }

    /// Whether the help overlay is currently shown.
    pub fn show_help(&self) -> bool {
        self.show_help
    }

    /// Toggles the help overlay and returns the new value.
    pub fn toggle_help(&mut self) -> bool {
        self.show_help = !self.show_help;
        self.show_help
    }

    /// The configured refresh interval, in milliseconds, for display in the
    /// footer.
    pub fn refresh_interval_ms(&self) -> u32 {
        self.refresh_interval_ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn process(pid: u32, cpu: f32, memory: u64) -> ProcessInfo {
        ProcessInfo::new(pid, format!("proc-{pid}"), cpu, memory)
    }

    fn snapshot(processes: Vec<ProcessInfo>) -> MetricsSnapshot {
        MetricsSnapshot::new(
            CpuStats::new(10.0, 4),
            MemoryStats::new(1_000, 400),
            Vec::new(),
            Vec::new(),
            processes,
            true,
        )
    }

    #[test]
    fn new_app_has_no_metrics_or_selection() {
        let app = App::new(1_000);
        assert!(app.processes().is_empty());
        assert!(app.disks().is_empty());
        assert_eq!(app.selected_index(), None);
        assert!(!app.cpu_metrics_initialized());
        assert_eq!(app.sort_mode(), SortMode::Cpu);
        assert!(!app.is_paused());
        assert!(!app.show_help());
        assert_eq!(app.refresh_interval_ms(), 1_000);
    }

    #[test]
    fn applying_a_snapshot_stores_its_disks() {
        let mut app = App::new(1_000);
        let disks = vec![DiskInfo::new("disk0", "/", 1_000, 400)];
        let snapshot = MetricsSnapshot::new(
            CpuStats::new(10.0, 4),
            MemoryStats::new(1_000, 400),
            disks.clone(),
            Vec::new(),
            Vec::new(),
            true,
        );

        app.apply_snapshot(snapshot);

        assert_eq!(app.disks(), disks.as_slice());
    }

    #[test]
    fn applying_a_snapshot_stores_its_networks() {
        let mut app = App::new(1_000);
        let networks = vec![NetworkInterfaceStats::new(
            "eth0",
            1_000,
            500,
            Some(10),
            Some(5),
        )];
        let snapshot = MetricsSnapshot::new(
            CpuStats::new(10.0, 4),
            MemoryStats::new(1_000, 400),
            Vec::new(),
            networks.clone(),
            Vec::new(),
            true,
        );

        app.apply_snapshot(snapshot);

        assert_eq!(app.networks(), networks.as_slice());
    }

    #[test]
    fn applying_a_snapshot_selects_the_first_process() {
        let mut app = App::new(1_000);
        app.apply_snapshot(snapshot(vec![process(1, 5.0, 10), process(2, 50.0, 20)]));

        assert_eq!(app.selected_index(), Some(0));
        assert_eq!(app.selected_process().unwrap().pid, 2);
    }

    #[test]
    fn empty_snapshot_clears_selection() {
        let mut app = App::new(1_000);
        app.apply_snapshot(snapshot(vec![process(1, 5.0, 10)]));
        app.apply_snapshot(snapshot(Vec::new()));

        assert_eq!(app.selected_index(), None);
        assert!(app.processes().is_empty());
    }

    #[test]
    fn select_next_and_previous_clamp_at_the_ends() {
        let mut app = App::new(1_000);
        app.apply_snapshot(snapshot(vec![
            process(1, 30.0, 10),
            process(2, 20.0, 10),
            process(3, 10.0, 10),
        ]));
        assert_eq!(app.selected_index(), Some(0));

        app.select_previous();
        assert_eq!(
            app.selected_index(),
            Some(0),
            "must not go above the first row"
        );

        app.select_next();
        app.select_next();
        assert_eq!(app.selected_index(), Some(2));

        app.select_next();
        assert_eq!(
            app.selected_index(),
            Some(2),
            "must not go below the last row"
        );
    }

    #[test]
    fn navigation_is_a_no_op_on_an_empty_list() {
        let mut app = App::new(1_000);
        app.select_next();
        app.select_previous();
        assert_eq!(app.selected_index(), None);
    }

    #[test]
    fn selection_follows_the_same_process_across_a_refresh() {
        let mut app = App::new(1_000);
        app.apply_snapshot(snapshot(vec![process(1, 10.0, 10), process(2, 50.0, 10)]));
        app.select_next();
        assert_eq!(app.selected_process().unwrap().pid, 1);

        // Process 2's CPU usage drops, so the CPU-sorted order flips, but
        // the selection should still track pid 1 by identity, not by row.
        app.apply_snapshot(snapshot(vec![process(1, 10.0, 10), process(2, 1.0, 10)]));
        assert_eq!(app.selected_process().unwrap().pid, 1);
    }

    #[test]
    fn selection_falls_back_to_clamped_index_when_process_disappears() {
        let mut app = App::new(1_000);
        app.apply_snapshot(snapshot(vec![
            process(1, 30.0, 10),
            process(2, 20.0, 10),
            process(3, 10.0, 10),
        ]));
        app.select_next();
        app.select_next();
        assert_eq!(app.selected_index(), Some(2));
        assert_eq!(app.selected_process().unwrap().pid, 3);

        // pid 3 exits; only two processes remain, so the index-2 selection
        // must clamp rather than go out of bounds.
        app.apply_snapshot(snapshot(vec![process(1, 30.0, 10), process(2, 20.0, 10)]));
        assert_eq!(app.selected_index(), Some(1));
        assert_eq!(app.selected_process().unwrap().pid, 2);
    }

    #[test]
    fn set_sort_mode_resorts_by_memory() {
        let mut app = App::new(1_000);
        app.apply_snapshot(snapshot(vec![process(1, 90.0, 10), process(2, 10.0, 90)]));
        assert_eq!(app.processes()[0].pid, 1, "starts CPU-sorted");

        app.set_sort_mode(SortMode::Memory);
        assert_eq!(app.sort_mode(), SortMode::Memory);
        assert_eq!(app.processes()[0].pid, 2, "now memory-sorted");
    }

    #[test]
    fn switching_sort_mode_keeps_the_same_selected_process() {
        let mut app = App::new(1_000);
        app.apply_snapshot(snapshot(vec![process(1, 90.0, 10), process(2, 10.0, 90)]));
        // pid 1 is selected (highest CPU, row 0).
        assert_eq!(app.selected_process().unwrap().pid, 1);

        app.set_sort_mode(SortMode::Memory);
        // pid 1 now has the lowest memory, so it moves to the last row, but
        // selection should still point at pid 1.
        assert_eq!(app.selected_process().unwrap().pid, 1);
        assert_eq!(app.selected_index(), Some(1));
    }

    #[test]
    fn toggle_pause_flips_and_returns_new_state() {
        let mut app = App::new(1_000);
        assert!(!app.is_paused());
        assert!(app.toggle_pause());
        assert!(app.is_paused());
        assert!(!app.toggle_pause());
    }

    #[test]
    fn toggle_help_flips_and_returns_new_state() {
        let mut app = App::new(1_000);
        assert!(!app.show_help());
        assert!(app.toggle_help());
        assert!(app.show_help());
        assert!(!app.toggle_help());
    }

    #[test]
    fn new_app_has_empty_history_and_no_alerts() {
        let app = App::new(1_000);
        assert!(app.cpu_history().is_empty());
        assert!(app.memory_history().is_empty());
        assert!(!app.has_active_alerts());
        assert!(app.active_alerts().is_empty());
    }

    fn snapshot_with_cpu_and_memory(
        cpu_percent: f32,
        cpu_initialized: bool,
        memory_used: u64,
    ) -> MetricsSnapshot {
        MetricsSnapshot::new(
            CpuStats::new(cpu_percent, 1),
            MemoryStats::new(100, memory_used),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            cpu_initialized,
        )
    }

    #[test]
    fn applying_a_snapshot_records_memory_history_even_before_cpu_is_initialized() {
        let mut app = App::new(1_000);
        app.apply_snapshot(snapshot_with_cpu_and_memory(0.0, false, 40));

        assert!(
            app.cpu_history().is_empty(),
            "CPU history must wait for a reliable sample"
        );
        assert_eq!(app.memory_history().len(), 1);
        assert_eq!(app.memory_history().latest().unwrap().value, 0.4);
    }

    #[test]
    fn applying_a_snapshot_records_cpu_history_once_initialized() {
        let mut app = App::new(1_000);
        app.apply_snapshot(snapshot_with_cpu_and_memory(50.0, true, 0));

        assert_eq!(app.cpu_history().len(), 1);
        assert_eq!(app.cpu_history().latest().unwrap().value, 0.5);
    }

    #[test]
    fn history_capacity_comes_from_app_options() {
        let mut app = App::with_options(AppOptions {
            refresh_interval_ms: 1_000,
            history_capacity: 2,
            cpu_alert_threshold_percent: 90.0,
            memory_alert_threshold_percent: 90.0,
            disk_alert_threshold_percent: 90.0,
            default_sort_mode: SortMode::Cpu,
        });

        for _ in 0..5 {
            app.apply_snapshot(snapshot_with_cpu_and_memory(10.0, true, 10));
        }

        assert_eq!(app.cpu_history().len(), 2);
        assert_eq!(app.memory_history().len(), 2);
    }

    #[test]
    fn applying_snapshots_activates_and_recovers_alerts() {
        let mut app = App::with_options(AppOptions {
            refresh_interval_ms: 1_000,
            history_capacity: 10,
            cpu_alert_threshold_percent: 50.0,
            memory_alert_threshold_percent: 90.0,
            disk_alert_threshold_percent: 90.0,
            default_sort_mode: SortMode::Cpu,
        });

        app.apply_snapshot(snapshot_with_cpu_and_memory(80.0, true, 0));
        assert!(app.has_active_alerts());
        assert_eq!(app.active_alerts()[0].label, "CPU");

        app.apply_snapshot(snapshot_with_cpu_and_memory(10.0, true, 0));
        assert!(!app.has_active_alerts());
    }

    #[test]
    fn with_options_rejects_invalid_thresholds_via_panic() {
        let result = std::panic::catch_unwind(|| {
            App::with_options(AppOptions {
                refresh_interval_ms: 1_000,
                history_capacity: 10,
                cpu_alert_threshold_percent: 200.0,
                memory_alert_threshold_percent: 90.0,
                disk_alert_threshold_percent: 90.0,
                default_sort_mode: SortMode::Cpu,
            })
        });
        assert!(
            result.is_err(),
            "an out-of-range threshold must not be silently accepted"
        );
    }
}
