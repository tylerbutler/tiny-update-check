//! Async usage example for tiny-update-check.
//!
//! Run with: `cargo run --example async_usage --features async`

use tiny_update_check::r#async::UpdateChecker;

#[tokio::main]
async fn main() {
    let checker = UpdateChecker::new("serde", "1.0.0");
    match checker.check().await {
        Ok(Some(update)) => {
            eprintln!("Update available: {} -> {}", update.current, update.latest);
        }
        Ok(None) => {
            eprintln!("Already on the latest version.");
        }
        Err(e) => {
            eprintln!("Update check failed: {e}");
        }
    }
}
