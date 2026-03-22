use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi(object)]
pub struct ConditionalProbabilityResult {
    /// Probability of a move >= +second_threshold after a first move >= first_threshold
    pub up_probability: f64,
    /// Probability of a move <= -second_threshold after a first move >= first_threshold
    pub down_probability: f64,
    /// Number of times the first move condition was met
    pub first_move_count: i32,
    /// Number of times the second move was up after first move
    pub up_count: i32,
    /// Number of times the second move was down after first move
    pub down_count: i32,
    /// Indices where up moves occurred (in original data)
    pub up_indices: Vec<i32>,
    /// Indices where down moves occurred (in original data)
    pub down_indices: Vec<i32>,
    /// All second move percentage changes (when first condition was met)
    pub second_move_returns: Vec<f64>,
}

/// Conditional Probability Analysis
///
/// Calculates: P(second_move >= threshold | first_move >= threshold)
///
/// Given a price series, finds all instances where the price moved by at least
/// `first_threshold` over `first_move_days`, then measures what happened over
/// the following `second_move_days`.
///
/// The first move is triggered by absolute change >= first_threshold (both up and down).
/// The second move probabilities are split into up (>= second_threshold) and down (<= -second_threshold).
#[napi]
pub fn conditional_probability(
    prices: Vec<f64>,
    first_move_days: u32,
    second_move_days: u32,
    first_threshold: f64,
    second_threshold: f64,
) -> Result<ConditionalProbabilityResult> {
    let first_days = first_move_days as usize;
    let second_days = second_move_days as usize;

    if prices.is_empty() {
        return Err(Error::from_reason("Prices array cannot be empty"));
    }
    if first_days == 0 || second_days == 0 {
        return Err(Error::from_reason("Move days must be greater than 0"));
    }
    if first_threshold <= 0.0 || second_threshold <= 0.0 {
        return Err(Error::from_reason("Thresholds must be greater than 0"));
    }
    if prices.len() < first_days + second_days + 1 {
        return Err(Error::from_reason("Not enough data for the given parameters"));
    }

    let mut first_move_count = 0i32;
    let mut up_count = 0i32;
    let mut down_count = 0i32;
    let mut up_indices = Vec::new();
    let mut down_indices = Vec::new();
    let mut second_move_returns = Vec::new();

    // For each bar, check if the first move condition is met
    for i in first_days..prices.len() {
        let first_pct = (prices[i] - prices[i - first_days]) / prices[i - first_days];

        // First move must exceed threshold (absolute value)
        if first_pct.abs() < first_threshold {
            continue;
        }

        first_move_count += 1;

        // Check if we have enough data for the second move
        let second_end = i + second_days;
        if second_end >= prices.len() {
            continue;
        }

        let second_pct = (prices[second_end] - prices[i]) / prices[i];
        second_move_returns.push(second_pct);

        if second_pct >= second_threshold {
            up_count += 1;
            up_indices.push(i as i32);
        } else if second_pct <= -second_threshold {
            down_count += 1;
            down_indices.push(i as i32);
        }
    }

    let up_probability = if first_move_count > 0 {
        up_count as f64 / first_move_count as f64
    } else {
        0.0
    };

    let down_probability = if first_move_count > 0 {
        down_count as f64 / first_move_count as f64
    } else {
        0.0
    };

    Ok(ConditionalProbabilityResult {
        up_probability,
        down_probability,
        first_move_count,
        up_count,
        down_count,
        up_indices,
        down_indices,
        second_move_returns,
    })
}

#[napi(object)]
pub struct ConditionalMatrixEntry {
    pub first_threshold: f64,
    pub second_threshold: f64,
    pub up_probability: f64,
    pub down_probability: f64,
    pub sample_count: i32,
}

/// Compute a matrix of conditional probabilities across multiple threshold combinations.
///
/// Useful for heatmap visualization: for each (first_threshold, second_threshold) pair,
/// returns the up and down probabilities.
#[napi]
pub fn conditional_probability_matrix(
    prices: Vec<f64>,
    first_move_days: u32,
    second_move_days: u32,
    first_thresholds: Vec<f64>,
    second_thresholds: Vec<f64>,
) -> Result<Vec<ConditionalMatrixEntry>> {
    if prices.is_empty() {
        return Err(Error::from_reason("Prices array cannot be empty"));
    }

    let first_days = first_move_days as usize;
    let second_days = second_move_days as usize;

    if first_days == 0 || second_days == 0 {
        return Err(Error::from_reason("Move days must be greater than 0"));
    }

    // Precompute all first and second moves
    let len = prices.len();
    let mut first_moves: Vec<(usize, f64)> = Vec::new();

    for i in first_days..len {
        let pct = (prices[i] - prices[i - first_days]) / prices[i - first_days];
        first_moves.push((i, pct));
    }

    let mut results = Vec::with_capacity(first_thresholds.len() * second_thresholds.len());

    for &ft in &first_thresholds {
        for &st in &second_thresholds {
            let mut count = 0i32;
            let mut up = 0i32;
            let mut down = 0i32;

            for &(i, first_pct) in &first_moves {
                if first_pct.abs() < ft {
                    continue;
                }
                count += 1;

                let second_end = i + second_days;
                if second_end >= len {
                    continue;
                }

                let second_pct = (prices[second_end] - prices[i]) / prices[i];
                if second_pct >= st {
                    up += 1;
                } else if second_pct <= -st {
                    down += 1;
                }
            }

            results.push(ConditionalMatrixEntry {
                first_threshold: ft,
                second_threshold: st,
                up_probability: if count > 0 { up as f64 / count as f64 } else { 0.0 },
                down_probability: if count > 0 { down as f64 / count as f64 } else { 0.0 },
                sample_count: count,
            });
        }
    }

    Ok(results)
}
