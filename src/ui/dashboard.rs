//! Dashboard layout and rendering.
//!
//! Rendering only *reads* [`App`] state — it never mutates the collector or
//! the application state. The parts of this module that don't need a real
//! `Frame` (layout math, gauge ratios, byte formatting, summary-line
//! formatting) are plain functions over [`Rect`] and plain data, so they
//! are unit tested directly; the actual widget rendering is exercised
//! manually in a real terminal.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Cell, Clear, Gauge, Paragraph, Row, Sparkline, Table, TableState, Wrap,
};

use super::app::{App, SortMode};
use crate::collector::{DiskInfo, NetworkInterfaceStats};
use crate::history::History;

/// The narrowest terminal width the full dashboard layout supports.
pub const MIN_TERMINAL_WIDTH: u16 = 70;
/// The shortest terminal height the full dashboard layout supports.
pub const MIN_TERMINAL_HEIGHT: u16 = 18;

/// Whether `area` is too small to render the full dashboard layout.
pub fn is_terminal_too_small(area: Rect) -> bool {
    area.width < MIN_TERMINAL_WIDTH || area.height < MIN_TERMINAL_HEIGHT
}

/// The rectangles the dashboard's panels occupy within a given area.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DashboardLayout {
    /// Title bar at the top (includes the alert badge, if any).
    pub header: Rect,
    /// CPU gauge, left half of the metrics row.
    pub cpu: Rect,
    /// Memory gauge, right half of the metrics row.
    pub memory: Rect,
    /// CPU and memory history sparklines (one line each).
    pub history: Rect,
    /// Disk usage summary, left half of the storage/network row.
    pub disk: Rect,
    /// Network throughput summary, right half of the storage/network row.
    pub network: Rect,
    /// The process table, filling the remaining space.
    pub processes: Rect,
    /// Keybinding/status bar at the bottom.
    pub footer: Rect,
}

/// Splits `area` into the dashboard's panel rectangles.
///
/// A pure function over [`Rect`], so layout math can be verified without a
/// real terminal.
pub fn compute_layout(area: Rect) -> DashboardLayout {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // header
            Constraint::Length(3), // cpu / memory gauges
            Constraint::Length(2), // cpu / memory history sparklines
            Constraint::Length(4), // disk / network summaries
            Constraint::Min(3),    // processes
            Constraint::Length(1), // footer
        ])
        .split(area);

    let gauges = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[1]);

    let disk_network = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[3]);

    DashboardLayout {
        header: rows[0],
        cpu: gauges[0],
        memory: gauges[1],
        history: rows[2],
        disk: disk_network[0],
        network: disk_network[1],
        processes: rows[4],
        footer: rows[5],
    }
}

/// Converts a byte count to whole mebibytes, for compact table display.
pub fn bytes_to_mebibytes(bytes: u64) -> u64 {
    bytes / (1024 * 1024)
}

/// Formats a byte-per-second rate with an appropriate binary unit, for
/// compact display in the network summary.
pub fn format_rate(bytes_per_sec: u64) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = KIB * 1024.0;

    let bytes_per_sec = bytes_per_sec as f64;
    if bytes_per_sec >= MIB {
        format!("{:.1} MiB/s", bytes_per_sec / MIB)
    } else if bytes_per_sec >= KIB {
        format!("{:.1} KiB/s", bytes_per_sec / KIB)
    } else {
        format!("{bytes_per_sec:.0} B/s")
    }
}

/// A one-line, comma-separated summary of every disk's usage, or a
/// placeholder message if `disks` is empty.
pub fn disk_summary_line(disks: &[DiskInfo]) -> String {
    if disks.is_empty() {
        return "No disks detected.".to_string();
    }
    disks
        .iter()
        .map(|disk| {
            format!(
                "{} {:.1}/{:.1} GiB ({:.0}%)",
                disk.mount_point(),
                disk.used_gibibytes(),
                disk.total_gibibytes(),
                disk.used_fraction() * 100.0
            )
        })
        .collect::<Vec<_>>()
        .join("   ")
}

/// A one-line, comma-separated summary of every network interface's
/// throughput, or a placeholder message if `networks` is empty.
pub fn network_summary_line(networks: &[NetworkInterfaceStats]) -> String {
    if networks.is_empty() {
        return "No network interfaces detected.".to_string();
    }
    networks
        .iter()
        .map(|network| {
            let received = network
                .received_bytes_per_sec()
                .map_or_else(|| "—".to_string(), format_rate);
            let transmitted = network
                .transmitted_bytes_per_sec()
                .map_or_else(|| "—".to_string(), format_rate);
            format!("{} ↓{received} ↑{transmitted}", network.name())
        })
        .collect::<Vec<_>>()
        .join("   ")
}

/// Converts a [`History`] of `0.0..=1.0` utilization fractions into whole
/// percentages, for feeding a [`Sparkline`].
pub fn history_to_percent_bars(history: &History) -> Vec<u64> {
    history
        .values()
        .into_iter()
        .map(|value| (value * 100.0).round().clamp(0.0, 100.0) as u64)
        .collect()
}

fn sort_mode_label(mode: SortMode) -> &'static str {
    match mode {
        SortMode::Cpu => "CPU",
        SortMode::Memory => "Memory",
    }
}

/// Renders the whole dashboard for the current frame.
///
/// Shows a resize prompt instead of the full layout when the terminal is
/// smaller than [`MIN_TERMINAL_WIDTH`] x [`MIN_TERMINAL_HEIGHT`].
pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();

    if is_terminal_too_small(area) {
        render_too_small(frame, area);
        return;
    }

    let layout = compute_layout(area);
    render_header(frame, layout.header, app);
    render_cpu_panel(frame, layout.cpu, app);
    render_memory_panel(frame, layout.memory, app);
    render_history_row(frame, layout.history, app);
    render_disk_panel(frame, layout.disk, app);
    render_network_panel(frame, layout.network, app);
    render_process_table(frame, layout.processes, app);
    render_footer(frame, layout.footer, app);

    if app.show_help() {
        render_help_overlay(frame, area);
    }
}

fn render_too_small(frame: &mut Frame, area: Rect) {
    let message = Paragraph::new(format!(
        "Terminal too small. Resize to at least {MIN_TERMINAL_WIDTH}x{MIN_TERMINAL_HEIGHT}."
    ))
    .alignment(Alignment::Center)
    .wrap(Wrap { trim: true });
    frame.render_widget(message, area);
}

fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let paused_suffix = if app.is_paused() { " [PAUSED]" } else { "" };
    let title = format!(
        "Ferrite v{} — Dashboard{paused_suffix}",
        env!("CARGO_PKG_VERSION")
    );

    let mut spans = vec![Span::styled(
        title,
        Style::default().add_modifier(Modifier::BOLD),
    )];
    let active_alerts = app.active_alerts();
    if !active_alerts.is_empty() {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            format!("⚠ {} ALERT(S)", active_alerts.len()),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ));
    }

    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn render_cpu_panel(frame: &mut Frame, area: Rect, app: &App) {
    let cpu = app.cpu();
    let label = if app.cpu_metrics_initialized() {
        format!(
            "{:.1}% across {} core(s)",
            cpu.usage_percent(),
            cpu.core_count()
        )
    } else {
        "collecting…".to_string()
    };
    let ratio = if app.cpu_metrics_initialized() {
        cpu.utilization_fraction()
    } else {
        0.0
    };

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("CPU"))
        .gauge_style(Style::default().fg(Color::Green))
        .ratio(ratio)
        .label(label);
    frame.render_widget(gauge, area);
}

fn render_memory_panel(frame: &mut Frame, area: Rect, app: &App) {
    let memory = app.memory();
    let label = format!(
        "{} / {} MiB ({:.1}%)",
        memory.used_mebibytes(),
        memory.total_mebibytes(),
        memory.used_fraction() * 100.0
    );
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("Memory"))
        .gauge_style(Style::default().fg(Color::Cyan))
        .ratio(memory.used_fraction().clamp(0.0, 1.0))
        .label(label);
    frame.render_widget(gauge, area);
}

fn render_history_row(frame: &mut Frame, area: Rect, app: &App) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(area);

    render_history_line(
        frame,
        rows[0],
        "CPU history",
        app.cpu_history(),
        Color::Green,
    );
    render_history_line(
        frame,
        rows[1],
        "Mem history",
        app.memory_history(),
        Color::Cyan,
    );
}

fn render_history_line(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    history: &History,
    color: Color,
) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(12), Constraint::Min(1)])
        .split(area);

    frame.render_widget(Paragraph::new(label), columns[0]);

    if history.is_empty() {
        frame.render_widget(Paragraph::new("(collecting…)"), columns[1]);
        return;
    }

    let bars = history_to_percent_bars(history);
    let sparkline = Sparkline::default()
        .data(&bars)
        .max(100)
        .style(Style::default().fg(color));
    frame.render_widget(sparkline, columns[1]);
}

fn render_disk_panel(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default().borders(Borders::ALL).title("Disks");
    let text = disk_summary_line(app.disks());
    frame.render_widget(
        Paragraph::new(text).block(block).wrap(Wrap { trim: true }),
        area,
    );
}

fn render_network_panel(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default().borders(Borders::ALL).title("Network");
    let text = network_summary_line(app.networks());
    frame.render_widget(
        Paragraph::new(text).block(block).wrap(Wrap { trim: true }),
        area,
    );
}

fn process_table_title(app: &App) -> String {
    format!(
        "Processes ({}) — sorted by {}",
        app.processes().len(),
        sort_mode_label(app.sort_mode())
    )
}

fn render_process_table(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(process_table_title(app));

    if app.processes().is_empty() {
        frame.render_widget(Paragraph::new("No processes available.").block(block), area);
        return;
    }

    let header = Row::new(["PID", "Name", "CPU %", "Memory"].map(Cell::from))
        .style(Style::default().add_modifier(Modifier::BOLD));

    let rows: Vec<Row> = app
        .processes()
        .iter()
        .map(|process| {
            let cpu_cell = if app.cpu_metrics_initialized() {
                format!("{:.1}", process.cpu_usage_percent)
            } else {
                "—".to_string()
            };
            Row::new(vec![
                Cell::from(process.pid.to_string()),
                Cell::from(process.name.clone()),
                Cell::from(cpu_cell),
                Cell::from(format!("{} MiB", bytes_to_mebibytes(process.memory_bytes))),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(8),
        Constraint::Min(10),
        Constraint::Length(8),
        Constraint::Length(12),
    ];
    let table = Table::new(rows, widths)
        .header(header)
        .block(block)
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    let mut state = TableState::default();
    state.select(app.selected_index());
    frame.render_stateful_widget(table, area, &mut state);
}

fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let text = format!(
        "q/Esc quit | j/k ↑/↓ navigate | c sort CPU | m sort memory | p pause | ? help  —  \
         sort: {}  refresh: {}ms{}",
        sort_mode_label(app.sort_mode()),
        app.refresh_interval_ms(),
        if app.is_paused() { "  [PAUSED]" } else { "" }
    );
    frame.render_widget(Paragraph::new(text), area);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}

fn render_help_overlay(frame: &mut Frame, area: Rect) {
    let popup_area = centered_rect(60, 50, area);
    let help_text = "q / Esc / Ctrl+C   Quit\n\
                      j / ↓              Select next process\n\
                      k / ↑              Select previous process\n\
                      c                  Sort by CPU usage\n\
                      m                  Sort by memory usage\n\
                      p                  Pause / resume refresh\n\
                      ?                  Toggle this help view";

    frame.render_widget(Clear, popup_area);
    frame.render_widget(
        Paragraph::new(help_text).block(Block::default().borders(Borders::ALL).title("Help")),
        popup_area,
    );
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::*;

    #[test]
    fn small_terminal_is_detected_in_both_dimensions() {
        assert!(is_terminal_too_small(Rect::new(
            0,
            0,
            MIN_TERMINAL_WIDTH - 1,
            MIN_TERMINAL_HEIGHT
        )));
        assert!(is_terminal_too_small(Rect::new(
            0,
            0,
            MIN_TERMINAL_WIDTH,
            MIN_TERMINAL_HEIGHT - 1
        )));
        assert!(!is_terminal_too_small(Rect::new(
            0,
            0,
            MIN_TERMINAL_WIDTH,
            MIN_TERMINAL_HEIGHT
        )));
    }

    #[test]
    fn layout_partitions_the_area_into_non_overlapping_rows() {
        let area = Rect::new(0, 0, 100, 40);
        let layout = compute_layout(area);

        assert_eq!(layout.header.y, 0);
        assert_eq!(layout.header.height, 1);
        assert_eq!(layout.cpu.y, layout.header.y + layout.header.height);
        assert_eq!(layout.memory.y, layout.cpu.y);
        assert_eq!(layout.cpu.width + layout.memory.width, area.width);
        assert_eq!(layout.history.y, layout.cpu.y + layout.cpu.height);
        assert_eq!(layout.disk.y, layout.history.y + layout.history.height);
        assert_eq!(layout.network.y, layout.disk.y);
        assert_eq!(layout.disk.width + layout.network.width, area.width);
        assert!(layout.processes.height >= 3);
        assert_eq!(layout.footer.height, 1);
        assert_eq!(layout.footer.y + layout.footer.height, area.height);
    }

    #[test]
    fn layout_fits_within_the_minimum_supported_terminal_size() {
        let area = Rect::new(0, 0, MIN_TERMINAL_WIDTH, MIN_TERMINAL_HEIGHT);
        let layout = compute_layout(area);

        assert!(layout.processes.height >= 3);
        assert!(layout.cpu.width > 0);
        assert!(layout.memory.width > 0);
        assert!(layout.disk.width > 0);
        assert!(layout.network.width > 0);
        assert!(layout.history.height >= 2);
    }

    #[test]
    fn bytes_to_mebibytes_truncates_to_whole_units() {
        assert_eq!(bytes_to_mebibytes(2 * 1024 * 1024), 2);
        assert_eq!(bytes_to_mebibytes(1024 * 1024 + 500), 1);
        assert_eq!(bytes_to_mebibytes(0), 0);
    }

    #[test]
    fn format_rate_scales_to_the_appropriate_unit() {
        assert_eq!(format_rate(500), "500 B/s");
        assert_eq!(format_rate(2_048), "2.0 KiB/s");
        assert_eq!(format_rate(5 * 1024 * 1024), "5.0 MiB/s");
    }

    #[test]
    fn disk_summary_line_reports_a_placeholder_when_empty() {
        assert_eq!(disk_summary_line(&[]), "No disks detected.");
    }

    #[test]
    fn disk_summary_line_includes_every_disk() {
        let disks = vec![
            DiskInfo::new("disk0", "/", 1_000, 500),
            DiskInfo::new("disk1", "/data", 2_000, 1_000),
        ];
        let summary = disk_summary_line(&disks);
        assert!(summary.contains('/'));
        assert!(summary.contains("/data"));
    }

    #[test]
    fn network_summary_line_reports_a_placeholder_when_empty() {
        assert_eq!(network_summary_line(&[]), "No network interfaces detected.");
    }

    #[test]
    fn network_summary_line_shows_dash_for_unavailable_rates() {
        let networks = vec![NetworkInterfaceStats::new("eth0", 100, 50, None, None)];
        let summary = network_summary_line(&networks);
        assert!(summary.contains("eth0"));
        assert!(summary.contains('—'));
    }

    #[test]
    fn network_summary_line_shows_formatted_rates_when_available() {
        let networks = vec![NetworkInterfaceStats::new(
            "eth0",
            100,
            50,
            Some(2_048),
            Some(512),
        )];
        let summary = network_summary_line(&networks);
        assert!(summary.contains("2.0 KiB/s"));
        assert!(summary.contains("512 B/s"));
    }

    #[test]
    fn history_to_percent_bars_converts_fractions_to_whole_percentages() {
        let mut history = History::new(10);
        let t0 = Instant::now();
        history.push(t0, 0.5);
        history.push(t0 + Duration::from_secs(1), 1.0);
        history.push(t0 + Duration::from_secs(2), 0.0);

        assert_eq!(history_to_percent_bars(&history), vec![50, 100, 0]);
    }

    #[test]
    fn history_to_percent_bars_handles_an_empty_history() {
        let history = History::new(10);
        assert!(history_to_percent_bars(&history).is_empty());
    }
}
