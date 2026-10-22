//! Ferrite executable entry point.
//!
//! Launches the interactive terminal dashboard. The refresh cadence comes
//! from [`AppConfig`]; metric collection, application state, and
//! rendering are otherwise delegated to the `ferrite` library.

use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event};
use ferrite::collector::Collector;
use ferrite::config::AppConfig;
use ferrite::ui::{self, App, TerminalSession};

fn main() {
    let config = AppConfig::default();

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
    let mut app = App::new(config.refresh_interval_ms());
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

fn redraw(session: &mut TerminalSession, app: &App) -> io::Result<()> {
    session.terminal_mut().draw(|frame| ui::draw(frame, app))?;
    Ok(())
}
