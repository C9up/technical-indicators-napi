use napi_derive::napi;
use crate::low_high_open_close_volume_date_to_array_helper::process_market_data;

struct PivotLevels {
    pivot_point: f64,
    resistance1: f64,
    resistance2: f64,
    support1: f64,
    support2: f64,
}

impl PivotLevels {
    fn new(high: f64, low: f64, close: f64) -> Self {
        let pivot_point = (high + low + close) / 3.0;
        PivotLevels {
            pivot_point,
            resistance1: 2.0 * pivot_point - low,
            resistance2: pivot_point + (high - low),
            support1: 2.0 * pivot_point - high,
            support2: pivot_point - (high - low),
        }
    }
}

#[napi]
pub fn pivot_points(
    data: Vec<crate::MarketData>
) -> napi::Result<Vec<f64>> {

    let data = process_market_data(data);
    let highs = data.highs;
    let lows = data.lows;
    let closes = data.closes;

    if highs.len() != lows.len() || lows.len() != closes.len() {
        return Err(napi::Error::from_reason("Inconsistent data lengths between highs, lows and closes"));
    }

    let mut results = Vec::with_capacity(highs.len() * 5);

    for ((high, low), close) in highs.iter().zip(lows).zip(closes) {
        let levels = PivotLevels::new(*high, low, close); // Correction ici
        results.extend([
            levels.pivot_point,
            levels.resistance1,
            levels.resistance2,
            levels.support1,
            levels.support2,
        ]);
    }

    Ok(results)
}