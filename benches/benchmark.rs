//! Performance benchmarks
//!
//! Run with: `cargo bench`
//!
//! Requires the `criterion` dev-dependency to be uncommented in Cargo.toml.

// Uncomment below when ready to add benchmarks:
//
// use criterion::{black_box, criterion_group, criterion_main, Criterion};
//
// fn example_benchmark(c: &mut Criterion) {
//     c.bench_function("example", |b| {
//         b.iter(|| {
//             // Replace with actual code to benchmark
//             black_box(42)
//         })
//     });
// }
//
// criterion_group!(benches, example_benchmark);
// criterion_main!(benches);

fn main() {
    println!("Benchmarks not yet configured.");
    println!("To enable:");
    println!("  1. Add criterion to [dev-dependencies] in Cargo.toml:");
    println!("     criterion = {{ version = \"0.5\", features = [\"html_reports\"] }}");
    println!("  2. Add [[bench]] section to Cargo.toml:");
    println!("     [[bench]]");
    println!("     name = \"benchmark\"");
    println!("     harness = false");
    println!("  3. Uncomment the benchmark code in this file");
}
