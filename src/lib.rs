//! Ferrite is a terminal-native system monitor.
//!
//! This crate exposes application configuration, a system metrics collector
//! (CPU, memory, disk, network, and process listing), bounded in-memory
//! metric history, and the interactive terminal dashboard built on top of
//! them.

pub mod collector;
pub mod config;
pub mod history;
pub mod ui;
