use napi::bindgen_prelude::*;
use napi_derive::napi;

/// Disparity Index
///
/// Measures the percentage distance between the current price and a moving average.
/// DI = 100 * (Close - MA(N)) / MA(N)
///
/// Positive values: price is above the MA (bullish)
/// Negative values: price is below the MA (bearish)
/// Extreme values suggest overbought/oversold conditions
#[napi]
pub fn disparity_index(
    prices: Vec<f64>,
    period: Option<u32>,
) -> Result<Vec<f64>> {
    let period = period.unwrap_or(14) as usize;

    if period < 1 {
        return Err(Error::from_reason("Period must be at least 1"));
    }
    if prices.len() < period {
        return Err(Error::from_reason("Not enough data for the given period"));
    }

    let n = prices.len();
    let mut result = vec![f64::NAN; n];

    // Rolling SMA
    let mut sum: f64 = prices[..period].iter().sum();

    for i in (period - 1)..n {
        if i >= period {
            sum += prices[i] - prices[i - period];
        }
        let ma = sum / period as f64;
        result[i] = if ma != 0.0 {
            100.0 * (prices[i] - ma) / ma
        } else {
            0.0
        };
    }

    Ok(result)
}
