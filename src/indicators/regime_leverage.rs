use napi::bindgen_prelude::*;
use napi_derive::napi;
use crate::helpers::low_high_open_close_volume_date_to_array_helper::process_market_data;

#[napi(object)]
pub struct RegimeLeverageResult {
    /// Hybrid oscillator values (smoothed)
    pub oscillator: Vec<f64>,
    /// Yang-Zhang volatility (annualized)
    pub yz_volatility: Vec<f64>,
    /// Volatility percentile (0-1, rolling 252-bar)
    pub vol_percentile: Vec<f64>,
    /// Regime label: 0=Defensive, 1=Moderate, 2=Bullish, 3=Aggressive
    pub regime: Vec<i32>,
    /// Leverage factor: 0.0, 1.0, 2.0, or 3.0
    pub leverage: Vec<f64>,
    /// VIX ratio (vix/vix3m) if VIX data provided, else NaN
    pub vix_ratio: Vec<f64>,
}

/// Market Regime Adaptive Leverage System (MRALS)
///
/// Classifies market into 4 regimes and assigns leverage:
/// - Aggressive (3x): low vol + bullish trend + normal VIX structure
/// - Bullish (2x): positive trend, moderate volatility
/// - Moderate (1x): neutral conditions
/// - Defensive (0x): high volatility or bearish signals
///
/// Uses a hybrid oscillator combining:
/// - Price momentum (EMA fast/slow differential)
/// - Relative strength (21-bar return vs rolling mean, z-scored)
/// - Volatility component (VIX ratio deviation, z-scored if available)
///
/// Parameters:
/// - data: OHLCV market data
/// - vix_values: optional VIX index values (same length)
/// - vix3m_values: optional VIX3M index values (same length)
/// - yz_window: Yang-Zhang vol window (default: 21)
/// - ema_fast: fast EMA for oscillator (default: 8)
/// - ema_slow: slow EMA for oscillator (default: 21)
/// - oscillator_smooth: EMA smoothing for oscillator (default: 5)
/// - vol_lookback: rolling window for vol percentile (default: 252)
/// - trend_period: SMA period for price trend (default: 50)
#[napi]
#[allow(clippy::too_many_arguments)]
pub fn regime_leverage(
    data: Vec<crate::MarketData>,
    vix_values: Option<Vec<f64>>,
    vix3m_values: Option<Vec<f64>>,
    yz_window: Option<u32>,
    ema_fast: Option<u32>,
    ema_slow: Option<u32>,
    oscillator_smooth: Option<u32>,
    vol_lookback: Option<u32>,
    trend_period: Option<u32>,
) -> Result<RegimeLeverageResult> {
    let market = process_market_data(data);
    let opens = &market.opens;
    let highs = &market.highs;
    let lows = &market.lows;
    let closes = &market.closes;
    let n = closes.len();

    let yz_w = yz_window.unwrap_or(21).max(2) as usize;
    let fast = ema_fast.unwrap_or(8) as usize;
    let slow = ema_slow.unwrap_or(21) as usize;
    let smooth = oscillator_smooth.unwrap_or(5) as usize;
    let vol_lb = vol_lookback.unwrap_or(252) as usize;
    let trend_p = trend_period.unwrap_or(50) as usize;

    if n < vol_lb + yz_w {
        return Err(Error::from_reason("Not enough data (need vol_lookback + yz_window bars)"));
    }

    // --- Yang-Zhang Volatility ---
    let yz_vol = compute_yz_vol(opens, highs, lows, closes, yz_w);

    // --- VIX ratio ---
    let has_vix = vix_values.is_some() && vix3m_values.is_some();
    let vix = vix_values.unwrap_or_default();
    let vix3m = vix3m_values.unwrap_or_default();

    let mut vix_ratio = vec![f64::NAN; n];
    if has_vix && vix.len() == n && vix3m.len() == n {
        for i in 0..n {
            if vix3m[i] > 0.0 {
                vix_ratio[i] = vix[i] / vix3m[i];
            }
        }
    }

    // --- Hybrid Oscillator ---
    // 1. Momentum: (EMA_fast / EMA_slow - 1) * 100
    let ema_f = compute_ema(closes, fast);
    let ema_s = compute_ema(closes, slow);

    let mut momentum = vec![f64::NAN; n];
    for i in 0..n {
        if !ema_f[i].is_nan() && !ema_s[i].is_nan() && ema_s[i] > 0.0 {
            momentum[i] = (ema_f[i] / ema_s[i] - 1.0) * 100.0;
        }
    }

    // 2. Relative strength: 21-bar return z-scored over 63 bars
    let mut rel_strength = vec![f64::NAN; n];
    for i in 21..n {
        let ret21 = if closes[i - 21] > 0.0 { (closes[i] - closes[i - 21]) / closes[i - 21] } else { 0.0 };

        if i >= 252 {
            // Rolling mean and std of 21-bar returns over 252 bars
            let mut sum = 0.0;
            let mut count = 0.0;
            for j in (i - 251)..=i {
                if j >= 21 && closes[j - 21] > 0.0 {
                    sum += (closes[j] - closes[j - 21]) / closes[j - 21];
                    count += 1.0;
                }
            }
            if count > 1.0 {
                let mean = sum / count;
                let mut var_sum = 0.0;
                for j in (i - 251)..=i {
                    if j >= 21 && closes[j - 21] > 0.0 {
                        let r = (closes[j] - closes[j - 21]) / closes[j - 21];
                        var_sum += (r - mean).powi(2);
                    }
                }
                let std = (var_sum / count).sqrt();
                rel_strength[i] = if std > 1e-15 { (ret21 - mean) / std } else { 0.0 };
            }
        }
    }

    // 3. VIX ratio component (if available)
    let mut vol_component = vec![0.0; n];
    if has_vix {
        for i in 63..n {
            if !vix_ratio[i].is_nan() {
                let window: Vec<f64> = (0..21).filter_map(|j| {
                    let idx = i - j;
                    if !vix_ratio[idx].is_nan() { Some(vix_ratio[idx]) } else { None }
                }).collect();

                if !window.is_empty() {
                    let mean_vr: f64 = window.iter().sum::<f64>() / window.len() as f64;
                    let dev = vix_ratio[i] - mean_vr;

                    let std_window: Vec<f64> = (0..63).filter_map(|j| {
                        let idx = i - j;
                        if !vix_ratio[idx].is_nan() { Some(vix_ratio[idx]) } else { None }
                    }).collect();

                    if std_window.len() > 1 {
                        let std_mean = std_window.iter().sum::<f64>() / std_window.len() as f64;
                        let std = (std_window.iter().map(|v| (v - std_mean).powi(2)).sum::<f64>() / std_window.len() as f64).sqrt();
                        vol_component[i] = if std > 1e-15 { dev / std } else { 0.0 };
                    }
                }
            }
        }
    }

    // Combine: 0.5 * momentum + 0.3 * rel_strength - 0.2 * vol_component
    let mut raw_osc = vec![f64::NAN; n];
    for i in 0..n {
        let m = if momentum[i].is_nan() { 0.0 } else { momentum[i] };
        let r = if rel_strength[i].is_nan() { 0.0 } else { rel_strength[i] };
        let v = vol_component[i];

        if !momentum[i].is_nan() {
            raw_osc[i] = 0.5 * m + 0.3 * r - 0.2 * v;
        }
    }

    // Smooth oscillator with EMA
    let oscillator = smooth_with_ema(&raw_osc, smooth);

    // --- Vol percentile (rolling rank) ---
    let mut vol_percentile = vec![f64::NAN; n];
    for i in vol_lb..n {
        if yz_vol[i].is_nan() { continue; }
        let mut count_below = 0.0;
        let mut total = 0.0;
        for j in (i - vol_lb + 1)..=i {
            if !yz_vol[j].is_nan() {
                total += 1.0;
                if yz_vol[j] <= yz_vol[i] {
                    count_below += 1.0;
                }
            }
        }
        if total > 0.0 {
            vol_percentile[i] = count_below / total;
        }
    }

    // --- Price trend: close > SMA(trend_period) ---
    let sma_trend = rolling_sma(closes, trend_p);

    // --- Regime classification ---
    let mut regime = vec![-1i32; n];
    let mut leverage = vec![f64::NAN; n];

    for i in 0..n {
        if vol_percentile[i].is_nan() || oscillator[i].is_nan() {
            continue;
        }

        let low_vol = vol_percentile[i] < 0.2;
        let high_vol = vol_percentile[i] > 0.8;
        let osc_bullish = oscillator[i] > 0.5;
        let osc_bearish = oscillator[i] < -0.5;
        let price_above_trend = !sma_trend[i].is_nan() && closes[i] > sma_trend[i];

        let vix_contango = if has_vix && !vix_ratio[i].is_nan() { vix_ratio[i] < 0.9 } else { true };
        let vix_backwardation = if has_vix && !vix_ratio[i].is_nan() { vix_ratio[i] > 1.1 } else { false };

        // Defensive first (highest priority)
        if osc_bearish || high_vol || vix_backwardation {
            regime[i] = 0; // Defensive
            leverage[i] = 0.0;
        } else if osc_bullish && low_vol && price_above_trend && vix_contango {
            regime[i] = 3; // Aggressive
            leverage[i] = 3.0;
        } else if osc_bullish && price_above_trend && !high_vol {
            regime[i] = 2; // Bullish
            leverage[i] = 2.0;
        } else {
            regime[i] = 1; // Moderate
            leverage[i] = 1.0;
        }
    }

    Ok(RegimeLeverageResult {
        oscillator,
        yz_volatility: yz_vol,
        vol_percentile,
        regime,
        leverage,
        vix_ratio,
    })
}

// --- Internal helpers ---

fn compute_yz_vol(opens: &[f64], highs: &[f64], lows: &[f64], closes: &[f64], w: usize) -> Vec<f64> {
    let n = opens.len();
    let k = 0.34 / (1.34 + (w as f64 + 1.0) / (w as f64 - 1.0));
    let sqrt252 = 252.0_f64.sqrt();
    let mut result = vec![f64::NAN; n];

    #[allow(clippy::needless_range_loop)]
    for i in w..n {
        let start = i - w + 1;
        let mut sum_co2 = 0.0;
        let mut sum_oc2 = 0.0;
        let mut sum_rs = 0.0;
        let mut count = 0.0;

        for j in start..=i {
            if j == 0 || opens[j] <= 0.0 || closes[j - 1] <= 0.0 || closes[j] <= 0.0 { continue; }
            let log_co = (opens[j] / closes[j - 1]).ln();
            let log_oc = (closes[j] / opens[j]).ln();
            let log_ho = (highs[j] / opens[j]).ln();
            let log_lo = (lows[j] / opens[j]).ln();
            let log_hc = (highs[j] / closes[j]).ln();
            let log_lc = (lows[j] / closes[j]).ln();

            sum_co2 += log_co * log_co;
            sum_oc2 += log_oc * log_oc;
            sum_rs += log_ho * log_hc + log_lo * log_lc;
            count += 1.0;
        }

        if count >= 2.0 {
            let yz_var = sum_co2 / count + k * sum_oc2 / count + (1.0 - k) * sum_rs / count;
            result[i] = if yz_var > 0.0 { yz_var.sqrt() * sqrt252 } else { 0.0 };
        }
    }

    result
}

fn compute_ema(data: &[f64], period: usize) -> Vec<f64> {
    let n = data.len();
    let mut result = vec![f64::NAN; n];
    if n < period { return result; }

    let k = 2.0 / (period as f64 + 1.0);
    let seed: f64 = data[..period].iter().sum::<f64>() / period as f64;
    result[period - 1] = seed;

    for i in period..n {
        let prev = result[i - 1];
        result[i] = k * data[i] + (1.0 - k) * prev;
    }

    result
}

fn smooth_with_ema(data: &[f64], period: usize) -> Vec<f64> {
    let n = data.len();
    let mut result = vec![f64::NAN; n];
    let k = 2.0 / (period as f64 + 1.0);

    let mut prev = f64::NAN;
    for i in 0..n {
        if data[i].is_nan() { continue; }
        if prev.is_nan() {
            prev = data[i];
            result[i] = data[i];
        } else {
            prev = k * data[i] + (1.0 - k) * prev;
            result[i] = prev;
        }
    }

    result
}

fn rolling_sma(data: &[f64], period: usize) -> Vec<f64> {
    let n = data.len();
    let mut result = vec![f64::NAN; n];
    if n < period { return result; }

    let mut sum: f64 = data[..period].iter().sum();
    result[period - 1] = sum / period as f64;

    for i in period..n {
        sum += data[i] - data[i - period];
        result[i] = sum / period as f64;
    }

    result
}
