//! Bounded, timestamped metric history.
//!
//! [`History`] is a small, generic, in-memory-only component: it has no
//! dependency on the collector, configuration, or UI, so it can store a
//! time series for any numeric metric (CPU utilization, memory
//! utilization, network throughput, ...) and is independently testable.
//! It does not persist anything to disk — all samples are lost when the
//! application exits.

use std::collections::VecDeque;
use std::time::Instant;

/// The default number of samples a [`History`] retains, chosen to cover a
/// couple of minutes at the default 1-second refresh interval without
/// using much memory.
pub const DEFAULT_HISTORY_CAPACITY: usize = 120;

/// A single timestamped sample.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sample {
    /// When the sample was recorded.
    pub timestamp: Instant,
    /// The recorded value. Unitless — callers agree on what it means
    /// (e.g. a 0.0..=1.0 utilization fraction).
    pub value: f64,
}

/// A bounded, FIFO history of timestamped metric samples.
///
/// Pushing past [`History::capacity`] samples evicts the oldest one first,
/// so memory usage never grows unbounded regardless of how long the
/// application runs.
#[derive(Debug, Clone)]
pub struct History {
    capacity: usize,
    samples: VecDeque<Sample>,
}

impl History {
    /// Creates an empty history bounded to `capacity` samples.
    ///
    /// `capacity` is clamped to at least `1`: a zero-capacity history
    /// could never hold anything useful, and clamping here means callers
    /// elsewhere never need to special-case it.
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            samples: VecDeque::new(),
        }
    }

    /// The maximum number of samples this history retains.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// The number of samples currently stored.
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    /// Whether this history currently holds no samples.
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// Records a new sample, evicting the oldest one first if the history
    /// is already at capacity.
    pub fn push(&mut self, timestamp: Instant, value: f64) {
        if self.samples.len() >= self.capacity {
            self.samples.pop_front();
        }
        self.samples.push_back(Sample { timestamp, value });
    }

    /// The most recently recorded sample, or `None` if the history is
    /// empty.
    pub fn latest(&self) -> Option<&Sample> {
        self.samples.back()
    }

    /// An iterator over every stored sample, oldest first.
    pub fn samples(&self) -> impl Iterator<Item = &Sample> {
        self.samples.iter()
    }

    /// Every stored sample's value, oldest first — a convenience for
    /// feeding a plotting widget that only cares about the values.
    pub fn values(&self) -> Vec<f64> {
        self.samples.iter().map(|sample| sample.value).collect()
    }
}

impl Default for History {
    /// Creates an empty history using [`DEFAULT_HISTORY_CAPACITY`].
    fn default() -> Self {
        Self::new(DEFAULT_HISTORY_CAPACITY)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn new_history_is_empty() {
        let history = History::new(10);
        assert!(history.is_empty());
        assert_eq!(history.len(), 0);
        assert_eq!(history.latest(), None);
        assert!(history.values().is_empty());
    }

    #[test]
    fn capacity_is_clamped_to_at_least_one() {
        let history = History::new(0);
        assert_eq!(history.capacity(), 1);
    }

    #[test]
    fn push_grows_up_to_capacity() {
        let mut history = History::new(3);
        let t0 = Instant::now();
        history.push(t0, 1.0);
        history.push(t0 + Duration::from_secs(1), 2.0);
        history.push(t0 + Duration::from_secs(2), 3.0);

        assert_eq!(history.len(), 3);
        assert_eq!(history.values(), vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn pushing_past_capacity_evicts_the_oldest_sample() {
        let mut history = History::new(3);
        let t0 = Instant::now();
        for i in 0..5 {
            history.push(t0 + Duration::from_secs(i), f64::from(i as u32));
        }

        assert_eq!(history.len(), 3);
        // The oldest two (0.0, 1.0) should have been evicted, leaving the
        // three most recent values in insertion order.
        assert_eq!(history.values(), vec![2.0, 3.0, 4.0]);
    }

    #[test]
    fn latest_reflects_the_most_recent_push() {
        let mut history = History::new(5);
        let t0 = Instant::now();
        history.push(t0, 10.0);
        history.push(t0 + Duration::from_secs(1), 20.0);

        assert_eq!(history.latest().unwrap().value, 20.0);
    }

    #[test]
    fn default_history_uses_the_documented_default_capacity() {
        let history = History::default();
        assert_eq!(history.capacity(), DEFAULT_HISTORY_CAPACITY);
        assert!(history.is_empty());
    }

    #[test]
    fn single_capacity_history_always_holds_only_the_latest_value() {
        let mut history = History::new(1);
        let t0 = Instant::now();
        history.push(t0, 1.0);
        history.push(t0 + Duration::from_secs(1), 2.0);

        assert_eq!(history.len(), 1);
        assert_eq!(history.values(), vec![2.0]);
    }
}
