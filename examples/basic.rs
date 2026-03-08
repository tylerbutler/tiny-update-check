//! Basic usage example for tiny-update-check.
//!
//! Run with: `cargo run --example basic`

use std::time::Duration;
use tiny_update_check::UpdateChecker;

fn main() {
    // Simple one-liner using the convenience function
    if let Ok(Some(update)) = tiny_update_check::check("serde", "1.0.0") {
        eprintln!("Update available: {} -> {}", update.current, update.latest);
    }

    // Builder pattern with configuration
    let checker = UpdateChecker::new("serde", "1.0.0")
        .cache_duration(Duration::from_secs(60 * 60)) // 1 hour
        .timeout(Duration::from_secs(10));

    match checker.check() {
        Ok(Some(update)) => {
            eprintln!(
                "New version {} available! (you have {})",
                update.latest, update.current
            );
        }
        Ok(None) => {
            eprintln!("Already on the latest version.");
        }
        Err(e) => {
            // Silently ignore errors in production — don't disrupt the user's workflow
            eprintln!("Update check failed: {e}");
        }
    }
}
