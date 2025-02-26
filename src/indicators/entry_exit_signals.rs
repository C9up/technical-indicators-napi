use napi_derive::napi;
use crate::helpers::calculate_atr_helper::calculate_atr;
use crate::helpers::calculate_ema_helper::calculate_ema;
use crate::helpers::calculate_sma_helper::calculate_sma;
use crate::helpers::entry_exit_signals_helper::{is_entry_signal, is_exit_signal};

#[napi(object)]
pub struct Signal {
    #[napi(js_name = "type")]
    pub kind: i32,
    pub price: f64,
    pub index: i32,
}

#[napi]
pub fn entry_exit_signals(
    data: Vec<f64>,
    sma_period: i32,
    ema_period: i32,
    atr_period: i32,
    threshold: f64,
) -> Vec<Signal> {
    // Convert periods to usize for index calculations
    let sma_period_usize = sma_period as usize;
    let ema_period_usize = ema_period as usize;
    let atr_period_usize = atr_period as usize;

    // Calculate required minimum length
    let required_min_len = *[
        sma_period_usize,
        ema_period_usize,
        atr_period_usize + 1,
    ].iter().max().unwrap();

    if data.len() < required_min_len {
        return Vec::new();
    }

    // Handle Result types from indicator calculations
    let sma_values = match calculate_sma(&data, sma_period) {
        Ok(values) => values,
        Err(_) => return Vec::new(),
    };

    let ema_values = match calculate_ema(&data, ema_period) {
        Ok(values) => values,
        Err(_) => return Vec::new(),
    };

    let atr_values = match calculate_atr(&data, atr_period) {
        Ok(values) => values,
        Err(_) => return Vec::new(),
    };

    // Determine start index using usize values
    let start_index = *[
        sma_period_usize - 1,
        ema_period_usize - 1,
        atr_period_usize,
    ].iter().max().unwrap();

    let mut signals = Vec::new();
    let mut trend_up = false;

    for i in start_index..data.len() {
        let current_sma_index = i - (sma_period_usize - 1);
        let current_ema_index = i - (ema_period_usize - 1);
        let current_atr_index = i - atr_period_usize;

        // Handle potential index out-of-bounds cases
        if current_sma_index >= sma_values.len()
            || current_ema_index >= ema_values.len()
            || current_atr_index >= atr_values.len()
        {
            continue;
        }

        let current_price = data[i];
        let current_sma = sma_values[current_sma_index];
        let current_ema = ema_values[current_ema_index];
        let current_atr = atr_values[current_atr_index];

        let entry_threshold = current_sma + current_atr * threshold;
        let exit_threshold = current_sma - current_atr * threshold;

        if is_entry_signal(current_price, current_sma, current_ema, entry_threshold, trend_up) {
            signals.push(Signal {
                kind: 0,
                price: current_price,
                index: i as i32,
            });
            trend_up = true;
        } else if is_exit_signal(current_price, current_sma, current_ema, exit_threshold, trend_up) {
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