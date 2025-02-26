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
    let tenkan_period = tenkan_period.unwrap_or(9).max(1) as usize;
    let kijun_period = kijun_period.unwrap_or(26).max(1) as usize;
    let senkou_b_period = senkou_b_period.unwrap_or(52).max(1) as usize;
    let chikou_shift = chikou_shift.unwrap_or(26).max(1) as usize;

    let n = data.highs.len();
    let highs = data.highs;
    let lows = data.lows;
    let closes = data.closes;

    let mut tenkan_sen = vec![f64::NAN; n];
    let mut kijun_sen = vec![f64::NAN; n];
    let mut senkou_span_a = vec![f64::NAN; n];
    let mut senkou_span_b = vec![f64::NAN; n];
    let mut chikou_span = vec![f64::NAN; n];

    // Calcul des composants de base
    for i in 0..n {
        // Tenkan-sen
        if i >= tenkan_period - 1 {
            let start = i - (tenkan_period - 1);
            let (high, low) = calculate_high_low(&highs, &lows, start, i);
            tenkan_sen[i] = (high + low) / 2.0;
        }

        // Kijun-sen
        if i >= kijun_period - 1 {
            let start = i - (kijun_period - 1);
            let (high, low) = calculate_high_low(&highs, &lows, start, i);
            kijun_sen[i] = (high + low) / 2.0;
        }

        // Chikou Span
        if i + chikou_shift < n {
            chikou_span[i] = closes[i + chikou_shift];
        }
    }

    // Pré-calcul pour Senkou Span B
    let mut senkou_b_buffer = vec![f64::NAN; n];
    for i in 0..n {
        if i >= senkou_b_period - 1 {
            let start = i - (senkou_b_period - 1);
            let (high, low) = calculate_high_low(&highs, &lows, start, i);
            senkou_b_buffer[i] = (high + low) / 2.0;
        }
    }

    // Calcul des spans avec décalage
    let senkou_shift = kijun_period;
    for i in 0..n {
        if i >= senkou_shift {
            let base_idx = i - senkou_shift;

            // Senkou Span A
            if !tenkan_sen[base_idx].is_nan() && !kijun_sen[base_idx].is_nan() {
                senkou_span_a[i] = (tenkan_sen[base_idx] + kijun_sen[base_idx]) / 2.0;
            }

            // Senkou Span B
            if !senkou_b_buffer[base_idx].is_nan() {
                senkou_span_b[i] = senkou_b_buffer[base_idx];
            }
        }
    }

    // Construction des résultats
    (0..n)
        .map(|i| IchimokuData {
            tenkan_sen: tenkan_sen[i],
            kijun_sen: kijun_sen[i],
            senkou_span_a: senkou_span_a[i],
            senkou_span_b: senkou_span_b[i],
            chikou_span: chikou_span[i],
        })
        .collect()
}