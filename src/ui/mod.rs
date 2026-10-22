//! The interactive terminal dashboard.
//!
//! [`app`] holds pure application state, independent of any terminal or
//! event library, so it can be unit tested on its own.

pub mod app;

pub use app::{App, SortMode};
