use napi::bindgen_prelude::*;
use napi_derive::napi;
use crate::helpers::low_high_open_close_volume_date_to_array_helper::process_market_data;

#[napi(object)]
pub struct ChoppinessResult {
    /// Choppiness Index values (0-100). NaN for warmup period.
    pub chop: Vec<f64>,
    /// Signals: 1 = trending (CHOP crosses below low_threshold),
    /// -1 = choppy/ranging (CHOP crosses above high_threshold), 0 = neutral
    pub signals: Vec<i32>,
}

/// Choppiness Index (CI)
///
/// CI = 100 * log10(Sum(TR, N) / (HighestHigh_N - LowestLow_N)) / log10(N)
///
/// Measures whether the market is trending or range-bound:
/// - Low values (< 38.2) indicate a strong trend
/// - High values (> 61.8) indicate a choppy/sideways market
///
/// Parameters:
/// - data: OHLCV market data
/// - period: lookback period (default: 14)
/// - low_threshold: below this = trending (default: 38.2)
/// - high_threshold: above this = choppy (default: 61.8)
#[napi]
pub fn choppiness_index(
    data: Vec<crate::MarketData>,
    period: Option<u32>,
    low_threshold: Option<f64>,
    high_threshold: Option<f64>,
) -> Result<ChoppinessResult> {
    let market = process_market_data(data);
    let highs = &market.highs;
    let lows = &market.lows;
    let closes = &market.closes;
    let n = closes.len();

    let period = period.unwrap_or(14) as usize;
    let low_thresh = low_threshold.unwrap_or(38.2);
    let high_thresh = high_threshold.unwrap_or(61.8);

    if period < 2 {
        return Err(Error::from_reason("Period must be at least 2"));
    }
    if n < period + 1 {
        return Err(Error::from_reason("Not enough data for the given period"));
    }

    // True Range
    let mut tr = vec![0.0; n];
    for i in 1..n {
        tr[i] = (highs[i] - lows[i])
            .max((highs[i] - closes[i - 1]).abs())
            .max((lows[i] - closes[i - 1]).abs());
    }

    let log_n = (period as f64).log10();
    let mut chop = vec![f64::NAN; n];
    let mut signals = vec![0i32; n];

    for i in period..n {
        // Sum of TR over period
        let sum_tr: f64 = tr[(i - period + 1)..=i].iter().sum();

        // Highest high and lowest low over period
        let hh = highs[(i - period + 1)..=i]
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max);
        let ll = lows[(i - period + 1)..=i]
            .iter()
            .cloned()
            .fold(f64::INFINITY, f64::min);

        let range = hh - ll;

        let ci = if range > 0.0 && log_n > 0.0 {
            (100.0 * (sum_tr / range).log10() / log_n).clamp(0.0, 100.0)
        } else {
            50.0 // neutral when range is zero
        };

        chop[i] = ci;

        // Signal on threshold crossover
        if i > period && !chop[i - 1].is_nan() {
            if ci < low_thresh && chop[i - 1] >= low_thresh {
                signals[i] = 1; // trending signal
            } else if ci > high_thresh && chop[i - 1] <= high_thresh {
                signals[i] = -1; // choppy signal
            }
        }
    }

    Ok(ChoppinessResult { chop, signals })
}
