//! Ferrite executable entry point.
//!
//! Launches the interactive terminal dashboard. Configuration comes from
//! an optional TOML file (falling back to documented defaults — see
//! [`ferrite::config::file`]); metric collection, application state, and
//! rendering are otherwise delegated to the `ferrite` library.

use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event};
use ferrite::collector::Collector;
use ferrite::config::{self, AppConfig};
use ferrite::ui::{self, App, AppOptions, SortMode, TerminalSession};

fn main() {
    let config = match config::file::load_or_default() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("ferrite: invalid configuration: {error}");
            std::process::exit(1);
        }
    };

    if let Err(error) = run(config) {
        eprintln!("ferrite: {error}");
        std::process::exit(1);
    }
}

/// Runs the dashboard until the user quits or an unrecoverable I/O error
/// occurs.
fn run(config: AppConfig) -> io::Result<()> {
    let mut session = TerminalSession::start()?;
    let mut collector = Collector::new();
    let mut app = App::with_options(app_options_from(&config));
    let refresh_interval = Duration::from_millis(u64::from(config.refresh_interval_ms()));
    // Bound how long a single poll blocks so a long refresh interval still
    // leaves the UI responsive to key presses and resizes.
    let poll_timeout = Duration::from_millis(100).min(refresh_interval);

    app.apply_snapshot(collector.refresh());
    redraw(&mut session, &app)?;
    let mut last_refresh = Instant::now();

    loop {
        if !app.is_paused() && last_refresh.elapsed() >= refresh_interval {
            app.apply_snapshot(collector.refresh());
            last_refresh = Instant::now();
            redraw(&mut session, &app)?;
        }

        if event::poll(poll_timeout)? {
            match event::read()? {
                Event::Key(key) => {
                    if let Some(action) = ui::map_key_event(key) {
                        if ui::apply_action(&mut app, action) {
                            return Ok(());
                        }
                        redraw(&mut session, &app)?;
                    }
                }
                Event::Resize(_, _) => redraw(&mut session, &app)?,
                _ => {}
            }
        }
    }
}

/// Converts a validated [`AppConfig`] into the [`AppOptions`] `App`
/// expects. Kept here, rather than in the library, so `ui` does not need
/// to depend on `config`'s types.
fn app_options_from(config: &AppConfig) -> AppOptions {
    AppOptions {
        refresh_interval_ms: config.refresh_interval_ms(),
        history_capacity: config.history_capacity(),
        cpu_alert_threshold_percent: config.cpu_alert_threshold_percent(),
        memory_alert_threshold_percent: config.memory_alert_threshold_percent(),
        disk_alert_threshold_percent: config.disk_alert_threshold_percent(),
        default_sort_mode: match config.default_sort_mode() {
            config::DefaultSortMode::Cpu => SortMode::Cpu,
            config::DefaultSortMode::Memory => SortMode::Memory,
        },
    }
}

fn redraw(session: &mut TerminalSession, app: &App) -> io::Result<()> {
    session.terminal_mut().draw(|frame| ui::draw(frame, app))?;
    Ok(())
}
