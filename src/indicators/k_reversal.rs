use napi::bindgen_prelude::*;
use napi_derive::napi;
use crate::highest_lowest_helper::calculate_high_low;
use crate::low_high_open_close_volume_date_to_array_helper::process_market_data;

#[napi(object)]
pub struct KReversalResult {
    pub k_values: Vec<f64>,
    pub buy_signals: Vec<KReversalSignal>,
    pub sell_signals: Vec<KReversalSignal>,
}

#[napi(object)]
pub struct KReversalSignal {
    pub index: i32,
    pub price: f64,
    pub k_value: f64,
}

/// K-Reversal Indicator
///
/// K = 100 * (Close - Low_N) / (High_N - Low_N)
///
/// K < buy_threshold (default 20) suggests potential uptrend (oversold)
/// K > sell_threshold (default 80) suggests potential downtrend (overbought)
#[napi]
pub fn k_reversal(
    data: Vec<crate::MarketData>,
    period: Option<u32>,
    buy_threshold: Option<f64>,
    sell_threshold: Option<f64>,
) -> Result<KReversalResult> {
    let data = process_market_data(data);
    let highs = data.highs;
    let lows = data.lows;
    let closes = data.closes;

    let period = period.unwrap_or(14) as usize;
    let buy_threshold = buy_threshold.unwrap_or(20.0);
    let sell_threshold = sell_threshold.unwrap_or(80.0);

    if period == 0 {
        return Err(Error::from_reason("Period must be greater than 0"));
    }

    if closes.len() < period {
        return Err(Error::from_reason("Not enough data for the given period"));
    }

    let mut k_values = vec![f64::NAN; period - 1];
    let mut buy_signals = Vec::new();
    let mut sell_signals = Vec::new();

    // K = 100 * (Close - Low_N) / (High_N - Low_N)
    // Window: [i-period+1, i] (includes current bar)
    #[allow(clippy::needless_range_loop)]
    for i in (period - 1)..closes.len() {
        let start = i - (period - 1);
        let (highest_high, lowest_low) = calculate_high_low(&highs, &lows, start, i);
        let range = highest_high - lowest_low;

        let k = if range == 0.0 { 50.0 } else { 100.0 * (closes[i] - lowest_low) / range };
        k_values.push(k);

        if k < buy_threshold {
            buy_signals.push(KReversalSignal {
                index: i as i32,
                price: closes[i],
                k_value: k,
            });
        } else if k > sell_threshold {
            sell_signals.push(KReversalSignal {
                index: i as i32,
                price: closes[i],
                k_value: k,
            });
        }
    }

    Ok(KReversalResult {
        k_values,
        buy_signals,
        sell_signals,
    })
}
