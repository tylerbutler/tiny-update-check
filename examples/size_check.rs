//! Minimal binary for measuring size impact.
//!
//! Build with: cargo build --release --example size_check
//! Check size: ls -lh target/release/examples/size_check

fn main() {
    match tiny_update_check::check("serde", "1.0.0") {
        Ok(Some(update)) => {
            eprintln!("Update available: {} -> {}", update.current, update.latest);
        }
        Ok(None) => eprintln!("Up to date"),
        Err(e) => eprintln!("Check failed: {e}"),
    }
}
