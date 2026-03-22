use napi::bindgen_prelude::*;
use napi_derive::napi;
use crate::helpers::low_high_open_close_volume_date_to_array_helper::process_market_data;

#[napi(object)]
pub struct AwesomeOscillatorResult {
    /// AO values (SMA5 - SMA34 of midpoints)
    pub ao: Vec<f64>,
    /// AO histogram color: 1 = green (rising), -1 = red (falling), 0 = neutral
    pub histogram: Vec<i32>,
}

/// Awesome Oscillator (Bill Williams)
///
/// AO = SMA(5, Midpoint) - SMA(34, Midpoint)
/// where Midpoint = (High + Low) / 2
///
/// Measures market momentum. Histogram bars are green when AO is rising,
/// red when falling. Zero-line crossovers signal trend changes.
///
/// Parameters:
/// - data: OHLCV market data
/// - fast_period: fast SMA period (default: 5)
/// - slow_period: slow SMA period (default: 34)
#[napi]
pub fn awesome_oscillator(
    data: Vec<crate::MarketData>,
    fast_period: Option<u32>,
    slow_period: Option<u32>,
) -> Result<AwesomeOscillatorResult> {
    let market = process_market_data(data);
    let highs = &market.highs;
    let lows = &market.lows;
    let n = highs.len();

    let fast = fast_period.unwrap_or(5) as usize;
    let slow = slow_period.unwrap_or(34) as usize;

    if fast >= slow {
        return Err(Error::from_reason("Fast period must be less than slow period"));
    }
    if n < slow {
        return Err(Error::from_reason("Not enough data for the given periods"));
    }

    // Midpoints
    let midpoints: Vec<f64> = highs.iter().zip(lows.iter()).map(|(h, l)| (h + l) / 2.0).collect();

    // SMA of midpoints
    let sma_fast = rolling_sma(&midpoints, fast);
    let sma_slow = rolling_sma(&midpoints, slow);

    let mut ao = vec![f64::NAN; n];
    let mut histogram = vec![0i32; n];

    for i in (slow - 1)..n {
        if !sma_fast[i].is_nan() && !sma_slow[i].is_nan() {
            ao[i] = sma_fast[i] - sma_slow[i];

            if i > slow - 1 && !ao[i - 1].is_nan() {
                histogram[i] = if ao[i] > ao[i - 1] { 1 } else if ao[i] < ao[i - 1] { -1 } else { 0 };
            }
        }
    }

    Ok(AwesomeOscillatorResult { ao, histogram })
}

fn rolling_sma(data: &[f64], period: usize) -> Vec<f64> {
    let n = data.len();
    let mut result = vec![f64::NAN; n];

    if n < period {
        return result;
    }

    let mut sum: f64 = data[..period].iter().sum();
    result[period - 1] = sum / period as f64;

    for i in period..n {
        sum += data[i] - data[i - period];
        result[i] = sum / period as f64;
    }

    result
}
