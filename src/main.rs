/// Estimates quantiles over a data stream.
#[derive(Debug, Clone)]
pub struct QuantileEstimator {
    val_count: usize,
    start: u64,
    end: u64,
    quantiles: Vec<usize>,
}

impl QuantileEstimator {
    /// Creates a new QuantileEstimator with the given start and end (inclusive).
    pub fn new(start: u64, end: u64) -> Self {
        let len = (end - start + 1) as usize;
        QuantileEstimator {
            val_count: 0,
            start,
            end,
            quantiles: vec![0; len],
        }
    }

    /// Adds a value to the estimator. Returns error if value is out of range.
    pub fn add_value(&mut self, value: u64) -> Result<(), &'static str> {
        if value < self.start || value > self.end {
            return Err("Value out of range");
        }
        self.val_count += 1;
        self.quantiles[(value - self.start) as usize] += 1;
        Ok(())
    }

    /// Returns the estimated quantile for a given fraction.
    pub fn estimate_quantile(&self, fraction: f64) -> Result<u64, &'static str> {
        if !(0.0..=1.0).contains(&fraction) {
            return Err("Fraction must be between 0 and 1");
        }
        if self.val_count == 0 {
            return Err("No values added to the estimator");
        }
        let mut index = (fraction * self.val_count as f64 - 1.0).round() as isize;
        if index < 0 {
            index = 0;
        }
        let mut cumulative = 0;
        for (i, &count) in self.quantiles.iter().enumerate() {
            cumulative += count;
            if cumulative > index as usize {
                return Ok(self.start + i as u64);
            }
        }
        Err("No quantile found for the given fraction")
    }
}

/// A ring buffer that stores QuantileEstimator instances for sliding window quantile estimation.
#[derive(Debug)]
pub struct TimeBasedRingBuffer {
    capacity: usize,
    duration: u64,
    windows: Vec<QuantileEstimator>,
    current: usize,
    start: u64,
    end: u64,
    current_window_start: u64,
    current_window_initialized: bool,
}

impl TimeBasedRingBuffer {
    /// Creates a new TimeBasedRingBuffer.
    pub fn new(capacity: usize, duration: u64, start: u64, end: u64) -> Self {
        let windows = vec![QuantileEstimator::new(start, end); capacity];
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
    pub fn insert(&mut self, value: u64, timestamp: u64) -> Result<(), &'static str> {
        if !self.current_window_initialized {
            if self.duration == 0 {
                return Err("Duration must be greater than zero");
            }
            self.current_window_start = timestamp - (timestamp % self.duration);
            self.current_window_initialized = true;
        }
        // Advance window(s) as needed
        while timestamp >= self.current_window_start + self.duration {
            self.current = (self.current + 1) % self.capacity;
            self.windows[self.current] = QuantileEstimator::new(self.start, self.end);
            self.current_window_start += self.duration;
        }
        self.windows[self.current].add_value(value)
    }

    /// Returns the quantile of all windows combined.
    pub fn estimate_quantile(&self, fraction: f64) -> Result<u64, &'static str> {
        if !(0.0..=1.0).contains(&fraction) {
            return Err("Fraction must be between 0 and 1");
        }
        if self.windows.is_empty() {
            return Err("No windows available in the ring buffer");
        }
        let total_val_count: usize = self.windows.iter().map(|w| w.val_count).sum();
        if total_val_count == 0 {
            return Err("No values added to any window");
        }
        let mut combined = vec![0; (self.end - self.start + 1) as usize];
        for window in &self.windows {
            for (i, &count) in window.quantiles.iter().enumerate() {
                combined[i] += count;
            }
        }
        let mut index = (fraction * total_val_count as f64 - 1.0).round() as isize;
        if index < 0 {
            index = 0;
        }
        let mut cumulative = 0;
        for (i, &count) in combined.iter().enumerate() {
            cumulative += count;
            if cumulative > index as usize {
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
        estimator.add_value(i).unwrap();
    }
    match estimator.estimate_quantile(0.5) {
        Ok(quantile) => println!("Estimated 50th percentile: {}", quantile),
        Err(e) => println!("Error estimating quantile: {}", e),
    }
    match estimator.estimate_quantile(0.99) {
        Ok(quantile) => println!("Estimated 99th percentile: {}", quantile),
        Err(e) => println!("Error estimating quantile: {}", e),
    }

    // Example usage of TimeBasedRingBuffer
    let mut ring_buffer = TimeBasedRingBuffer::new(3, 10, 0, 1000);
    for i in 0..11 {
        ring_buffer.insert(i, i * 2).unwrap();
    }
    ring_buffer.estimate_quantile(0.5)
        .map(|quantile| println!("Estimated 50th percentile from ring buffer: {}", quantile))
        .unwrap_or_else(|e| println!("Error estimating quantile from ring buffer: {}", e));
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_quantile_estimator() {
        let mut estimator = QuantileEstimator::new(0, 100);
        for i in 1..=100 {
            estimator.add_value(i).unwrap();
        }
        assert_eq!(estimator.estimate_quantile(0.5).unwrap(), 50);
        assert_eq!(estimator.estimate_quantile(0.9).unwrap(), 90);
        assert_eq!(estimator.estimate_quantile(0.99).unwrap(), 99);
        assert_eq!(estimator.estimate_quantile(0.0).unwrap(), 1);
        assert_eq!(estimator.estimate_quantile(1.0).unwrap(), 100);
        assert!(estimator.estimate_quantile(1.1).is_err());
        let empty_estimator = QuantileEstimator::new(0, 100);
        assert!(empty_estimator.estimate_quantile(0.5).is_err());
    }
    #[test]
    fn test_time_based_ring_buffer() {
        let mut ring_buffer = TimeBasedRingBuffer::new(3, 10, 0, 100);
        ring_buffer.insert(1, 0).unwrap();
        ring_buffer.insert(2, 5).unwrap();
        ring_buffer.insert(3, 5).unwrap();
        assert_eq!(ring_buffer.current, 0);
        ring_buffer.insert(3, 100).unwrap();
        assert_eq!(ring_buffer.current, 1);
    }
}