use napi::bindgen_prelude::*;
use napi_derive::napi;
use crate::helpers::low_high_open_close_volume_date_to_array_helper::process_market_data;

#[napi(object)]
pub struct ThreeWayResult {
    /// Combined score at each bar (-3 to +3)
    pub score: Vec<f64>,
    /// Trend component: +1 (SMA fast > slow), -1 (opposite), 0 (warmup)
    pub trend: Vec<f64>,
    /// Momentum component: +1 (RSI > 50), -1 (RSI < 50), 0 (neutral/warmup)
    pub momentum: Vec<f64>,
    /// Volatility component: +1 (expanding, ATR rising), -1 (contracting), 0 (warmup)
    pub volatility: Vec<f64>,
    /// Signal: 1 = strong buy (score >= 2), -1 = strong sell (score <= -2), 0 = neutral
    pub signals: Vec<i32>,
}

/// Three Way Indicator
///
/// Combines three independent market dimensions into a single composite score:
/// 1. Trend: SMA crossover (fast vs slow)
/// 2. Momentum: RSI position relative to 50
/// 3. Volatility: ATR direction (expanding or contracting)
///
/// Score ranges from -3 (all bearish) to +3 (all bullish).
/// Signals fire when score >= buy_threshold or <= -sell_threshold.
///
/// Parameters:
/// - data: OHLCV market data
/// - fast_sma: fast SMA period (default: 10)
/// - slow_sma: slow SMA period (default: 30)
/// - rsi_period: RSI period (default: 14)
/// - atr_period: ATR period (default: 14)
/// - atr_lookback: bars to compare ATR direction (default: 5)
/// - signal_threshold: absolute score threshold for signals (default: 2)
#[napi]
pub fn three_way_indicator(
    data: Vec<crate::MarketData>,
    fast_sma: Option<u32>,
    slow_sma: Option<u32>,
    rsi_period: Option<u32>,
    atr_period: Option<u32>,
    atr_lookback: Option<u32>,
    signal_threshold: Option<f64>,
) -> Result<ThreeWayResult> {
    let market = process_market_data(data);
    let highs = &market.highs;
    let lows = &market.lows;
    let closes = &market.closes;
    let n = closes.len();

    let fast = fast_sma.unwrap_or(10) as usize;
    let slow = slow_sma.unwrap_or(30) as usize;
    let rsi_p = rsi_period.unwrap_or(14) as usize;
    let atr_p = atr_period.unwrap_or(14) as usize;
    let atr_lb = atr_lookback.unwrap_or(5) as usize;
    let sig_thresh = signal_threshold.unwrap_or(2.0);

    let min_data = slow.max(rsi_p + 1).max(atr_p + atr_lb + 1);
    if n < min_data {
        return Err(Error::from_reason(format!("Need at least {} data points", min_data)));
    }

    // --- Component 1: Trend (SMA crossover) ---
    let sma_fast = rolling_sma(closes, fast);
    let sma_slow = rolling_sma(closes, slow);

    let mut trend = vec![0.0; n];
    for i in 0..n {
        if !sma_fast[i].is_nan() && !sma_slow[i].is_nan() {
            trend[i] = if sma_fast[i] > sma_slow[i] { 1.0 } else { -1.0 };
        }
    }

    // --- Component 2: Momentum (RSI relative to 50) ---
    let changes: Vec<f64> = closes.windows(2).map(|w| w[1] - w[0]).collect();
    let mut rsi_vals = vec![f64::NAN; n];

    if changes.len() >= rsi_p {
        let mut avg_gain = 0.0;
        let mut avg_loss = 0.0;
        for c in &changes[..rsi_p] {
            if *c > 0.0 { avg_gain += c; } else { avg_loss -= c; }
        }
        avg_gain /= rsi_p as f64;
        avg_loss /= rsi_p as f64;

        rsi_vals[rsi_p] = if avg_loss == 0.0 { 100.0 } else { 100.0 - 100.0 / (1.0 + avg_gain / avg_loss) };

        for i in rsi_p..changes.len() {
            let g = if changes[i] > 0.0 { changes[i] } else { 0.0 };
            let l = if changes[i] < 0.0 { -changes[i] } else { 0.0 };
            avg_gain = (avg_gain * (rsi_p as f64 - 1.0) + g) / rsi_p as f64;
            avg_loss = (avg_loss * (rsi_p as f64 - 1.0) + l) / rsi_p as f64;
            rsi_vals[i + 1] = if avg_loss == 0.0 { 100.0 } else { 100.0 - 100.0 / (1.0 + avg_gain / avg_loss) };
        }
    }

    let mut momentum = vec![0.0; n];
    for i in 0..n {
        if !rsi_vals[i].is_nan() {
            momentum[i] = if rsi_vals[i] > 50.0 { 1.0 } else if rsi_vals[i] < 50.0 { -1.0 } else { 0.0 };
        }
    }

    // --- Component 3: Volatility (ATR direction) ---
    let mut tr = vec![0.0; n];
    for i in 1..n {
        tr[i] = (highs[i] - lows[i])
            .max((highs[i] - closes[i - 1]).abs())
            .max((lows[i] - closes[i - 1]).abs());
    }

    let mut atr = vec![f64::NAN; n];
    if n > atr_p {
        atr[atr_p] = tr[1..=atr_p].iter().sum::<f64>() / atr_p as f64;
        for i in (atr_p + 1)..n {
            atr[i] = (atr[i - 1] * (atr_p as f64 - 1.0) + tr[i]) / atr_p as f64;
        }
    }

    let mut volatility = vec![0.0; n];
    for i in atr_lb..n {
        if !atr[i].is_nan() && !atr[i - atr_lb].is_nan() {
            volatility[i] = if atr[i] > atr[i - atr_lb] { 1.0 } else { -1.0 };
        }
    }

    // --- Composite score and signals ---
    let mut score = vec![0.0; n];
    let mut signals = vec![0i32; n];

    for i in 0..n {
        score[i] = trend[i] + momentum[i] + volatility[i];

        if score[i] >= sig_thresh {
            signals[i] = 1;
        } else if score[i] <= -sig_thresh {
            signals[i] = -1;
        }
    }

    Ok(ThreeWayResult {
        score,
        trend,
        momentum,
        volatility,
        signals,
    })
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
