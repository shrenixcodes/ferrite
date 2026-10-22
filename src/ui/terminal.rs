//! Terminal lifecycle management.
//!
//! Wraps `ratatui`'s terminal setup/teardown helpers so initialization
//! failures are recoverable (no panic) and the terminal is always restored
//! exactly once, regardless of how the application exits.

use std::io;

use ratatui::DefaultTerminal;

/// An initialized terminal session (raw mode, alternate screen) that
/// restores the terminal when dropped.
///
/// `ratatui::try_init` additionally installs a panic hook that restores the
/// terminal *before* the default panic message is printed, so a panic
/// message is never swallowed by the alternate screen buffer. This guard's
/// [`Drop`] implementation handles every other exit path (a normal return,
/// or an early return via `?`), so the terminal is left in a usable state
/// no matter how the caller holding this session exits.
pub struct TerminalSession {
    terminal: DefaultTerminal,
}

impl TerminalSession {
    /// Initializes the terminal (raw mode, alternate screen, panic hook).
    ///
    /// Returns an [`io::Error`] instead of panicking if initialization
    /// fails — for example, because stdout is not a real terminal.
    pub fn start() -> io::Result<Self> {
        let terminal = ratatui::try_init()?;
        Ok(Self { terminal })
    }

    /// The underlying terminal, for drawing frames.
    pub fn terminal_mut(&mut self) -> &mut DefaultTerminal {
        &mut self.terminal
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        ratatui::restore();
    }
}
