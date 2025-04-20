# 1brc in Rust

### How to run
* Generate `measurements.txt` 
* [Install Rust](https://www.rust-lang.org/tools/install)

### Release run
* `cargo build --release && ./target/release/calculate_average_pranavsb`

### Debugging
* `cargo build && ./target/debug/calculate_average_pranavsb`

### Testing
* to run a particular test from `test/resources/samples`:
    * `cargo build && ./target/debug/calculate_average_pranavsb ../../test/resources/samples/measurements-3.txt`

### Benchmarks