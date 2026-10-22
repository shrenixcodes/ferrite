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

    /// The fraction of total CPU capacity in use, in `0.0..=1.0`.
    ///
    /// [`Self::usage_percent`] is a sum across cores (so it can exceed
    /// `100.0`), while this is a ratio of total capacity across every
    /// core — the basis for both the dashboard's CPU gauge and CPU alert
    /// evaluation. Guards against `core_count == 0` and clamps away any
    /// floating-point rounding that could otherwise push the result
    /// outside `0.0..=1.0`.
    pub fn utilization_fraction(&self) -> f64 {
        if self.core_count == 0 {
            return 0.0;
        }
        let capacity = self.core_count as f64 * 100.0;
        (f64::from(self.usage_percent) / capacity).clamp(0.0, 1.0)
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

    #[test]
    fn utilization_fraction_is_fraction_of_total_capacity() {
        let stats = CpuStats::new(200.0, 4);
        assert!((stats.utilization_fraction() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn utilization_fraction_clamps_above_full_load() {
        let stats = CpuStats::new(999.0, 4);
        assert_eq!(stats.utilization_fraction(), 1.0);
    }

    #[test]
    fn utilization_fraction_handles_zero_cores_without_dividing_by_zero() {
        let stats = CpuStats::new(0.0, 0);
        assert_eq!(stats.utilization_fraction(), 0.0);
    }
}
