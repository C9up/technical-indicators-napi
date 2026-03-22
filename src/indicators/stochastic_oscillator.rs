use napi::bindgen_prelude::*;
use napi_derive::napi;
use crate::highest_lowest_helper::calculate_high_low;
use crate::low_high_open_close_volume_date_to_array_helper::process_market_data;

#[napi]
pub fn stochastic_oscillator(
    data: Vec<crate::MarketData>,
    period: u32,
) -> Result<Vec<f64>> {
    let data = process_market_data(data);
    let highs = data.highs;
    let lows = data.lows;
    let closes = data.closes;

    let period = period as usize;

    if period == 0 {
        return Err(Error::from_reason("Period must be greater than 0"));
    }

    if closes.len() < period {
        return Err(Error::from_reason("Not enough data for the given period"));
    }

    let mut result = Vec::with_capacity(closes.len() - period + 1);

    // Window includes current bar: [i-period+1, i]
    #[allow(clippy::needless_range_loop)]
    for i in (period - 1)..closes.len() {
        let start = i - (period - 1);
        let (highest_high, lowest_low) = calculate_high_low(&highs, &lows, start, i);
        let range = highest_high - lowest_low;
        let stoch = if range == 0.0 { 50.0 } else { 100.0 * (closes[i] - lowest_low) / range };
        result.push(stoch);
    }

    Ok(result)
}
