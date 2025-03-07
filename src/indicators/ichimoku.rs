use napi_derive::napi;
use crate::helpers::MarketData;
use crate::highest_lowest_helper::calculate_high_low;
use crate::low_high_open_close_volume_date_to_array_helper::process_market_data;

#[napi(object)]
pub struct IchimokuData {
    pub tenkan_sen: f64,
    pub kijun_sen: f64,
    pub senkou_span_a: f64,
    pub senkou_span_b: f64,
    pub chikou_span: f64,
}

#[napi]
pub fn ichimoku(
    data: Vec<MarketData>,
    #[napi(ts_arg_type = "number", default = 9)] tenkan_period: Option<u32>,
    #[napi(ts_arg_type = "number", default = 26)] kijun_period: Option<u32>,
    #[napi(ts_arg_type = "number", default = 52)] senkou_b_period: Option<u32>,
    #[napi(ts_arg_type = "number", default = 26)] chikou_shift: Option<u32>,
) -> Vec<IchimokuData> {

    let data = process_market_data(data);
    let tenkan_p = tenkan_period.unwrap_or(9).max(1) as usize;
    let kijun_p = kijun_period.unwrap_or(26).max(1) as usize;
    let senkou_b_p = senkou_b_period.unwrap_or(52).max(1) as usize;
    let chikou_shift = chikou_shift.unwrap_or(26).max(1) as usize;

    let n = data.highs.len();
    let highs = data.highs;
    let lows = data.lows;
    let closes = data.closes;

    // Initialisation des tableaux
    let mut tenkan = vec![f64::NAN; n];
    let mut kijun = vec![f64::NAN; n];
    let mut senkou_a = vec![f64::NAN; n];
    let mut senkou_b = vec![f64::NAN; n];
    let mut chikou = vec![f64::NAN; n]; // Chikou corrigé

    for i in 0..n {
        if i >= tenkan_p - 1 {
            let start = i - (tenkan_p - 1);
            let (h, l) = calculate_high_low(&highs, &lows, start, i);
            tenkan[i] = (h + l) / 2.0;
        }

        if i >= kijun_p - 1 {
            let start = i - (kijun_p - 1);
            let (h, l) = calculate_high_low(&highs, &lows, start, i);
            kijun[i] = (h + l) / 2.0;
        }
    }

    let mut senkou_b_buffer = vec![f64::NAN; n];
    for i in 0..n {
        if i >= senkou_b_p - 1 {
            let start = i - (senkou_b_p - 1);
            let (h, l) = calculate_high_low(&highs, &lows, start, i);
            senkou_b_buffer[i] = (h + l) / 2.0;
        }
    }

    let senkou_shift = kijun_p;
    for i in 0..n {
        if i >= senkou_shift {
            let base_idx = i - senkou_shift;

            // Senkou A
            if !tenkan[base_idx].is_nan() && !kijun[base_idx].is_nan() {
                senkou_a[i] = (tenkan[base_idx] + kijun[base_idx]) / 2.0;
            }

            // Senkou B
            if !senkou_b_buffer[base_idx].is_nan() {
                senkou_b[i] = senkou_b_buffer[base_idx];
            }
        }
    }

    for i in 0..n {
        if i >= chikou_shift {
            chikou[i] = closes[i - chikou_shift];
        }
    }

    (0..n)
        .map(|i| IchimokuData {
            tenkan_sen: tenkan[i],
            kijun_sen: kijun[i],
            senkou_span_a: senkou_a[i],
            senkou_span_b: senkou_b[i],
            chikou_span: chikou[i],
        })
        .collect()
}