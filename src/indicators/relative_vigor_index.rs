use napi::bindgen_prelude::*;
use napi_derive::napi;
use crate::helpers::low_high_open_close_volume_date_to_array_helper::process_market_data;

#[napi(object)]
pub struct RviResult {
    /// RVI line values
    pub rvi: Vec<f64>,
    /// Signal line (4-period weighted moving average of RVI)
    pub signal: Vec<f64>,
}

/// Relative Vigor Index (RVI)
///
/// Measures the conviction of a price move by comparing the close-open range
/// to the high-low range. The idea: in uptrends, closes tend to be above opens,
/// and the opposite in downtrends.
///
/// RVI = SMA(N, numerator) / SMA(N, denominator)
/// where:
///   numerator = (Close - Open) + 2*(Close[-1] - Open[-1]) + 2*(Close[-2] - Open[-2]) + (Close[-3] - Open[-3]) / 6
///   denominator = (High - Low) + 2*(High[-1] - Low[-1]) + 2*(High[-2] - Low[-2]) + (High[-3] - Low[-3]) / 6
///
/// Signal = (RVI + 2*RVI[-1] + 2*RVI[-2] + RVI[-3]) / 6
///
/// Parameters:
/// - data: OHLCV market data
/// - period: SMA smoothing period (default: 10)
#[napi]
pub fn relative_vigor_index(
    data: Vec<crate::MarketData>,
    period: Option<u32>,
) -> Result<RviResult> {
    let market = process_market_data(data);
    let opens = &market.opens;
    let highs = &market.highs;
    let lows = &market.lows;
    let closes = &market.closes;
    let n = closes.len();

    let period = period.unwrap_or(10) as usize;

    if n < period + 4 {
        return Err(Error::from_reason("Not enough data for the given period"));
    }

    // Weighted close-open and high-low (4-bar symmetric weighting: 1,2,2,1 / 6)
    let mut num = vec![0.0; n];
    let mut den = vec![0.0; n];

    for i in 3..n {
        num[i] = ((closes[i] - opens[i])
            + 2.0 * (closes[i - 1] - opens[i - 1])
            + 2.0 * (closes[i - 2] - opens[i - 2])
            + (closes[i - 3] - opens[i - 3]))
            / 6.0;

        den[i] = ((highs[i] - lows[i])
            + 2.0 * (highs[i - 1] - lows[i - 1])
            + 2.0 * (highs[i - 2] - lows[i - 2])
            + (highs[i - 3] - lows[i - 3]))
            / 6.0;
    }

    // SMA of numerator and denominator
    let mut rvi = vec![f64::NAN; n];

    for i in (period + 2)..n {
        let sum_num: f64 = num[(i - period + 1)..=i].iter().sum();
        let sum_den: f64 = den[(i - period + 1)..=i].iter().sum();

        rvi[i] = if sum_den.abs() > 1e-15 {
            sum_num / sum_den
        } else {
            0.0
        };
    }

    // Signal line: 4-bar symmetric weighted MA of RVI (1,2,2,1 / 6)
    let mut signal = vec![f64::NAN; n];

    for i in (period + 5)..n {
        if !rvi[i].is_nan() && !rvi[i - 1].is_nan() && !rvi[i - 2].is_nan() && !rvi[i - 3].is_nan() {
            signal[i] = (rvi[i] + 2.0 * rvi[i - 1] + 2.0 * rvi[i - 2] + rvi[i - 3]) / 6.0;
        }
    }

    Ok(RviResult { rvi, signal })
}
