# 1brc in Rust

Dependency free Rust implementation of 1brc. One quirk of Rust is that rounding to one decimal place rounds down so had to do some custom multiply divide mathemagic.

First did a basic implementation without multithreading and then added threads since this is a classic MapReduce problem which can be effectively parallelized.

## Results
| What    | How slow |
| -------- | ------- |
| Baseline (Java)  | 163.625 s ±  1.434 s |
| Rust without multithreading | 230.619 s ±  2.856 s |
| Rust with 500 threads   | 720.46s |

Benchmarked using Hyperfine on my Mac M3 Pro with 36 GB RAM.


### How to run
* Generate `measurements.txt` 
* [Install Rust](https://www.rust-lang.org/tools/install)

### Release run
* `cargo build --release && ./target/release/calculate_average_pranavsb`

### Debugging
* `cargo build && ./target/debug/calculate_average_pranavsb`
* debug build prints some data to stdout and validates that the final output is correct

### Testing
* To run a particular test from `test/resources/samples`:
    * `cargo build && ./target/debug/calculate_average_pranavsb ../../test/resources/samples/measurements-3.txt`
    * All tests are passing. There's a 0.1 difference with some values in the 1b file which I'm tryng to figure out (doing some custom rounding which is the likely culprit)

### Benchmarks

#### Basic benchmarking with `time`
On my Mac M3 Pro with 36 GB RAM, my naive implementation (without multithreading) is slower than the baseline by a minute:
* Mine (Rust) - 

```bash
time ./calculate_average/calculate_average_pranavsb.sh

./calculate_average/calculate_average_pranavsb.sh  227.86s user 3.14s system 98% cpu 3:53.91 total
```
* Baseline (Java) - 
```bash
time ./calculate_average/calculate_average_baseline.sh

./calculate_average/calculate_average_baseline.sh  157.12s user 4.78s system 100% cpu 2:41.56 total
```

#### Using `hyperfine`
* On Mac, `brew install hyperfine`
* `hyperfine "./calculate_average/calculate_average_pranavsb.sh"`


Note that there's still some rounding off-by-one for some values in my code but it shouldn't affect the benchmarks.