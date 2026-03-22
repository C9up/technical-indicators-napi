use napi::bindgen_prelude::*;
use napi_derive::napi;
use crate::helpers::low_high_open_close_volume_date_to_array_helper::process_market_data;

#[napi(object)]
pub struct YangZhangResult {
    /// Yang-Zhang volatility (annualized)
    pub volatility: Vec<f64>,
    /// Overnight component (close-to-open)
    pub overnight_vol: Vec<f64>,
    /// Intraday component (open-to-close)
    pub intraday_vol: Vec<f64>,
    /// Rogers-Satchell component
    pub rogers_satchell: Vec<f64>,
}

/// Yang-Zhang Volatility Estimator
///
/// Combines three volatility components for more accurate estimation than
/// simple standard deviation:
/// - Overnight volatility: log(Open / prev_Close)²
/// - Intraday volatility: log(Close / Open)²
/// - Rogers-Satchell: log(H/O)*log(H/C) + log(L/O)*log(L/C)
///
/// YZ = sqrt(overnight + k * intraday + (1-k) * RS)
/// where k = 0.34 / (1.34 + (n+1)/(n-1))
///
/// Output is annualized (multiplied by sqrt(252)).
///
/// Parameters:
/// - data: OHLCV market data
/// - window: rolling window (default: 10)
#[napi]
pub fn yang_zhang_volatility(
    data: Vec<crate::MarketData>,
    window: Option<u32>,
) -> Result<YangZhangResult> {
    let market = process_market_data(data);
    let opens = &market.opens;
    let highs = &market.highs;
    let lows = &market.lows;
    let closes = &market.closes;
    let n = opens.len();

    let w = window.unwrap_or(10).max(2) as usize;

    if n < w + 1 {
        return Err(Error::from_reason("Not enough data for the given window"));
    }

    // Log returns
    let mut log_co = vec![f64::NAN; n]; // close-to-open (overnight)
    let mut log_oc = vec![0.0; n];      // open-to-close (intraday)
    let mut log_ho = vec![0.0; n];      // high-to-open
    let mut log_lo = vec![0.0; n];      // low-to-open
    let mut log_hc = vec![0.0; n];      // high-to-close
    let mut log_lc = vec![0.0; n];      // low-to-close

    for i in 0..n {
        if opens[i] > 0.0 {
            log_oc[i] = (closes[i] / opens[i]).ln();
            log_ho[i] = (highs[i] / opens[i]).ln();
            log_lo[i] = (lows[i] / opens[i]).ln();
        }
        if closes[i] > 0.0 {
            log_hc[i] = (highs[i] / closes[i]).ln();
            log_lc[i] = (lows[i] / closes[i]).ln();
        }
        if i > 0 && opens[i] > 0.0 && closes[i - 1] > 0.0 {
            log_co[i] = (opens[i] / closes[i - 1]).ln();
        }
    }

    // Rogers-Satchell per bar: log(H/O)*log(H/C) + log(L/O)*log(L/C)
    let mut rs_bar = vec![0.0; n];
    for i in 0..n {
        rs_bar[i] = log_ho[i] * log_hc[i] + log_lo[i] * log_lc[i];
    }

    // Yang-Zhang k factor
    let k = 0.34 / (1.34 + (w as f64 + 1.0) / (w as f64 - 1.0));
    let sqrt252 = 252.0_f64.sqrt();

    let mut volatility = vec![f64::NAN; n];
    let mut overnight_vol = vec![f64::NAN; n];
    let mut intraday_vol = vec![f64::NAN; n];
    let mut rogers_satchell = vec![f64::NAN; n];

    for i in w..n {
        let start = i - w + 1;

        // Rolling means of squared components
        let mut sum_co2 = 0.0;
        let mut sum_oc2 = 0.0;
        let mut sum_rs = 0.0;
        let mut count = 0.0;

        for j in start..=i {
            if !log_co[j].is_nan() {
                sum_co2 += log_co[j] * log_co[j];
                sum_oc2 += log_oc[j] * log_oc[j];
                sum_rs += rs_bar[j];
                count += 1.0;
            }
        }

        if count < 2.0 {
            continue;
        }

        let open_var = sum_co2 / count;
        let close_var = sum_oc2 / count;
        let rs_var = sum_rs / count;

        overnight_vol[i] = (open_var * 252.0).sqrt();
        intraday_vol[i] = (close_var * 252.0).sqrt();
        rogers_satchell[i] = (rs_var.abs() * 252.0).sqrt();

        let yz_var = open_var + k * close_var + (1.0 - k) * rs_var;
        volatility[i] = if yz_var > 0.0 { yz_var.sqrt() * sqrt252 } else { 0.0 };
    }

    Ok(YangZhangResult {
        volatility,
        overnight_vol,
        intraday_vol,
        rogers_satchell,
    })
}
