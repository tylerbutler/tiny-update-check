//! Minimal binary for measuring size impact with rustls.
//!
//! Build with: cargo build --release --example size_check_rustls --no-default-features --features rustls
//! Check size: ls -lh target/release/examples/size_check_rustls

fn main() {
    match tiny_update_check::check("serde", "1.0.0") {
        Ok(Some(update)) => {
            eprintln!("Update available: {} -> {}", update.current, update.latest);
        }
        Ok(None) => eprintln!("Up to date"),
        Err(e) => eprintln!("Check failed: {e}"),
    }
}
