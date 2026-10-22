//! Memory usage reporting.

/// A snapshot of system memory usage, measured in bytes.
///
/// `used_bytes` is clamped to `total_bytes` so that transient reporting
/// inconsistencies from the underlying platform never produce a usage
/// fraction above 100%.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryStats {
    total_bytes: u64,
    used_bytes: u64,
}

impl MemoryStats {
    /// Creates a new [`MemoryStats`] from raw byte counts.
    ///
    /// `used_bytes` is clamped to `total_bytes`.
    pub fn new(total_bytes: u64, used_bytes: u64) -> Self {
        Self {
            total_bytes,
            used_bytes: used_bytes.min(total_bytes),
        }
    }

    /// Total physical memory, in bytes.
    pub fn total_bytes(&self) -> u64 {
        self.total_bytes
    }

    /// Used physical memory, in bytes.
    pub fn used_bytes(&self) -> u64 {
        self.used_bytes
    }

    /// Available (unused) physical memory, in bytes.
    pub fn available_bytes(&self) -> u64 {
        self.total_bytes.saturating_sub(self.used_bytes)
    }

    /// Used memory as a fraction of total memory, in `0.0..=1.0`.
    ///
    /// Returns `0.0` when `total_bytes` is `0`, rather than dividing by
    /// zero.
    pub fn used_fraction(&self) -> f64 {
        if self.total_bytes == 0 {
            0.0
        } else {
            self.used_bytes as f64 / self.total_bytes as f64
        }
    }

    /// Total memory, in whole mebibytes (1 MiB = 1,048,576 bytes).
    pub fn total_mebibytes(&self) -> u64 {
        self.total_bytes / (1024 * 1024)
    }

    /// Used memory, in whole mebibytes (1 MiB = 1,048,576 bytes).
    pub fn used_mebibytes(&self) -> u64 {
        self.used_bytes / (1024 * 1024)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_raw_byte_counts() {
        let stats = MemoryStats::new(8_000_000_000, 4_000_000_000);
        assert_eq!(stats.total_bytes(), 8_000_000_000);
        assert_eq!(stats.used_bytes(), 4_000_000_000);
    }

    #[test]
    fn computes_available_bytes() {
        let stats = MemoryStats::new(1_000, 400);
        assert_eq!(stats.available_bytes(), 600);
    }

    #[test]
    fn clamps_used_bytes_to_total() {
        let stats = MemoryStats::new(1_000, 1_500);
        assert_eq!(stats.used_bytes(), 1_000);
        assert_eq!(stats.available_bytes(), 0);
    }

    #[test]
    fn computes_used_fraction() {
        let stats = MemoryStats::new(1_000, 250);
        assert_eq!(stats.used_fraction(), 0.25);
    }

    #[test]
    fn used_fraction_is_zero_when_total_is_zero() {
        let stats = MemoryStats::new(0, 0);
        assert_eq!(stats.used_fraction(), 0.0);
    }

    #[test]
    fn converts_bytes_to_mebibytes() {
        let stats = MemoryStats::new(2 * 1024 * 1024, 1024 * 1024);
        assert_eq!(stats.total_mebibytes(), 2);
        assert_eq!(stats.used_mebibytes(), 1);
    }
}
