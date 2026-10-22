//! Network traffic rate reporting.
//!
//! [`bytes_per_second`] and [`NetworkRateTracker`] are pure, independently
//! testable: they operate on plain `(name, total_bytes)` tuples and
//! [`Instant`] values, with no dependency on `sysinfo` or any other
//! operating-system API. [`crate::collector::system::Collector`] is the
//! only place that feeds them real data.

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

/// A snapshot of one network interface's traffic.
///
/// `received_bytes_per_sec` and `transmitted_bytes_per_sec` are `None`
/// until a second sample has been taken for this interface — the first
/// sample has no prior counter value to diff against, so reporting a rate
/// for it would be misleading (see [`NetworkRateTracker::update`]).
#[derive(Debug, Clone, PartialEq)]
pub struct NetworkInterfaceStats {
    name: String,
    total_received_bytes: u64,
    total_transmitted_bytes: u64,
    received_bytes_per_sec: Option<u64>,
    transmitted_bytes_per_sec: Option<u64>,
}

impl NetworkInterfaceStats {
    /// Creates a new [`NetworkInterfaceStats`] from its component fields.
    pub fn new(
        name: impl Into<String>,
        total_received_bytes: u64,
        total_transmitted_bytes: u64,
        received_bytes_per_sec: Option<u64>,
        transmitted_bytes_per_sec: Option<u64>,
    ) -> Self {
        Self {
            name: name.into(),
            total_received_bytes,
            total_transmitted_bytes,
            received_bytes_per_sec,
            transmitted_bytes_per_sec,
        }
    }

    /// The interface name (e.g. `eth0` or `Wi-Fi`), as reported by the
    /// operating system.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Cumulative bytes received since the interface came up, as reported
    /// by the operating system.
    pub fn total_received_bytes(&self) -> u64 {
        self.total_received_bytes
    }

    /// Cumulative bytes transmitted since the interface came up, as
    /// reported by the operating system.
    pub fn total_transmitted_bytes(&self) -> u64 {
        self.total_transmitted_bytes
    }

    /// Received bytes per second since the previous sample, or `None` if
    /// this is the first sample seen for this interface.
    pub fn received_bytes_per_sec(&self) -> Option<u64> {
        self.received_bytes_per_sec
    }

    /// Transmitted bytes per second since the previous sample, or `None`
    /// if this is the first sample seen for this interface.
    pub fn transmitted_bytes_per_sec(&self) -> Option<u64> {
        self.transmitted_bytes_per_sec
    }
}

/// Computes a byte-per-second rate from two cumulative counter samples and
/// the elapsed time between them.
///
/// Returns `0` instead of panicking or producing a nonsensical value when:
/// - `elapsed` is zero or negative-length (impossible for a real
///   [`Duration`], but guards against a zero divide all the same).
/// - `current_total` is less than `previous_total` — a counter reset (the
///   interface restarted, or the OS counter wrapped). `saturating_sub`
///   clamps the delta to `0` rather than underflowing into a huge value,
///   which unsigned subtraction would otherwise produce.
pub fn bytes_per_second(previous_total: u64, current_total: u64, elapsed: Duration) -> u64 {
    let elapsed_secs = elapsed.as_secs_f64();
    if elapsed_secs <= 0.0 {
        return 0;
    }
    let delta_bytes = current_total.saturating_sub(previous_total);
    (delta_bytes as f64 / elapsed_secs).round() as u64
}

/// Tracks successive per-interface counter samples so [`bytes_per_second`]
/// rates can be computed across refreshes.
///
/// Interfaces that stop appearing in [`NetworkRateTracker::update`] are
/// dropped from the tracker, so memory does not grow unbounded as
/// interfaces come and go (for example, a VPN connecting and
/// disconnecting). If an interface reappears later, it is treated as a
/// fresh first sample rather than computing a rate against a stale,
/// long-past timestamp.
#[derive(Debug, Default)]
pub struct NetworkRateTracker {
    previous: HashMap<String, (u64, u64, Instant)>,
}

impl NetworkRateTracker {
    /// Creates an empty tracker with no prior samples.
    pub fn new() -> Self {
        Self::default()
    }

    /// Given the current cumulative `(name, total_received, total_transmitted)`
    /// for every interface, returns per-interface stats with rates computed
    /// against the previous sample for that interface, if any.
    pub fn update(
        &mut self,
        current: &[(String, u64, u64)],
        now: Instant,
    ) -> Vec<NetworkInterfaceStats> {
        let mut seen = HashSet::with_capacity(current.len());

        let stats = current
            .iter()
            .map(|(name, rx_total, tx_total)| {
                seen.insert(name.clone());

                let rates = self
                    .previous
                    .get(name)
                    .map(|&(prev_rx, prev_tx, prev_time)| {
                        let elapsed = now.saturating_duration_since(prev_time);
                        (
                            bytes_per_second(prev_rx, *rx_total, elapsed),
                            bytes_per_second(prev_tx, *tx_total, elapsed),
                        )
                    });

                self.previous
                    .insert(name.clone(), (*rx_total, *tx_total, now));

                NetworkInterfaceStats {
                    name: name.clone(),
                    total_received_bytes: *rx_total,
                    total_transmitted_bytes: *tx_total,
                    received_bytes_per_sec: rates.map(|(rx, _)| rx),
                    transmitted_bytes_per_sec: rates.map(|(_, tx)| tx),
                }
            })
            .collect();

        self.previous.retain(|name, _| seen.contains(name));

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_is_delta_over_elapsed_time() {
        let rate = bytes_per_second(1_000, 3_000, Duration::from_secs(2));
        assert_eq!(rate, 1_000);
    }

    #[test]
    fn rate_is_zero_for_zero_elapsed_time() {
        let rate = bytes_per_second(1_000, 3_000, Duration::ZERO);
        assert_eq!(rate, 0);
    }

    #[test]
    fn rate_does_not_underflow_on_counter_reset() {
        // current < previous: the counter must have reset (e.g. interface
        // restart), not gone negative.
        let rate = bytes_per_second(5_000, 100, Duration::from_secs(1));
        assert_eq!(rate, 0);
    }

    #[test]
    fn rate_handles_equal_totals() {
        let rate = bytes_per_second(1_000, 1_000, Duration::from_secs(1));
        assert_eq!(rate, 0);
    }

    #[test]
    fn first_sample_reports_no_rate() {
        let mut tracker = NetworkRateTracker::new();
        let stats = tracker.update(&[("eth0".to_string(), 1_000, 500)], Instant::now());

        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].name(), "eth0");
        assert_eq!(stats[0].total_received_bytes(), 1_000);
        assert_eq!(stats[0].received_bytes_per_sec(), None);
        assert_eq!(stats[0].transmitted_bytes_per_sec(), None);
    }

    #[test]
    fn second_sample_reports_a_real_rate() {
        let mut tracker = NetworkRateTracker::new();
        let t0 = Instant::now();
        tracker.update(&[("eth0".to_string(), 1_000, 500)], t0);

        let t1 = t0 + Duration::from_secs(2);
        let stats = tracker.update(&[("eth0".to_string(), 5_000, 1_500)], t1);

        assert_eq!(stats[0].received_bytes_per_sec(), Some(2_000));
        assert_eq!(stats[0].transmitted_bytes_per_sec(), Some(500));
    }

    #[test]
    fn disappeared_interface_is_dropped_from_the_tracker() {
        let mut tracker = NetworkRateTracker::new();
        let t0 = Instant::now();
        tracker.update(&[("eth0".to_string(), 1_000, 500)], t0);
        assert_eq!(tracker.previous.len(), 1);

        tracker.update(&[], t0 + Duration::from_secs(1));
        assert_eq!(tracker.previous.len(), 0);
    }

    #[test]
    fn reappearing_interface_is_treated_as_a_first_sample() {
        let mut tracker = NetworkRateTracker::new();
        let t0 = Instant::now();
        tracker.update(&[("eth0".to_string(), 1_000, 500)], t0);
        // Interface disappears for a while.
        tracker.update(&[], t0 + Duration::from_secs(5));
        // ...then reappears with a fresh counter.
        let stats = tracker.update(&[("eth0".to_string(), 10, 5)], t0 + Duration::from_secs(10));

        assert_eq!(stats[0].received_bytes_per_sec(), None);
        assert_eq!(stats[0].transmitted_bytes_per_sec(), None);
    }

    #[test]
    fn empty_interface_list_produces_empty_stats() {
        let mut tracker = NetworkRateTracker::new();
        let stats = tracker.update(&[], Instant::now());
        assert!(stats.is_empty());
    }

    #[test]
    fn multiple_interfaces_are_tracked_independently() {
        let mut tracker = NetworkRateTracker::new();
        let t0 = Instant::now();
        tracker.update(
            &[("eth0".to_string(), 0, 0), ("wlan0".to_string(), 0, 0)],
            t0,
        );

        let t1 = t0 + Duration::from_secs(1);
        let stats = tracker.update(
            &[("eth0".to_string(), 100, 0), ("wlan0".to_string(), 0, 200)],
            t1,
        );

        let eth0 = stats.iter().find(|s| s.name() == "eth0").unwrap();
        let wlan0 = stats.iter().find(|s| s.name() == "wlan0").unwrap();
        assert_eq!(eth0.received_bytes_per_sec(), Some(100));
        assert_eq!(wlan0.transmitted_bytes_per_sec(), Some(200));
    }
}
