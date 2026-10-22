//! Ferrite executable entry point.

use ferrite::config::AppConfig;

fn main() {
    let config = AppConfig::default();

    println!("Ferrite {} (early development)", env!("CARGO_PKG_VERSION"));
    println!("A terminal-native system monitor — currently a project foundation.");
    println!("Resource metrics, process inspection, and alerts are not yet implemented.");
    println!(
        "Configured refresh interval: {} ms",
        config.refresh_interval_ms()
    );
}
