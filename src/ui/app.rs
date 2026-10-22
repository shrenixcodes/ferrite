//! Dashboard application state.
//!
//! `App` holds everything the dashboard needs to render a frame: the most
//! recent metrics, the current sort mode, the selected process, and
//! pause/help flags. It never talks to the operating system or a terminal —
//! it only transforms state, so every transition here can be unit tested
//! without a real [`crate::collector::Collector`] or terminal.

use crate::collector::{
    CpuStats, DiskInfo, MemoryStats, MetricsSnapshot, ProcessInfo, sort_processes_by_cpu_desc,
    sort_processes_by_memory_desc,
};

/// How the process table is currently sorted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    /// Highest CPU usage first.
    Cpu,
    /// Highest memory usage first.
    Memory,
}

/// Dashboard application state.
///
/// Construct with [`App::new`], feed it metrics with [`App::apply_snapshot`]
/// on every refresh, and mutate it via the navigation/sort/pause methods in
/// response to user input (see [`crate::ui::events`]).
#[derive(Debug, Clone)]
pub struct App {
    cpu: CpuStats,
    memory: MemoryStats,
    disks: Vec<DiskInfo>,
    cpu_metrics_initialized: bool,
    processes: Vec<ProcessInfo>,
    selected_pid: Option<u32>,
    sort_mode: SortMode,
    paused: bool,
    show_help: bool,
    refresh_interval_ms: u32,
}

impl App {
    /// Creates a fresh `App` with no metrics yet, using `refresh_interval_ms`
    /// (from [`crate::config::AppConfig`]) for display in the footer.
    pub fn new(refresh_interval_ms: u32) -> Self {
        Self {
            cpu: CpuStats::new(0.0, 0),
            memory: MemoryStats::new(0, 0),
            disks: Vec::new(),
            cpu_metrics_initialized: false,
            processes: Vec::new(),
            selected_pid: None,
            sort_mode: SortMode::Cpu,
            paused: false,
            show_help: false,
            refresh_interval_ms,
        }
    }

    /// Replaces the current metrics with a freshly collected snapshot,
    /// re-sorting the process list according to the active [`SortMode`] and
    /// preserving the selection where possible.
    ///
    /// If the previously selected process is still present (even at a
    /// different index, due to re-sorting), it stays selected. Otherwise the
    /// selection falls back to the same row index as before, clamped to the
    /// new (possibly shorter) list; if the list is empty, nothing is
    /// selected.
    pub fn apply_snapshot(&mut self, snapshot: MetricsSnapshot) {
        let previous_index = self.selected_index();

        self.cpu = *snapshot.cpu();
        self.memory = *snapshot.memory();
        self.disks = snapshot.disks().to_vec();
        self.cpu_metrics_initialized = snapshot.cpu_metrics_initialized();
        self.processes = snapshot.processes().to_vec();
        self.sort_processes();
        self.reconcile_selection(previous_index);
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
            true,
        );

        app.apply_snapshot(snapshot);

        assert_eq!(app.disks(), disks.as_slice());
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
}
