use napi_derive::napi;
use crate::helpers::low_high_open_close_volume_date_to_array_helper::process_market_data;
use crate::helpers::MarketData;

#[napi(object)]
pub struct DMIResult {
    pub plus_di: Vec<f64>,
    pub minus_di: Vec<f64>,
    pub adx: Vec<f64>,
}

#[napi]
pub fn directional_movement_index(
    data: Vec<MarketData>,
    period: i32
) -> napi::Result<DMIResult> {
    if period <= 0 {
        return Err(napi::Error::from_reason("Period must be greater than 0."));
    }

    let data = process_market_data(data);
    let period = period as usize;

    let highs = data.highs;
    let lows = data.lows;
    let closes = data.closes;

    let len = highs.len();
    if len < period * 2 {
        return Err(napi::Error::from_reason(format!("Not enough data points. Need at least {}", period * 2)));
    }

    let mut tr = vec![0.0; len];
    let mut plus_dm = vec![0.0; len];
    let mut minus_dm = vec![0.0; len];
    let mut plus_di = vec![0.0; len];
    let mut minus_di = vec![0.0; len];
    let mut dx = vec![0.0; len];
    let mut adx = vec![0.0; len];

    for i in 1..len {
        let high = highs[i];
        let low = lows[i];
        let prev_high = highs[i - 1];
        let prev_low = lows[i - 1];
        let prev_close = closes[i - 1];

        // True Range
        tr[i] = (high - low)
            .max((high - prev_close).abs())
            .max((low - prev_close).abs());

        // Directional Movement
        let up_move = high - prev_high;
        let down_move = prev_low - low;

        plus_dm[i] = if up_move > down_move && up_move > 0.0 {
            up_move
        } else {
            0.0
        };

        minus_dm[i] = if down_move > up_move && down_move > 0.0 {
            down_move
        } else {
            0.0
        };
    }


    let mut tr_sum: f64 = tr[1..=period].iter().sum();
    let mut plus_dm_sum: f64 = plus_dm[1..=period].iter().sum();
    let mut minus_dm_sum: f64 = minus_dm[1..=period].iter().sum();

    plus_di[period] = (plus_dm_sum / tr_sum) * 100.0;
    minus_di[period] = (minus_dm_sum / tr_sum) * 100.0;

    for i in (period + 1)..len {
        tr_sum = tr_sum - tr_sum / period as f64 + tr[i];
        plus_dm_sum = plus_dm_sum - plus_dm_sum / period as f64 + plus_dm[i];
        minus_dm_sum = minus_dm_sum - minus_dm_sum / period as f64 + minus_dm[i];

        plus_di[i] = (plus_dm_sum / tr_sum) * 100.0;
        minus_di[i] = (minus_dm_sum / tr_sum) * 100.0;
    }

    for i in period..len {
        let di_diff = (plus_di[i] - minus_di[i]).abs();
        let di_sum = plus_di[i] + minus_di[i];
        dx[i] = if di_sum > 0.0 { (di_diff / di_sum) * 100.0 } else { 0.0 };
    }

    let adx_start = period * 2;
    let mut adx_sum: f64 = dx[period..adx_start].iter().sum();
    adx[adx_start - 1] = adx_sum / period as f64;

    for i in adx_start..len {
        adx_sum = adx_sum - adx[i - period] + dx[i];
        adx[i] = adx_sum / period as f64;
    }

    Ok(DMIResult {
        plus_di,
        minus_di,
        adx,
    })
}