//! Integration tests for the public `Collector` API.
//!
//! These tests run against the real host, so they assert invariants that
//! hold on any machine rather than exact CPU or process-count values, which
//! are inherently environment-dependent.

use std::thread::sleep;
use std::time::Duration;

use ferrite::collector::Collector;

#[test]
fn snapshot_reports_nonzero_total_memory() {
    let mut collector = Collector::new();
    let snapshot = collector.refresh();
    assert!(snapshot.memory().total_bytes() > 0);
}

#[test]
fn used_memory_never_exceeds_total_memory() {
    let mut collector = Collector::new();
    let snapshot = collector.refresh();
    assert!(snapshot.memory().used_bytes() <= snapshot.memory().total_bytes());
}

#[test]
fn process_list_contains_this_test_process() {
    let mut collector = Collector::new();
    let snapshot = collector.refresh();
    let this_pid = std::process::id();
    assert!(
        snapshot
            .processes()
            .iter()
            .any(|process| process.pid == this_pid)
    );
}

#[test]
fn process_pids_are_unique() {
    let mut collector = Collector::new();
    let snapshot = collector.refresh();

    let mut pids: Vec<u32> = snapshot
        .processes()
        .iter()
        .map(|process| process.pid)
        .collect();
    let count_before_dedup = pids.len();
    pids.sort_unstable();
    pids.dedup();

    assert_eq!(pids.len(), count_before_dedup);
}

#[test]
fn process_list_is_sorted_by_cpu_usage_with_pid_tie_break() {
    let mut collector = Collector::new();
    let snapshot = collector.refresh();

    for pair in snapshot.processes().windows(2) {
        let (first, second) = (&pair[0], &pair[1]);
        match first
            .cpu_usage_percent
            .partial_cmp(&second.cpu_usage_percent)
        {
            Some(std::cmp::Ordering::Less) => panic!(
                "process {} ({} %) sorted before process {} ({} %), expected descending order",
                first.pid, first.cpu_usage_percent, second.pid, second.cpu_usage_percent
            ),
            Some(std::cmp::Ordering::Equal) | None => {
                assert!(
                    first.pid < second.pid,
                    "equal (or NaN) CPU usage must be tie-broken by ascending PID"
                );
            }
            Some(std::cmp::Ordering::Greater) => {}
        }
    }
}

#[test]
fn cpu_metrics_become_reliable_after_waiting_the_minimum_interval() {
    let mut collector = Collector::new();
    sleep(Duration::from_millis(300));
    let snapshot = collector.refresh();
    assert!(snapshot.cpu_metrics_initialized());
}

#[test]
fn every_disk_reports_used_bytes_within_its_total() {
    let mut collector = Collector::new();
    let snapshot = collector.refresh();

    for disk in snapshot.disks() {
        assert!(
            disk.used_bytes() <= disk.total_bytes(),
            "disk {:?} at {:?} reported used_bytes > total_bytes",
            disk.name(),
            disk.mount_point()
        );
    }
}

#[test]
fn disk_list_does_not_panic_when_empty_or_populated() {
    let mut collector = Collector::new();
    let snapshot = collector.refresh();
    // No assertion on the count (host-dependent); just confirms collection
    // and iteration complete without panicking either way.
    let _ = snapshot.disks().len();
}

#[test]
fn first_network_sample_reports_no_rates() {
    let mut collector = Collector::new();
    let snapshot = collector.refresh();

    for network in snapshot.networks() {
        assert_eq!(
            network.received_bytes_per_sec(),
            None,
            "interface {:?} should have no rate on the very first sample",
            network.name()
        );
        assert_eq!(network.transmitted_bytes_per_sec(), None);
    }
}

#[test]
fn second_network_sample_can_report_real_rates_without_panicking() {
    let mut collector = Collector::new();
    collector.refresh();
    sleep(Duration::from_millis(50));
    let snapshot = collector.refresh();

    // Rates are host-dependent, so we only assert that reading them (an
    // `Option<u64>`, never negative by construction) does not panic.
    for network in snapshot.networks() {
        let _ = network.received_bytes_per_sec();
        let _ = network.transmitted_bytes_per_sec();
    }
}
