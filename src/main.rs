//! Ferrite executable entry point.

use ferrite::config::AppConfig;

fn main() {
    let config = AppConfig::default();

    println!("Ferrite {} (early development)", env!("CARGO_PKG_VERSION"));
    println!("A terminal-native system monitor — currently a project foundation.");
    println!(
        "CPU, memory, and process collection exist in the library, but this executable \
         does not yet display them; there is no dashboard, alerting, or history yet."
    );
    println!(
        "Configured refresh interval: {} ms",
        config.refresh_interval_ms()
    );
}
