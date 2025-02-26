pub fn is_entry_signal(current_price: f64, sma_value: f64, ema_value: f64, entry_threshold: f64, trend_up: bool) -> bool {
    current_price > sma_value && current_price > ema_value && current_price > entry_threshold && !trend_up
}

pub fn is_exit_signal(current_price: f64, sma_value: f64, ema_value: f64, exit_threshold: f64, trend_up: bool) -> bool {
    current_price < sma_value && current_price < ema_value && current_price < exit_threshold && trend_up
}