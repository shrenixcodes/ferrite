//! System metrics collection.
//!
//! This module is the only place in the crate that talks to the operating
//! system for resource metrics. Everything else — [`CpuStats`],
//! [`MemoryStats`], [`ProcessInfo`], [`MetricsSnapshot`], and the sorting
//! helpers [`sort_processes_by_cpu_desc`] and [`sort_processes_by_memory_desc`]
//! — is a plain data type or pure function, so it can be constructed and
//! tested without a real system collector.

mod cpu;
mod memory;
mod process;
mod snapshot;
mod system;

pub use cpu::CpuStats;
pub use memory::MemoryStats;
pub use process::{ProcessInfo, sort_processes_by_cpu_desc, sort_processes_by_memory_desc};
pub use snapshot::MetricsSnapshot;
pub use system::Collector;
