//! The interactive terminal dashboard.
//!
//! [`app`] holds pure application state, independent of any terminal or
//! event library, so it can be unit tested on its own.

pub mod app;
pub mod dashboard;
pub mod events;
pub mod terminal;

pub use app::{App, SortMode};
pub use dashboard::draw;
pub use events::{Action, apply_action, map_key_event};
pub use terminal::TerminalSession;
