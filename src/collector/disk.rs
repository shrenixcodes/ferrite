//! Disk usage reporting.

/// A snapshot of one disk/mount's storage usage, measured in bytes.
///
/// `available_bytes` is clamped to `total_bytes`, mirroring
/// [`crate::collector::MemoryStats`], so a platform quirk can never report
/// more available space than the disk's total capacity.
///
/// `available_bytes` is the filesystem's free space as reported by the
/// operating system, which is not always exactly `total_bytes -
/// used_bytes`: some filesystems (for example ext4's reserved blocks)
/// withhold a margin that counts toward neither "used" nor "available" in
/// other tools' accounting. [`DiskInfo::used_bytes`] here is simply
/// `total_bytes - available_bytes`, which matches what most disk-usage
/// tools display as "used," but is an approximation of true file content
/// size on filesystems with such reservations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiskInfo {
    name: String,
    mount_point: String,
    total_bytes: u64,
    available_bytes: u64,
}

impl DiskInfo {
    /// Creates a new [`DiskInfo`] from raw values.
    ///
    /// `available_bytes` is clamped to `total_bytes`.
    pub fn new(
        name: impl Into<String>,
        mount_point: impl Into<String>,
        total_bytes: u64,
        available_bytes: u64,
    ) -> Self {
        Self {
            name: name.into(),
            mount_point: mount_point.into(),
            total_bytes,
            available_bytes: available_bytes.min(total_bytes),
        }
    }

    /// The disk or volume name, as reported by the operating system.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The filesystem mount point (e.g. `/` or `C:\`).
    ///
    /// No particular path format is assumed; this is whatever the
    /// operating system reports, so it is not Linux-specific.
    pub fn mount_point(&self) -> &str {
        &self.mount_point
    }

    /// Total capacity, in bytes.
    pub fn total_bytes(&self) -> u64 {
        self.total_bytes
    }

    /// Available (free) space, in bytes.
    pub fn available_bytes(&self) -> u64 {
        self.available_bytes
    }

    /// Used space, in bytes (`total_bytes - available_bytes`).
    pub fn used_bytes(&self) -> u64 {
        self.total_bytes.saturating_sub(self.available_bytes)
    }

    /// Used space as a fraction of total capacity, in `0.0..=1.0`.
    ///
    /// Returns `0.0` when `total_bytes` is `0`, rather than dividing by
    /// zero.
    pub fn used_fraction(&self) -> f64 {
        if self.total_bytes == 0 {
            0.0
        } else {
            self.used_bytes() as f64 / self.total_bytes as f64
        }
    }

    /// Total capacity, in whole gibibytes (1 GiB = 1,073,741,824 bytes).
    pub fn total_gibibytes(&self) -> f64 {
        self.total_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
    }

    /// Used space, in whole gibibytes (1 GiB = 1,073,741,824 bytes).
    pub fn used_gibibytes(&self) -> f64 {
        self.used_bytes() as f64 / (1024.0 * 1024.0 * 1024.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn disk(total: u64, available: u64) -> DiskInfo {
        DiskInfo::new("disk0", "/mnt/test", total, available)
    }

    #[test]
    fn reports_name_and_mount_point() {
        let disk = disk(1_000, 400);
        assert_eq!(disk.name(), "disk0");
        assert_eq!(disk.mount_point(), "/mnt/test");
    }

    #[test]
    fn computes_used_bytes() {
        let disk = disk(1_000, 400);
        assert_eq!(disk.used_bytes(), 600);
    }

    #[test]
    fn clamps_available_bytes_to_total() {
        let disk = disk(1_000, 1_500);
        assert_eq!(disk.available_bytes(), 1_000);
        assert_eq!(disk.used_bytes(), 0);
    }

    #[test]
    fn computes_used_fraction() {
        let disk = disk(1_000, 250);
        assert_eq!(disk.used_fraction(), 0.75);
    }

    #[test]
    fn used_fraction_is_zero_for_zero_capacity_disk() {
        let disk = disk(0, 0);
        assert_eq!(disk.used_fraction(), 0.0);
    }

    #[test]
    fn converts_bytes_to_gibibytes() {
        let disk = disk(2 * 1024 * 1024 * 1024, 1024 * 1024 * 1024);
        assert_eq!(disk.total_gibibytes(), 2.0);
        assert_eq!(disk.used_gibibytes(), 1.0);
    }

    #[test]
    fn fully_available_disk_has_zero_used_bytes() {
        let disk = disk(1_000, 1_000);
        assert_eq!(disk.used_bytes(), 0);
        assert_eq!(disk.used_fraction(), 0.0);
    }
}
