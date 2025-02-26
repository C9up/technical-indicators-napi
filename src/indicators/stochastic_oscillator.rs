use napi::{bindgen_prelude::*};
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
    let mut result = Vec::with_capacity(closes.len().saturating_sub(period));

    for i in period..closes.len() {
        let (highest_high, lowest_low) = calculate_high_low(&highs, &lows, i - period, i - 1);
        let stoch = 100.0 * (closes[i] - lowest_low) / (highest_high - lowest_low);
        result.push(stoch);
    }

    Ok(result)
}