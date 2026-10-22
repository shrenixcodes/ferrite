//! Threshold alerts for supported metrics.
//!
//! [`Alert`] is a small state machine that turns a raw percentage reading
//! into an activation/recovery transition, so callers are told exactly
//! once when a threshold is crossed in either direction rather than being
//! notified every refresh while it remains exceeded. [`AlertEngine`]
//! applies this to CPU, memory, and (per-disk) disk usage from a
//! [`crate::collector::MetricsSnapshot`].
//!
//! Alerts never execute external commands or notifications — they only
//! expose state for [`crate::ui::dashboard`] to render.

use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::collector::MetricsSnapshot;

/// The smallest threshold [`Alert::new`] accepts, as a percentage.
pub const MIN_THRESHOLD_PERCENT: f32 = 0.0;
/// The largest threshold [`Alert::new`] accepts, as a percentage.
pub const MAX_THRESHOLD_PERCENT: f32 = 100.0;

/// An error returned when an alert threshold is out of range.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InvalidThreshold {
    /// The value that was rejected.
    pub value: f32,
}

impl fmt::Display for InvalidThreshold {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "alert threshold of {} is out of range: must be a finite percentage between \
             {MIN_THRESHOLD_PERCENT} and {MAX_THRESHOLD_PERCENT} inclusive",
            self.value
        )
    }
}

impl std::error::Error for InvalidThreshold {}

/// The result of evaluating a new reading against an [`Alert`]'s threshold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertTransition {
    /// The reading did not change which side of the threshold it is on.
    Unchanged,
    /// The reading just crossed above the threshold.
    Activated,
    /// The reading just dropped back below the threshold.
    Recovered,
}

/// Tracks whether a single metric is currently above its configured
/// threshold.
///
/// [`Alert::evaluate`] only returns [`AlertTransition::Activated`] or
/// [`AlertTransition::Recovered`] on the refresh where the state actually
/// changes, so a caller that only acts on those two variants cannot spam
/// repeated notifications while a threshold remains exceeded.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Alert {
    threshold_percent: f32,
    active: bool,
    last_value_percent: Option<f32>,
}

impl Alert {
    /// Creates a new, initially-inactive alert for `threshold_percent`.
    ///
    /// Rejects values that are not finite or fall outside
    /// [`MIN_THRESHOLD_PERCENT`]..=[`MAX_THRESHOLD_PERCENT`], rather than
    /// panicking or silently clamping.
    pub fn new(threshold_percent: f32) -> Result<Self, InvalidThreshold> {
        if !threshold_percent.is_finite()
            || !(MIN_THRESHOLD_PERCENT..=MAX_THRESHOLD_PERCENT).contains(&threshold_percent)
        {
            return Err(InvalidThreshold {
                value: threshold_percent,
            });
        }
        Ok(Self {
            threshold_percent,
            active: false,
            last_value_percent: None,
        })
    }

    /// The configured threshold, as a percentage.
    pub fn threshold_percent(&self) -> f32 {
        self.threshold_percent
    }

    /// Whether the most recent [`Alert::evaluate`] call found the metric
    /// at or above the threshold.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// The most recent available reading passed to [`Alert::evaluate`],
    /// or `None` if every reading so far has been unavailable.
    pub fn last_value_percent(&self) -> Option<f32> {
        self.last_value_percent
    }

    /// Evaluates a new reading, returning how the alert's state changed.
    ///
    /// `value_percent` is `None` when the metric was unavailable this
    /// cycle; evaluating `None` never activates or recovers the alert, so
    /// a missing reading can never produce a false alert or a false
    /// recovery (the previous state is preserved as-is, with
    /// [`AlertTransition::Unchanged`]).
    pub fn evaluate(&mut self, value_percent: Option<f32>) -> AlertTransition {
        let Some(value) = value_percent else {
            return AlertTransition::Unchanged;
        };
        self.last_value_percent = Some(value);

        let now_exceeds = value >= self.threshold_percent;
        let transition = match (self.active, now_exceeds) {
            (false, true) => AlertTransition::Activated,
            (true, false) => AlertTransition::Recovered,
            _ => AlertTransition::Unchanged,
        };
        self.active = now_exceeds;
        transition
    }
}

/// A currently-active alert, as reported by [`AlertEngine::active_alerts`].
#[derive(Debug, Clone, PartialEq)]
pub struct ActiveAlert {
    /// A short, human-readable description (e.g. `"CPU"` or a disk's mount
    /// point).
    pub label: String,
    /// The reading that triggered the alert, as a percentage.
    pub value_percent: f32,
    /// The configured threshold, as a percentage.
    pub threshold_percent: f32,
}

/// Evaluates CPU, memory, and per-disk alerts against a
/// [`MetricsSnapshot`].
///
/// Disk alerts are tracked per mount point in a map that is pruned of any
/// mount point not present in the latest snapshot, so a removed disk's
/// alert state does not linger forever (mirroring
/// [`crate::collector::NetworkRateTracker`]'s handling of interfaces that
/// disappear).
#[derive(Debug, Clone)]
pub struct AlertEngine {
    cpu: Alert,
    memory: Alert,
    disk_threshold_percent: f32,
    disks: HashMap<String, Alert>,
}

impl AlertEngine {
    /// Creates a new engine with independent thresholds for CPU, memory,
    /// and every disk.
    pub fn new(
        cpu_threshold_percent: f32,
        memory_threshold_percent: f32,
        disk_threshold_percent: f32,
    ) -> Result<Self, InvalidThreshold> {
        // Validate the disk threshold up front too, even though it is only
        // actually used lazily (per disk, as disks appear).
        Alert::new(disk_threshold_percent)?;
        Ok(Self {
            cpu: Alert::new(cpu_threshold_percent)?,
            memory: Alert::new(memory_threshold_percent)?,
            disk_threshold_percent,
            disks: HashMap::new(),
        })
    }

    /// Evaluates every tracked alert against `snapshot`, updating internal
    /// state. Returns nothing: call [`AlertEngine::active_alerts`]
    /// afterwards for the current state to render.
    ///
    /// CPU is evaluated as `None` (never triggering or clearing) while
    /// [`MetricsSnapshot::cpu_metrics_initialized`] is `false`, since an
    /// uninitialized CPU reading is not a real measurement.
    pub fn evaluate(&mut self, snapshot: &MetricsSnapshot) {
        let cpu_percent = snapshot.cpu_metrics_initialized().then(|| {
            let cpu = snapshot.cpu();
            if cpu.core_count() == 0 {
                0.0
            } else {
                (cpu.usage_percent() / cpu.core_count() as f32).clamp(0.0, 100.0)
            }
        });
        self.cpu.evaluate(cpu_percent);

        let memory_percent = (snapshot.memory().used_fraction() * 100.0) as f32;
        self.memory.evaluate(Some(memory_percent));

        let mut seen = HashSet::with_capacity(snapshot.disks().len());
        for disk in snapshot.disks() {
            seen.insert(disk.mount_point().to_string());
            let alert = self
                .disks
                .entry(disk.mount_point().to_string())
                .or_insert_with(|| {
                    Alert::new(self.disk_threshold_percent)
                        .expect("disk_threshold_percent was already validated in AlertEngine::new")
                });
            alert.evaluate(Some((disk.used_fraction() * 100.0) as f32));
        }
        self.disks
            .retain(|mount_point, _| seen.contains(mount_point));
    }

    /// Every alert currently active, in a stable order (CPU, then memory,
    /// then disks by mount point).
    pub fn active_alerts(&self) -> Vec<ActiveAlert> {
        let mut alerts = Vec::new();

        if self.cpu.is_active() {
            alerts.push(ActiveAlert {
                label: "CPU".to_string(),
                value_percent: self
                    .cpu
                    .last_value_percent()
                    .unwrap_or(self.cpu.threshold_percent()),
                threshold_percent: self.cpu.threshold_percent(),
            });
        }
        if self.memory.is_active() {
            alerts.push(ActiveAlert {
                label: "Memory".to_string(),
                value_percent: self
                    .memory
                    .last_value_percent()
                    .unwrap_or(self.memory.threshold_percent()),
                threshold_percent: self.memory.threshold_percent(),
            });
        }

        let mut disk_mounts: Vec<&String> = self.disks.keys().collect();
        disk_mounts.sort();
        for mount_point in disk_mounts {
            let alert = &self.disks[mount_point];
            if alert.is_active() {
                alerts.push(ActiveAlert {
                    label: format!("Disk {mount_point}"),
                    value_percent: alert
                        .last_value_percent()
                        .unwrap_or(alert.threshold_percent()),
                    threshold_percent: alert.threshold_percent(),
                });
            }
        }

        alerts
    }

    /// Whether any alert is currently active.
    pub fn has_active_alerts(&self) -> bool {
        self.cpu.is_active() || self.memory.is_active() || self.disks.values().any(Alert::is_active)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collector::{CpuStats, DiskInfo, MemoryStats, MetricsSnapshot};

    #[test]
    fn rejects_negative_threshold() {
        assert!(Alert::new(-1.0).is_err());
    }

    #[test]
    fn rejects_threshold_above_max() {
        assert!(Alert::new(100.1).is_err());
    }

    #[test]
    fn rejects_non_finite_threshold() {
        assert!(Alert::new(f32::NAN).is_err());
        assert!(Alert::new(f32::INFINITY).is_err());
    }

    #[test]
    fn accepts_boundary_thresholds() {
        assert!(Alert::new(0.0).is_ok());
        assert!(Alert::new(100.0).is_ok());
    }

    #[test]
    fn activates_when_crossing_above_threshold() {
        let mut alert = Alert::new(80.0).unwrap();
        assert_eq!(alert.evaluate(Some(50.0)), AlertTransition::Unchanged);
        assert!(!alert.is_active());

        assert_eq!(alert.evaluate(Some(85.0)), AlertTransition::Activated);
        assert!(alert.is_active());
    }

    #[test]
    fn does_not_repeat_activation_while_still_exceeded() {
        let mut alert = Alert::new(80.0).unwrap();
        alert.evaluate(Some(90.0));

        assert_eq!(alert.evaluate(Some(95.0)), AlertTransition::Unchanged);
        assert_eq!(alert.evaluate(Some(91.0)), AlertTransition::Unchanged);
        assert!(alert.is_active());
    }

    #[test]
    fn recovers_when_dropping_below_threshold() {
        let mut alert = Alert::new(80.0).unwrap();
        alert.evaluate(Some(90.0));

        assert_eq!(alert.evaluate(Some(70.0)), AlertTransition::Recovered);
        assert!(!alert.is_active());
    }

    #[test]
    fn does_not_repeat_recovery_while_still_under_threshold() {
        let mut alert = Alert::new(80.0).unwrap();
        alert.evaluate(Some(90.0));
        alert.evaluate(Some(70.0));

        assert_eq!(alert.evaluate(Some(60.0)), AlertTransition::Unchanged);
        assert!(!alert.is_active());
    }

    #[test]
    fn exactly_at_threshold_counts_as_exceeding() {
        let mut alert = Alert::new(80.0).unwrap();
        assert_eq!(alert.evaluate(Some(80.0)), AlertTransition::Activated);
    }

    #[test]
    fn unavailable_reading_never_activates_or_recovers() {
        let mut alert = Alert::new(80.0).unwrap();
        assert_eq!(alert.evaluate(None), AlertTransition::Unchanged);
        assert!(!alert.is_active());

        alert.evaluate(Some(90.0));
        assert_eq!(alert.evaluate(None), AlertTransition::Unchanged);
        assert!(
            alert.is_active(),
            "a missing reading must not clear an active alert"
        );
    }

    fn snapshot_with(
        cpu_percent: f32,
        cpu_initialized: bool,
        memory_used_fraction_percent: u64,
        disks: Vec<DiskInfo>,
    ) -> MetricsSnapshot {
        MetricsSnapshot::new(
            CpuStats::new(cpu_percent, 1),
            MemoryStats::new(100, memory_used_fraction_percent),
            disks,
            Vec::new(),
            Vec::new(),
            cpu_initialized,
        )
    }

    #[test]
    fn engine_rejects_invalid_thresholds() {
        assert!(AlertEngine::new(-1.0, 50.0, 50.0).is_err());
        assert!(AlertEngine::new(50.0, 200.0, 50.0).is_err());
        assert!(AlertEngine::new(50.0, 50.0, f32::NAN).is_err());
    }

    #[test]
    fn engine_does_not_alert_on_cpu_before_initialized() {
        let mut engine = AlertEngine::new(10.0, 90.0, 90.0).unwrap();
        let snapshot = snapshot_with(99.0, false, 0, Vec::new());

        engine.evaluate(&snapshot);

        assert!(
            !engine.has_active_alerts(),
            "uninitialized CPU must never trigger an alert"
        );
    }

    #[test]
    fn engine_reports_active_cpu_and_memory_alerts() {
        let mut engine = AlertEngine::new(50.0, 50.0, 90.0).unwrap();
        let snapshot = snapshot_with(80.0, true, 60, Vec::new());

        engine.evaluate(&snapshot);

        let active = engine.active_alerts();
        let labels: Vec<&str> = active.iter().map(|a| a.label.as_str()).collect();
        assert_eq!(labels, vec!["CPU", "Memory"]);
        assert_eq!(
            active[0].value_percent, 80.0,
            "must report the real reading, not the threshold"
        );
        assert_eq!(active[1].value_percent, 60.0);
    }

    #[test]
    fn engine_tracks_disk_alerts_per_mount_point() {
        let mut engine = AlertEngine::new(90.0, 90.0, 50.0).unwrap();
        let disks = vec![
            DiskInfo::new("disk0", "/", 1_000, 100),
            DiskInfo::new("disk1", "/data", 1_000, 900),
        ];
        let snapshot = snapshot_with(0.0, true, 0, disks);

        engine.evaluate(&snapshot);

        let active = engine.active_alerts();
        let labels: Vec<&str> = active.iter().map(|a| a.label.as_str()).collect();
        assert_eq!(labels, vec!["Disk /"]);
    }

    #[test]
    fn engine_prunes_alert_state_for_disks_that_disappear() {
        let mut engine = AlertEngine::new(90.0, 90.0, 50.0).unwrap();
        let full_disk = vec![DiskInfo::new("disk0", "/mnt/usb", 1_000, 50)];
        engine.evaluate(&snapshot_with(0.0, true, 0, full_disk));
        assert!(engine.has_active_alerts());

        // The USB drive is unplugged: it no longer appears in the snapshot.
        engine.evaluate(&snapshot_with(0.0, true, 0, Vec::new()));

        assert!(!engine.has_active_alerts());
        assert!(engine.active_alerts().is_empty());
    }
}
