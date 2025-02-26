use napi::bindgen_prelude::*;
use napi_derive::napi;
use crate::calculate_ema_helper::calculate_ema;
use crate::low_high_open_close_volume_date_to_array_helper::process_market_data;

pub fn true_range(high: &[f64], low: &[f64], close: &[f64], index: usize) -> f64 {
    let tr1 = high[index] - low[index];
    let tr2 = (high[index] - close[index - 1]).abs();
    let tr3 = (low[index] - close[index - 1]).abs();
    tr1.max(tr2).max(tr3)
}

#[napi]
pub fn trends_meter(
    data: Vec<crate::MarketData>,
    period: Option<u32>
) -> Result<Vec<f64>> {
    let period = period.unwrap_or(14) as usize;
    let data = process_market_data(data);

    if period < 2 {
        return Err(Error::from_reason("Period must be greater than 1"));
    }

    let highs = data.highs;
    let lows = data.lows;
    let closes = data.closes;

    if closes.len() < period {
        return Err(Error::from_reason("Insufficient data for period"));
    }

    // Calcul du True Range
    let mut tr = vec![0.0; closes.len()];
    for i in 1..closes.len() {
        tr[i] = true_range(&highs, &lows, &closes, i);
    }

    // Calcul EMA pour TR
    let tr_ema = calculate_ema(&tr, period as i32)?;

    // Calcul du Momentum
    let mut momentum = vec![0.0; closes.len()];
    for i in period..closes.len() {
        momentum[i] = closes[i] - closes[i - period];
    }

    // Calcul EMA pour Momentum
    let momentum_ema = calculate_ema(&momentum, period as i32)?;

    // Calcul final du Trends Meter
    let mut trends_meter = vec![0.0; closes.len()];
    for i in period..closes.len() {
        trends_meter[i] = (tr_ema[i] + momentum_ema[i]) / 2.0;
    }

    Ok(trends_meter)
}