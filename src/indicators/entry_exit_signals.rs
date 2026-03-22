use napi_derive::napi;
use crate::helpers::calculate_atr_helper::calculate_atr;
use crate::helpers::calculate_ema_helper::calculate_ema;
use crate::helpers::calculate_sma_helper::calculate_sma;
use crate::helpers::entry_exit_signals_helper::{is_entry_signal, is_exit_signal};
use crate::helpers::low_high_open_close_volume_date_to_array_helper::process_market_data;

#[napi(object)]
pub struct Signal {
    #[napi(js_name = "type")]
    pub kind: i32,
    pub price: f64,
    pub index: i32,
}

#[napi]
pub fn entry_exit_signals(
    data: Vec<crate::MarketData>,
    sma_period: i32,
    ema_period: i32,
    atr_period: i32,
    threshold: f64,
) -> Vec<Signal> {
    let market_data = process_market_data(data);
    let closes = &market_data.closes;
    let highs = &market_data.highs;
    let lows = &market_data.lows;

    let sma_period_usize = sma_period as usize;
    let ema_period_usize = ema_period as usize;
    let atr_period_usize = atr_period as usize;

    let required_min_len = *[
        sma_period_usize,
        ema_period_usize,
        atr_period_usize + 1,
    ]
    .iter()
    .max()
    .expect("static array always has elements");

    if closes.len() < required_min_len {
        return Vec::new();
    }

    // SMA: returns array of len = closes.len(), first sma_period-1 values are NaN
    // sma_values[i] corresponds to closes[i]
    let sma_values = match calculate_sma(closes, sma_period) {
        Ok(values) => values,
        Err(_) => return Vec::new(),
    };

    // EMA: returns array of len = closes.len() - ema_period + 1
    // ema_values[0] corresponds to closes[ema_period - 1]
    let ema_values = match calculate_ema(closes, ema_period) {
        Ok(values) => values,
        Err(_) => return Vec::new(),
    };

    // ATR: returns array of len = closes.len() - atr_period
    // atr_values[0] corresponds to candle index atr_period
    let atr_values = match calculate_atr(highs, lows, closes, atr_period_usize) {
        Ok(values) => values,
        Err(_) => return Vec::new(),
    };

    // Start where all indicators have valid values
    let start_index = *[
        sma_period_usize - 1,
        ema_period_usize - 1,
        atr_period_usize,
    ]
    .iter()
    .max()
    .expect("static array always has elements");

    let mut signals = Vec::new();
    let mut trend_up = false;

    #[allow(clippy::needless_range_loop)]
    for i in start_index..closes.len() {
        // SMA is aligned with closes: sma_values[i] = SMA at bar i
        let current_sma = sma_values[i];
        if current_sma.is_nan() {
            continue;
        }

        // EMA starts at index ema_period-1 of closes, so ema_values index = i - (ema_period - 1)
        let ema_idx = i - (ema_period_usize - 1);
        if ema_idx >= ema_values.len() {
            continue;
        }
        let current_ema = ema_values[ema_idx];

        // ATR starts at index atr_period of closes, so atr_values index = i - atr_period
        let atr_idx = i - atr_period_usize;
        if atr_idx >= atr_values.len() {
            continue;
        }
        let current_atr = atr_values[atr_idx];

        let current_price = closes[i];
        let entry_threshold = current_sma + current_atr * threshold;
        let exit_threshold = current_sma - current_atr * threshold;

        if is_entry_signal(current_price, current_sma, current_ema, entry_threshold, trend_up) {
            signals.push(Signal {
                kind: 0,
                price: current_price,
                index: i as i32,
            });
            trend_up = true;
        } else if is_exit_signal(current_price, current_sma, current_ema, exit_threshold, trend_up)
        {
            signals.push(Signal {
                kind: 1,
                price: current_price,
                index: i as i32,
            });
            trend_up = false;
        }
    }

    signals
}
