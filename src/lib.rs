//! Ferrite is a terminal-native system monitor.
//!
//! This crate currently exposes the application configuration API and a
//! system metrics collector (CPU, memory, and process listing). Alerting,
//! historical storage, and the terminal dashboard are not yet implemented.

pub mod collector;
pub mod config;
