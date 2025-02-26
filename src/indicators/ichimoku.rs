use napi_derive::napi;
use crate::helpers::MarketData;
use crate::low_high_open_close_volume_date_to_array_helper::process_market_data;

#[napi(object)]
pub struct IchimokuData {
    pub tenkan_sen: Option<f64>,
    pub kijun_sen: Option<f64>,
    pub senkou_span_a: Option<f64>,
    pub senkou_span_b: Option<f64>,
    pub chikou_span: Option<f64>,
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
    let chikou_shift = chikou_shift.unwrap_or(26).max(0) as usize;

    let n = data.highs.len();
    let highs = data.highs;
    let lows = data.lows;
    let closes = data.closes;

    let mut tenkan = vec![None; n];
    let mut kijun = vec![None; n];
    let mut senkou_a = vec![None; n];
    let mut senkou_b = vec![None; n];
    let mut chikou = vec![None; n];

    // Conversion des périodes en usize
    //let tenkan_period = tenkan_period.max(1) as usize;
    //let kijun_period = kijun_period.max(1) as usize;
    //let senkou_b_period = senkou_b_period.max(1) as usize;
    //let chikou_shift = chikou_shift as usize;
    let senkou_shift = kijun_period;

    // Calcul des composants Tenkan et Kijun
    for i in 0..n {
        // Tenkan-sen
        if i >= tenkan_period - 1 {
            let start = i - (tenkan_period - 1);
            let max_high = highs[start..=i].iter().fold(f64::MIN, |a, &b| a.max(b));
            let min_low = lows[start..=i].iter().fold(f64::MAX, |a, &b| a.min(b));
            tenkan[i] = Some((max_high + min_low) / 2.0);
        }

        // Kijun-sen
        if i >= kijun_period - 1 {
            let start = i - (kijun_period - 1);
            let max_high = highs[start..=i].iter().fold(f64::MIN, |a, &b| a.max(b));
            let min_low = lows[start..=i].iter().fold(f64::MAX, |a, &b| a.min(b));
            kijun[i] = Some((max_high + min_low) / 2.0);
        }
    }

    // Pré-calcul pour Senkou Span B
    let mut max_high_senkou_b = vec![None; n];
    let mut min_low_senkou_b = vec![None; n];
    for i in 0..n {
        if i >= senkou_b_period - 1 {
            let start = i - (senkou_b_period - 1);
            let max_h = highs[start..=i].iter().fold(f64::MIN, |a, &b| a.max(b));
            let min_l = lows[start..=i].iter().fold(f64::MAX, |a, &b| a.min(b));
            max_high_senkou_b[i] = Some(max_h);
            min_low_senkou_b[i] = Some(min_l);
        }
    }

    // Calcul des spans et Chikou
    for j in 0..n {
        // Senkou Span A (décalage de kijun_period)
        if j >= senkou_shift {
            let i = j - senkou_shift;
            if i >= tenkan_period - 1 && i >= kijun_period - 1 {
                if let (Some(t), Some(k)) = (tenkan[i], kijun[i]) {
                    senkou_a[j] = Some((t + k) / 2.0);
                }
            }
        }

        // Senkou Span B (décalage de kijun_period)
        if j >= senkou_shift {
            let i = j - senkou_shift;
            if i >= senkou_b_period - 1 {
                if let (Some(max_h), Some(min_l)) = (max_high_senkou_b[i], min_low_senkou_b[i]) {
                    senkou_b[j] = Some((max_h + min_l) / 2.0);
                }
            }
        }

        // Chikou Span
        if j + chikou_shift < n {
            chikou[j] = Some(closes[j + chikou_shift]);
        }
    }

    // Construction des résultats
    let mut result = Vec::with_capacity(n);
    for i in 0..n {
        result.push(IchimokuData {
            tenkan_sen: tenkan[i].or(Some(f64::NAN)),
            kijun_sen: kijun[i].or(Some(f64::NAN)),
            senkou_span_a: senkou_a[i].or(Some(f64::NAN)),
            senkou_span_b: senkou_b[i].or(Some(f64::NAN)),
            chikou_span: chikou[i].or(Some(f64::NAN)),
        });
    }

    result
}