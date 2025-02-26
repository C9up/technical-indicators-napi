use napi_derive::napi;
use napi::Result;
use crate::calculate_ema_helper::calculate_ema;
use crate::highest_lowest_helper::calculate_high_low;
use crate::low_high_open_close_volume_date_to_array_helper::process_market_data;

#[napi]
pub fn stochastic_momentum_index(
    data: Vec<crate::MarketData>,
    period_k: Option<u32>,
    period_d: Option<u32>
) -> Result<Vec<f64>> {

    let data = process_market_data(data);

    // Validate input data consistency
    let n = data.highs.len();
    if data.lows.len() != n || data.closes.len() != n {
        return Err(napi::Error::from_reason("Highs, lows and closes arrays must have the same length".to_string()));
    }

    let lookback = period_k.unwrap_or(14) as usize;
    let smoothing = period_d.unwrap_or(3) as usize;

    // Handle insufficient data case
    if n < lookback {
        return Ok(vec![f64::NAN; n]);
    }

    // Calculate HH/LL windows and build D/R arrays
    let valid_len = n - lookback + 1;
    let mut diff = Vec::with_capacity(valid_len);
    let mut range = Vec::with_capacity(valid_len);

    for i in (lookback - 1)..n {
        let start = i - lookback + 1;
        let (hh, ll) = calculate_high_low(&data.highs, &data.lows, start, i);
        let midpoint = (hh + ll) / 2.0;
        diff.push(data.closes[i] - midpoint);
        range.push(hh - ll);
    }

    // Double EMA smoothing avec propagation des erreurs
    let ema_diff1 = calculate_ema(&diff, lookback as i32)?;
    let ema_diff2 = calculate_ema(&ema_diff1, smoothing as i32)?;
    let ema_range1 = calculate_ema(&range, lookback as i32)?;
    let ema_range2 = calculate_ema(&ema_range1, smoothing as i32)?;

    // Build final result with padding and SMI values
    let mut smi = vec![f64::NAN; lookback - 1];
    for i in 0..ema_diff2.len() {
        smi.push(100.0 * (ema_diff2[i] / ema_range2[i]));
    }

    Ok(smi)
}