//! System metrics collection.
//!
//! This module is the only place in the crate that talks to the operating
//! system for resource metrics. Everything else — [`CpuStats`],
//! [`MemoryStats`], [`DiskInfo`], [`NetworkInterfaceStats`],
//! [`ProcessInfo`], [`MetricsSnapshot`], and the pure transformation
//! helpers [`sort_processes_by_cpu_desc`], [`sort_processes_by_memory_desc`],
//! and [`bytes_per_second`] — is a plain data type or pure function, so it
//! can be constructed and tested without a real system collector.

mod cpu;
mod disk;
mod memory;
mod network;
mod process;
mod snapshot;
mod system;

pub use cpu::CpuStats;
pub use disk::DiskInfo;
pub use memory::MemoryStats;
pub use network::{NetworkInterfaceStats, NetworkRateTracker, bytes_per_second};
pub use process::{ProcessInfo, sort_processes_by_cpu_desc, sort_processes_by_memory_desc};
pub use snapshot::MetricsSnapshot;
pub use system::Collector;
