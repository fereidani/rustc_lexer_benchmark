# rustc_lexer Benchmark

This repository is designed to benchmark and track performance improvements across different `rustc_lexer` implementations.

## Project Structure

- **`src_new/`** — Contains the new implementation (default)
- **`src_orig/`** — Contains the original implementation (enabled via feature flag)

## Usage

To compare performance between implementations:

1. Benchmark the original implementation:

   ```sh
   cargo bench --features orig
   ```

2. Benchmark the new implementation:
   ```sh
   cargo bench
   ```

Compare the results to measure performance improvements.
