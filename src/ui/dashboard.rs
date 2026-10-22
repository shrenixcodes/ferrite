//! Dashboard layout and rendering.
//!
//! Rendering only *reads* [`App`] state — it never mutates the collector or
//! the application state. The parts of this module that don't need a real
//! `Frame` (layout math, gauge ratios, byte formatting) are plain functions
//! over [`Rect`] and numeric data, so they are unit tested directly; the
//! actual widget rendering is exercised manually in a real terminal.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{
    Block, Borders, Cell, Clear, Gauge, Paragraph, Row, Table, TableState, Wrap,
};

use super::app::{App, SortMode};

/// The narrowest terminal width the full dashboard layout supports.
pub const MIN_TERMINAL_WIDTH: u16 = 60;
/// The shortest terminal height the full dashboard layout supports.
pub const MIN_TERMINAL_HEIGHT: u16 = 14;

/// Whether `area` is too small to render the full dashboard layout.
pub fn is_terminal_too_small(area: Rect) -> bool {
    area.width < MIN_TERMINAL_WIDTH || area.height < MIN_TERMINAL_HEIGHT
}

/// The rectangles the dashboard's panels occupy within a given area.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DashboardLayout {
    /// Title bar at the top.
    pub header: Rect,
    /// CPU gauge, left half of the metrics row.
    pub cpu: Rect,
    /// Memory gauge, right half of the metrics row.
    pub memory: Rect,
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
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(3),
            Constraint::Length(1),
        ])
        .split(area);

    let gauges = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[1]);

    DashboardLayout {
        header: rows[0],
        cpu: gauges[0],
        memory: gauges[1],
        processes: rows[2],
        footer: rows[3],
    }
}

/// Converts a byte count to whole mebibytes, for compact table display.
pub fn bytes_to_mebibytes(bytes: u64) -> u64 {
    bytes / (1024 * 1024)
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
    let status = if app.is_paused() { " [PAUSED]" } else { "" };
    let title = format!("Ferrite v{} — Dashboard{status}", env!("CARGO_PKG_VERSION"));
    frame.render_widget(
        Paragraph::new(title).style(Style::default().add_modifier(Modifier::BOLD)),
        area,
    );
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
    fn layout_partitions_the_area_into_five_non_overlapping_rows() {
        let area = Rect::new(0, 0, 100, 40);
        let layout = compute_layout(area);

        assert_eq!(layout.header.y, 0);
        assert_eq!(layout.header.height, 1);
        assert_eq!(layout.cpu.y, layout.header.y + layout.header.height);
        assert_eq!(layout.memory.y, layout.cpu.y);
        assert_eq!(layout.cpu.width + layout.memory.width, area.width);
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
    }

    #[test]
    fn bytes_to_mebibytes_truncates_to_whole_units() {
        assert_eq!(bytes_to_mebibytes(2 * 1024 * 1024), 2);
        assert_eq!(bytes_to_mebibytes(1024 * 1024 + 500), 1);
        assert_eq!(bytes_to_mebibytes(0), 0);
    }
}
