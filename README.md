# Quantile Estimator

A simple Rust program for estimating quantiles over a data stream, with support for sliding window quantile estimation using a time-based ring buffer.

*I am learning some Rust with Copilot's help*. 😎

## Features

- **QuantileEstimator**: Tracks quantiles for integer values within a specified range.
- **TimeBasedRingBuffer**: Maintains multiple quantile estimators in a ring buffer for sliding window quantile calculations.

## Usage

Add the source files to your Rust project.

### Example: Basic Quantile Estimation

```rust
let mut estimator = QuantileEstimator::new(0, 1000);
for i in 0..=101 {
    estimator.add_value(i).unwrap();
}
let median = estimator.estimate_quantile(0.5).unwrap();
println!("Estimated 50th percentile: {}", median);
```

### Example: Sliding Window Quantile Estimation

```rust
let mut ring_buffer = TimeBasedRingBuffer::new(3, 10, 0, 1000);
for i in 0..15 {
    ring_buffer.insert(i, i * 2).unwrap();
}
let quantile = ring_buffer.estimate_quantile(0.5).unwrap();
println!("Estimated 50th percentile from ring buffer: {}", quantile);
```

## API

### QuantileEstimator

- `QuantileEstimator::new(start: u64, end: u64) -> Self`
- `add_value(&mut self, value: u64) -> Result<(), &'static str>`
- `estimate_quantile(&self, fraction: f64) -> Result<u64, &'static str>`

### TimeBasedRingBuffer

- `TimeBasedRingBuffer::new(capacity: usize, duration: u64, start: u64, end: u64) -> Self`
- `insert(&mut self, value: u64, timestamp: u64) -> Result<(), &'static str>`
- `estimate_quantile(&self, fraction: f64) -> Result<u64, &'static str>`

## Testing

Run the included tests with:

```sh
cargo test
```