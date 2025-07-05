use std::cmp;

/// This struct estimates quantiles over a data stream.
struct QuantileEstimator {
    // Number of values processed
    val_count: u64,
    // Start of the range
    start: u64,
    // End of the range
    end: u64,
    // Stored quantile values
    quantiles: Vec<u64>,
}


/// QuantileEstimator implementation
impl QuantileEstimator {
    /// Creates a new QuantileEstimator with the given start, end, and quantiles.
    pub fn new(start: u64, end: u64) -> Self {
        QuantileEstimator {
            val_count: 0,
            start: start,
            end: end,
            quantiles: vec![0; (end-start + 1) as usize],
        }
    }

    /// Adds a value to the estimator.
    pub fn add_value(&mut self, value: u64) {
        if value < self.start || value > self.end {
            panic!("Value out of range");
        }
        self.val_count += 1;
        self.quantiles[(value - self.start) as usize] += 1;
    }

    /// Returns the estimated quantile for a given fraction.
    /// Returns `Ok(u64)` if a quantile is found, or `Err(&str)` if not.
    pub fn estimate_quantile(&self, fraction: f64) -> Result<u64, &'static str> {
        if fraction < 0.0 || fraction > 1.0 {
            return Err("Fraction must be between 0 and 1");
        }
        if self.val_count == 0 {
            return Err("No values added to the estimator");
        }
        // Get the index corresponding to the fraction, make sure it has the correct upper bound
        let mut index = (fraction * self.val_count as f64 - 1.0).round() as usize;
        if index >= self.quantiles.len() {
            index = self.quantiles.len() - 1; // Ensure index is within bounds
        }
        // Iterate through the quantiles to find the value at the index
        let mut cumulative_count: u64 = 0;
        for (i, &count) in self.quantiles.iter().enumerate() {
            cumulative_count += count;
            if cumulative_count > index as u64 {
                return Ok(self.start + i as u64);
            }
        }
        Err("No quantile found for the given fraction")
    }
}

/// This struct is a ring buffer that stores QuantileEstimator instances.
/// It is used to track a sliding window of quantile estimations.
/// The TimeBasedRingBuffer struct defines its capacity (# of windows), the duration each window covers,
/// and maintains a current index for ring buffer operations.
struct TimeBasedRingBuffer {
    capacity: usize,
    duration: u64,
    windows: Vec<QuantileEstimator>,
    current: usize, // Points to the current (oldest) window in the ring buffer
    start: u64,     // Range start for QuantileEstimator
    end: u64,       // Range end for QuantileEstimator
    current_window_start: u64, // Start timestamp of the current window
    current_window_initialized: bool, // Flag to check if the current window has been initialized
}

/// TimeBasedRingBuffer implementation
impl TimeBasedRingBuffer {
    /// Creates a new TimeBasedRingBuffer with the given capacity, duration, and quantile range.
    pub fn new(capacity: usize, duration: u64, start: u64, end: u64) -> Self {
        let mut windows = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            windows.push(QuantileEstimator::new(start, end));
        }
        TimeBasedRingBuffer {
            capacity,
            duration,
            windows,
            current: 0,
            start,
            end,
            current_window_start: 0,
            current_window_initialized: false,
        }
    }

    /// Inserts a value with a timestamp into the appropriate window.
    /// If the timestamp is outside the current window, advances the window and resets the estimator.
    pub fn insert(&mut self, value: u64, timestamp: u64) {
        // If this is the first insert, initialize the window start
        if self.current_window_initialized == false {
            if self.duration == 0 {
                panic!("Duration must be greater than zero");
            }
            self.current_window_start = timestamp - (timestamp % self.duration);
            self.current_window_initialized = true;
        }
        // Advance window(s) as needed if timestamp is outside the current window
        while timestamp > self.current_window_start + self.duration {
            self.current = (self.current + 1) % self.capacity;
            self.windows[self.current] = QuantileEstimator::new(self.start, self.end);
            self.current_window_start += cmp::max(timestamp, self.duration);
        }
        // Insert into the current window
        self.windows[self.current].add_value(value);
    }

    /// Returns the quantile of all windows combined.
    /// We sum all quantiles vectors to make a new vector.
    /// Returns `Ok(u64)` if a quantile is found, or `Err(&str)` if not.
    pub fn estimate_quantile(&self, fraction: f64) -> Result<u64, &'static str> {
        if fraction < 0.0 || fraction > 1.0 {
            return Err("Fraction must be between 0 and 1");
        }
        if self.windows.is_empty() {
            return Err("No windows available in the ring buffer");
        }
        // sum val_count from all windows's QuantileEstimators
        let total_val_count: u64 = self.windows.iter().map(|w| w.val_count).sum();
        if total_val_count == 0 {
            return Err("No values added to any window");
        }
        // Create a new quantiles vector to hold the combined quantiles
        let mut combined_quantiles = vec![0; (self.end - self.start + 1) as usize];
        // Sum the quantiles from all windows
        for window in &self.windows {
            for (i, &count) in window.quantiles.iter().enumerate() {
                combined_quantiles[i] += count;
            }
        }
        // Get the index corresponding to the fraction, make sure it has the correct upper bound
        let mut index = (fraction * total_val_count as f64 - 1.0).round() as usize;
        if index >= combined_quantiles.len() {
            index = combined_quantiles.len() - 1; // Ensure index is within bounds
        }
        // Iterate through the combined quantiles to find the value at the index
        let mut cumulative_count: u64 = 0;
        for (i, &count) in combined_quantiles.iter().enumerate() {
            cumulative_count += count;
            if cumulative_count > index as u64 {
                return Ok(self.start + i as u64);
            }
        }
        Err("No quantile found for the given fraction")
    }
}

fn main() {
    // Example usage of QuantileEstimator
    let mut estimator = QuantileEstimator::new(0, 1000);
    for i in 0..=101 {
        estimator.add_value(i);
    }
    // Estimate the 50th percentile (median)
    match estimator.estimate_quantile(0.5) {
        Ok(quantile) => println!("Estimated 50th percentile: {}", quantile),
        Err(e) => println!("Error estimating quantile: {}", e),
    }

    // Estimate the 99th percentile
    match estimator.estimate_quantile(0.99) {
        Ok(quantile) => println!("Estimated 99th percentile: {}", quantile),
        Err(e) => println!("Error estimating quantile: {}", e),
    }

    // Example usage of TimeBasedRingBuffer
    let mut ring_buffer = TimeBasedRingBuffer::new(3, 10, 0, 1000);
    // Insert some values with timestamps
    for i in 0..11 {
        ring_buffer.insert(i, i * 2); // Insert values with even timestamps
    }
    ring_buffer.estimate_quantile(0.5)
        .map(|quantile| println!("Estimated 50th percentile from ring buffer: {}", quantile))
        .unwrap_or_else(|e| println!("Error estimating quantile from ring buffer: {}", e));
}


// Write a test for the QuantileEstimator
#[cfg(test)]
mod tests {
    use super::*; // Import the QuantileEstimator
    #[test]
    fn test_quantile_estimator() {
        let mut estimator = QuantileEstimator::new(0, 100);
        // Add some values to the estimator
        for i in 1..101 {
            estimator.add_value(i);
        }
        // Test the 50th percentile (median)
        assert_eq!(estimator.estimate_quantile(0.5).unwrap(), 50);
        // Test the 90th percentile
        assert_eq!(estimator.estimate_quantile(0.9).unwrap(), 90);
        // Test the 99th percentile
        assert_eq!(estimator.estimate_quantile(0.99).unwrap(), 99);
        // Test the 0th percentile (minimum)
        assert_eq!(estimator.estimate_quantile(0.0).unwrap(), 1);
        // Test the 100th percentile (maximum)
        assert_eq!(estimator.estimate_quantile(1.0).unwrap(), 100);
        // Test an out-of-range fraction
        assert!(estimator.estimate_quantile(1.1).is_err());
        // Test an empty estimator
        let empty_estimator = QuantileEstimator::new(0, 100);
        assert!(empty_estimator.estimate_quantile(0.5).is_err());
    }
    #[test]
    fn test_time_based_ring_buffer() {
        let mut ring_buffer = TimeBasedRingBuffer::new(3, 10, 0, 100);
        // Insert values into the ring buffer
        ring_buffer.insert(1, 0);
        ring_buffer.insert(2, 5);
        ring_buffer.insert(3, 5);
        // Check the current window's quantiles
        assert_eq!(ring_buffer.current, 0);
        // Force a window change by inserting a value with a timestamp that exceeds the current window duration
        ring_buffer.insert(3, 100);
        assert_eq!(ring_buffer.current, 1);
    }
}