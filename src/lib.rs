//! Ferrite is a terminal-native system monitor.
//!
//! This crate exposes application configuration, a system metrics collector
//! (CPU, memory, and process listing), and the interactive terminal
//! dashboard built on top of them. Alerting and historical storage are not
//! yet implemented.

pub mod collector;
pub mod config;
pub mod ui;
