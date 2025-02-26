/// Calculate the highest high and lowest low in a given range of data.
///
/// # Arguments
/// * `high_prices` - A slice of high prices.
/// * `low_prices` - A slice of low prices.
/// * `start` - The starting index (inclusive).
/// * `end` - The ending index (inclusive).
///
/// # Returns
/// A tuple `(highest_high, lowest_low)`.
pub fn calculate_high_low(high_prices: &[f64], low_prices: &[f64], start: usize, end: usize) -> (f64, f64) {
    high_prices[start..=end]
        .iter()
        .zip(low_prices[start..=end].iter())
        .fold((f64::MIN, f64::MAX), |(mut max_high, mut min_low), (&high, &low)| {
            if high > max_high {
                max_high = high;
            }
            if low < min_low {
                min_low = low;
            }
            (max_high, min_low)
        })
}