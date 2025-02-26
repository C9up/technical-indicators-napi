use napi_derive::napi;
pub mod low_high_open_close_volume_date_to_array_helper;
pub mod compute_bollinger_bands;
pub mod calculate_ema_helper;
pub mod calculate_atr_helper;
pub mod calculate_sma_helper;
pub mod entry_exit_signals_helper;
pub mod relative_strength_index_helper;
pub mod highest_lowest_helper;


#[napi(object)]
#[derive(Clone)]
pub struct MarketData {
    pub low: f64,
    pub high: f64,
    pub open: f64,
    pub close: f64,
    pub volume: f64,
    pub date: String,
}

#[napi(object)]
pub struct MarketDataResult {
    pub lows: Vec<f64>,
    pub highs: Vec<f64>,
    pub opens: Vec<f64>,
    pub closes: Vec<f64>,
    pub volumes: Vec<f64>,
    pub dates: Vec<String>,
}