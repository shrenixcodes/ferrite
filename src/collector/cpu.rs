//! CPU usage reporting.

/// A snapshot of system-wide CPU usage.
///
/// `usage_percent` is the sum of all core usage, expressed as a percentage
/// of a single core's capacity (e.g. a fully loaded 4-core machine reports
/// approximately `400.0`). See
/// [`crate::collector::MetricsSnapshot::cpu_metrics_initialized`] for when
/// this value is actually meaningful.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CpuStats {
    usage_percent: f32,
    core_count: usize,
}

impl CpuStats {
    /// Creates a new [`CpuStats`] from a global usage percentage and the
    /// number of logical cores it was computed over.
    pub fn new(usage_percent: f32, core_count: usize) -> Self {
        Self {
            usage_percent,
            core_count,
        }
    }

    /// Global CPU usage, summed across all logical cores.
    pub fn usage_percent(&self) -> f32 {
        self.usage_percent
    }

    /// The number of logical cores detected on this host.
    pub fn core_count(&self) -> usize {
        self.core_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stores_usage_and_core_count() {
        let stats = CpuStats::new(42.5, 8);
        assert_eq!(stats.usage_percent(), 42.5);
        assert_eq!(stats.core_count(), 8);
    }

    #[test]
    fn zero_cores_is_representable_without_panicking() {
        let stats = CpuStats::new(0.0, 0);
        assert_eq!(stats.core_count(), 0);
    }
}
