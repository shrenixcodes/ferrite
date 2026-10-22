//! Keyboard input handling.
//!
//! Key-to-action mapping ([`map_key_event`]) and action application
//! ([`apply_action`]) are both pure functions over plain data ([`KeyEvent`],
//! [`App`]), so the dashboard's keyboard behavior can be unit tested
//! without a real terminal.

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use super::app::{App, SortMode};

/// A user-triggered intent, decoupled from the raw key event so the mapping
/// and its effects can be tested independently of `crossterm`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Quit the application.
    Quit,
    /// Move the process selection down one row.
    SelectNext,
    /// Move the process selection up one row.
    SelectPrevious,
    /// Sort the process table by CPU usage.
    SortByCpu,
    /// Sort the process table by memory usage.
    SortByMemory,
    /// Pause or resume metric refreshing.
    TogglePause,
    /// Show or hide the help overlay.
    ToggleHelp,
}

/// Maps a key event to an [`Action`], or `None` if the key has no bound
/// action.
///
/// Only key-press events are mapped; key releases (and repeats, which some
/// terminals report as a distinct [`KeyEventKind`]) are ignored here so a
/// single physical keystroke cannot trigger an action more than once.
///
/// Raw mode (which the dashboard enables) stops the terminal from turning
/// Ctrl+C into a `SIGINT` — it arrives here as an ordinary key event
/// instead, so it is mapped to [`Action::Quit`] explicitly as a safety net
/// for anyone reflexively hitting it.
pub fn map_key_event(key: KeyEvent) -> Option<Action> {
    if key.kind != KeyEventKind::Press {
        return None;
    }

    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return Some(Action::Quit);
    }

    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => Some(Action::Quit),
        KeyCode::Char('j') | KeyCode::Down => Some(Action::SelectNext),
        KeyCode::Char('k') | KeyCode::Up => Some(Action::SelectPrevious),
        KeyCode::Char('c' | 'C') => Some(Action::SortByCpu),
        KeyCode::Char('m' | 'M') => Some(Action::SortByMemory),
        KeyCode::Char('p' | 'P') => Some(Action::TogglePause),
        KeyCode::Char('?') => Some(Action::ToggleHelp),
        _ => None,
    }
}

/// Applies `action` to `app`, returning `true` if the application should
/// quit.
pub fn apply_action(app: &mut App, action: Action) -> bool {
    match action {
        Action::Quit => return true,
        Action::SelectNext => app.select_next(),
        Action::SelectPrevious => app.select_previous(),
        Action::SortByCpu => app.set_sort_mode(SortMode::Cpu),
        Action::SortByMemory => app.set_sort_mode(SortMode::Memory),
        Action::TogglePause => {
            app.toggle_pause();
        }
        Action::ToggleHelp => {
            app.toggle_help();
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use crossterm::event::KeyEventState;

    use super::*;
    use crate::collector::{CpuStats, MemoryStats, MetricsSnapshot, ProcessInfo};

    fn press(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn release(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Release,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn maps_quit_keys() {
        assert_eq!(map_key_event(press(KeyCode::Char('q'))), Some(Action::Quit));
        assert_eq!(map_key_event(press(KeyCode::Esc)), Some(Action::Quit));
    }

    #[test]
    fn maps_ctrl_c_to_quit() {
        let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(map_key_event(ctrl_c), Some(Action::Quit));
    }

    #[test]
    fn plain_c_without_control_still_sorts_by_cpu() {
        assert_eq!(
            map_key_event(press(KeyCode::Char('c'))),
            Some(Action::SortByCpu)
        );
    }

    #[test]
    fn maps_navigation_keys() {
        assert_eq!(
            map_key_event(press(KeyCode::Char('j'))),
            Some(Action::SelectNext)
        );
        assert_eq!(
            map_key_event(press(KeyCode::Down)),
            Some(Action::SelectNext)
        );
        assert_eq!(
            map_key_event(press(KeyCode::Char('k'))),
            Some(Action::SelectPrevious)
        );
        assert_eq!(
            map_key_event(press(KeyCode::Up)),
            Some(Action::SelectPrevious)
        );
    }

    #[test]
    fn maps_sort_keys_case_insensitively() {
        assert_eq!(
            map_key_event(press(KeyCode::Char('c'))),
            Some(Action::SortByCpu)
        );
        assert_eq!(
            map_key_event(press(KeyCode::Char('C'))),
            Some(Action::SortByCpu)
        );
        assert_eq!(
            map_key_event(press(KeyCode::Char('m'))),
            Some(Action::SortByMemory)
        );
        assert_eq!(
            map_key_event(press(KeyCode::Char('M'))),
            Some(Action::SortByMemory)
        );
    }

    #[test]
    fn maps_pause_and_help_keys() {
        assert_eq!(
            map_key_event(press(KeyCode::Char('p'))),
            Some(Action::TogglePause)
        );
        assert_eq!(
            map_key_event(press(KeyCode::Char('?'))),
            Some(Action::ToggleHelp)
        );
    }

    #[test]
    fn unmapped_key_returns_none() {
        assert_eq!(map_key_event(press(KeyCode::Char('x'))), None);
    }

    #[test]
    fn release_events_are_ignored() {
        assert_eq!(map_key_event(release(KeyCode::Char('q'))), None);
    }

    fn app_with_two_processes() -> App {
        let mut app = App::new(1_000);
        app.apply_snapshot(MetricsSnapshot::new(
            CpuStats::new(10.0, 4),
            MemoryStats::new(1_000, 400),
            Vec::new(),
            Vec::new(),
            vec![
                ProcessInfo::new(1, "a", 50.0, 10),
                ProcessInfo::new(2, "b", 10.0, 90),
            ],
            true,
        ));
        app
    }

    #[test]
    fn quit_action_requests_exit_without_mutating_app() {
        let mut app = app_with_two_processes();
        let before = app.selected_index();
        assert!(apply_action(&mut app, Action::Quit));
        assert_eq!(app.selected_index(), before);
    }

    #[test]
    fn select_next_action_moves_selection() {
        let mut app = app_with_two_processes();
        assert_eq!(app.selected_index(), Some(0));
        assert!(!apply_action(&mut app, Action::SelectNext));
        assert_eq!(app.selected_index(), Some(1));
    }

    #[test]
    fn sort_action_changes_sort_mode() {
        let mut app = app_with_two_processes();
        assert_eq!(app.sort_mode(), SortMode::Cpu);
        assert!(!apply_action(&mut app, Action::SortByMemory));
        assert_eq!(app.sort_mode(), SortMode::Memory);
    }

    #[test]
    fn toggle_pause_action_flips_pause_state() {
        let mut app = app_with_two_processes();
        assert!(!app.is_paused());
        assert!(!apply_action(&mut app, Action::TogglePause));
        assert!(app.is_paused());
    }

    #[test]
    fn toggle_help_action_flips_help_state() {
        let mut app = app_with_two_processes();
        assert!(!app.show_help());
        assert!(!apply_action(&mut app, Action::ToggleHelp));
        assert!(app.show_help());
    }
}
