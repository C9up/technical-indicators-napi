use napi::bindgen_prelude::*;
use napi_derive::napi;
use crate::helpers::low_high_open_close_volume_date_to_array_helper::process_market_data;
use crate::helpers::calculate_ema_helper::calculate_ema;
use crate::helpers::calculate_sma_helper::calculate_sma;

#[napi(object)]
pub struct FeatureRow {
    /// Bar index
    pub index: i32,

    // --- Returns ---
    /// 1-bar return (pct change)
    pub return_1: f64,
    /// 5-bar return
    pub return_5: f64,
    /// 10-bar return
    pub return_10: f64,
    /// 20-bar return
    pub return_20: f64,

    // --- Volatility ---
    /// True Range
    pub true_range: f64,
    /// ATR (Wilder's, 14-period)
    pub atr_14: f64,
    /// Rolling std dev of 1-bar returns (20-period)
    pub volatility_20: f64,
    /// High-Low range as % of close
    pub range_pct: f64,

    // --- Momentum ---
    /// RSI (14-period, Wilder's)
    pub rsi_14: f64,
    /// Rate of Change (10-period)
    pub roc_10: f64,
    /// Momentum (close - close[10])
    pub momentum_10: f64,

    // --- Moving Averages ---
    /// SMA 5
    pub sma_5: f64,
    /// SMA 20
    pub sma_20: f64,
    /// SMA 50
    pub sma_50: f64,
    /// EMA 12
    pub ema_12: f64,
    /// EMA 26
    pub ema_26: f64,
    /// MACD line (EMA12 - EMA26)
    pub macd: f64,
    /// MACD signal (EMA9 of MACD)
    pub macd_signal: f64,
    /// MACD histogram
    pub macd_histogram: f64,

    // --- Bollinger Bands ---
    /// Bollinger %B: (close - lower) / (upper - lower)
    pub bb_pct_b: f64,
    /// Bollinger bandwidth: (upper - lower) / middle
    pub bb_bandwidth: f64,

    // --- Price Position ---
    /// Close relative to SMA20: (close - sma20) / sma20
    pub close_to_sma20: f64,
    /// Close relative to SMA50
    pub close_to_sma50: f64,
    /// Distance from 20-bar high (%)
    pub dist_from_high_20: f64,
    /// Distance from 20-bar low (%)
    pub dist_from_low_20: f64,

    // --- Volume ---
    /// Volume change (pct)
    pub volume_change: f64,
    /// Volume / SMA20 of volume
    pub volume_ratio: f64,

    // --- Candle Features ---
    /// Body size: |close - open| / (high - low)
    pub body_ratio: f64,
    /// Upper shadow: (high - max(open,close)) / (high - low)
    pub upper_shadow: f64,
    /// Lower shadow: (min(open,close) - low) / (high - low)
    pub lower_shadow: f64,
    /// Gap: (open - prev_close) / prev_close
    pub gap: f64,

    // --- Trend ---
    /// SMA5 > SMA20 (1.0 or 0.0)
    pub trend_sma_5_20: f64,
    /// SMA20 > SMA50 (1.0 or 0.0)
    pub trend_sma_20_50: f64,
}

/// Generate a complete feature matrix from OHLCV data for ML pipelines.
///
/// Computes ~35 features per bar covering returns, volatility, momentum,
/// moving averages, MACD, Bollinger Bands, price position, volume,
/// candle patterns, and trend signals.
///
/// First ~50 bars are skipped (warmup period). Returns one FeatureRow per valid bar.
#[napi]
pub fn feature_engine(data: Vec<crate::MarketData>) -> Result<Vec<FeatureRow>> {
    let market = process_market_data(data);
    let opens = &market.opens;
    let highs = &market.highs;
    let lows = &market.lows;
    let closes = &market.closes;
    let volumes = &market.volumes;
    let n = closes.len();

    if n < 52 {
        return Err(Error::from_reason("Need at least 52 data points"));
    }

    // Precompute indicators

    // --- ATR (Wilder's 14) ---
    let mut tr = vec![0.0; n];
    for i in 1..n {
        tr[i] = (highs[i] - lows[i])
            .max((highs[i] - closes[i - 1]).abs())
            .max((lows[i] - closes[i - 1]).abs());
    }
    let mut atr = vec![f64::NAN; n];
    if n > 14 {
        atr[14] = tr[1..=14].iter().sum::<f64>() / 14.0;
        for i in 15..n {
            atr[i] = (atr[i - 1] * 13.0 + tr[i]) / 14.0;
        }
    }

    // --- Returns ---
    let mut ret1 = vec![f64::NAN; n];
    for i in 1..n {
        ret1[i] = if closes[i - 1] != 0.0 { (closes[i] - closes[i - 1]) / closes[i - 1] } else { 0.0 };
    }

    // --- Rolling volatility (std dev of returns, 20-period) ---
    let mut vol20 = vec![f64::NAN; n];
    for i in 20..n {
        let w = &ret1[(i - 19)..=i];
        let valid: Vec<f64> = w.iter().filter(|x| !x.is_nan()).copied().collect();
        if valid.len() >= 2 {
            let mean = valid.iter().sum::<f64>() / valid.len() as f64;
            let var = valid.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / valid.len() as f64;
            vol20[i] = var.sqrt();
        }
    }

    // --- RSI 14 (Wilder's) ---
    let mut rsi = vec![f64::NAN; n];
    if n > 15 {
        let changes: Vec<f64> = closes.windows(2).map(|w| w[1] - w[0]).collect();
        let mut avg_gain = 0.0;
        let mut avg_loss = 0.0;
        for c in &changes[..14] {
            if *c > 0.0 { avg_gain += c; } else { avg_loss -= c; }
        }
        avg_gain /= 14.0;
        avg_loss /= 14.0;

        rsi[14] = if avg_loss == 0.0 { 100.0 } else { 100.0 - 100.0 / (1.0 + avg_gain / avg_loss) };

        for i in 14..changes.len() {
            let g = if changes[i] > 0.0 { changes[i] } else { 0.0 };
            let l = if changes[i] < 0.0 { -changes[i] } else { 0.0 };
            avg_gain = (avg_gain * 13.0 + g) / 14.0;
            avg_loss = (avg_loss * 13.0 + l) / 14.0;
            rsi[i + 1] = if avg_loss == 0.0 { 100.0 } else { 100.0 - 100.0 / (1.0 + avg_gain / avg_loss) };
        }
    }

    // --- SMAs ---
    let sma5 = calculate_sma(closes, 5).unwrap_or_else(|_| vec![f64::NAN; n]);
    let sma20 = calculate_sma(closes, 20).unwrap_or_else(|_| vec![f64::NAN; n]);
    let sma50 = calculate_sma(closes, 50).unwrap_or_else(|_| vec![f64::NAN; n]);

    // --- EMAs ---
    let ema12_raw = calculate_ema(closes, 12).unwrap_or_default();
    let ema26_raw = calculate_ema(closes, 26).unwrap_or_default();

    // Align EMA to full length (ema starts at index period-1)
    let mut ema12 = vec![f64::NAN; n];
    let mut ema26 = vec![f64::NAN; n];
    for (i, &v) in ema12_raw.iter().enumerate() {
        ema12[i + 11] = v;
    }
    for (i, &v) in ema26_raw.iter().enumerate() {
        ema26[i + 25] = v;
    }

    // --- MACD ---
    let mut macd_line = vec![f64::NAN; n];
    for i in 0..n {
        if !ema12[i].is_nan() && !ema26[i].is_nan() {
            macd_line[i] = ema12[i] - ema26[i];
        }
    }

    // MACD signal = EMA9 of MACD line (only valid portion)
    let macd_valid: Vec<f64> = macd_line.iter().filter(|x| !x.is_nan()).copied().collect();
    let macd_signal_raw = calculate_ema(&macd_valid, 9).unwrap_or_default();
    let mut macd_signal = vec![f64::NAN; n];
    let macd_start = n - macd_valid.len();
    for (i, &v) in macd_signal_raw.iter().enumerate() {
        let target = macd_start + i + 8; // EMA9 starts at index 8
        if target < n {
            macd_signal[target] = v;
        }
    }

    // --- Bollinger Bands (20, 2.0) ---
    let mut bb_upper = vec![f64::NAN; n];
    let mut bb_lower = vec![f64::NAN; n];
    let mut bb_middle = vec![f64::NAN; n];
    for i in 19..n {
        let w = &closes[(i - 19)..=i];
        let mean = w.iter().sum::<f64>() / 20.0;
        let var = w.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / 20.0;
        let std = var.max(0.0).sqrt();
        bb_middle[i] = mean;
        bb_upper[i] = mean + 2.0 * std;
        bb_lower[i] = mean - 2.0 * std;
    }

    // --- Volume SMA20 ---
    let vol_sma20 = calculate_sma(volumes, 20).unwrap_or_else(|_| vec![f64::NAN; n]);

    // --- Rolling high/low 20 ---
    let mut high20 = vec![f64::NAN; n];
    let mut low20 = vec![f64::NAN; n];
    for i in 19..n {
        let w_h = &highs[(i - 19)..=i];
        let w_l = &lows[(i - 19)..=i];
        high20[i] = w_h.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        low20[i] = w_l.iter().cloned().fold(f64::INFINITY, f64::min);
    }

    // --- Build feature rows (start at 50 for warmup) ---
    let start = 50;
    let mut rows = Vec::with_capacity(n - start);

    for i in start..n {
        let close = closes[i];
        let hl_range = highs[i] - lows[i];

        // Returns
        let r5 = if i >= 5 && closes[i - 5] != 0.0 { (close - closes[i - 5]) / closes[i - 5] } else { f64::NAN };
        let r10 = if i >= 10 && closes[i - 10] != 0.0 { (close - closes[i - 10]) / closes[i - 10] } else { f64::NAN };
        let r20 = if i >= 20 && closes[i - 20] != 0.0 { (close - closes[i - 20]) / closes[i - 20] } else { f64::NAN };

        // Momentum
        let roc10 = if i >= 10 && closes[i - 10] != 0.0 { (close - closes[i - 10]) / closes[i - 10] * 100.0 } else { f64::NAN };
        let mom10 = if i >= 10 { close - closes[i - 10] } else { f64::NAN };

        // Bollinger %B and bandwidth
        let bb_pct_b = if !bb_upper[i].is_nan() && !bb_lower[i].is_nan() {
            let bw = bb_upper[i] - bb_lower[i];
            if bw > 0.0 { (close - bb_lower[i]) / bw } else { 0.5 }
        } else { f64::NAN };

        let bb_bw = if !bb_upper[i].is_nan() && !bb_middle[i].is_nan() && bb_middle[i] > 0.0 {
            (bb_upper[i] - bb_lower[i]) / bb_middle[i]
        } else { f64::NAN };

        // Price position
        let c_to_sma20 = if !sma20[i].is_nan() && sma20[i] > 0.0 { (close - sma20[i]) / sma20[i] } else { f64::NAN };
        let c_to_sma50 = if !sma50[i].is_nan() && sma50[i] > 0.0 { (close - sma50[i]) / sma50[i] } else { f64::NAN };
        let d_high = if !high20[i].is_nan() && high20[i] > 0.0 { (close - high20[i]) / high20[i] } else { f64::NAN };
        let d_low = if !low20[i].is_nan() && low20[i] > 0.0 { (close - low20[i]) / low20[i] } else { f64::NAN };

        // Volume
        let vol_change = if i >= 1 && volumes[i - 1] > 0.0 { (volumes[i] - volumes[i - 1]) / volumes[i - 1] } else { f64::NAN };
        let vol_ratio = if !vol_sma20[i].is_nan() && vol_sma20[i] > 0.0 { volumes[i] / vol_sma20[i] } else { f64::NAN };

        // Candle features
        let body = (closes[i] - opens[i]).abs();
        let body_ratio = if hl_range > 0.0 { body / hl_range } else { 0.0 };
        let upper_shadow = if hl_range > 0.0 { (highs[i] - closes[i].max(opens[i])) / hl_range } else { 0.0 };
        let lower_shadow = if hl_range > 0.0 { (closes[i].min(opens[i]) - lows[i]) / hl_range } else { 0.0 };
        let gap = if i >= 1 && closes[i - 1] > 0.0 { (opens[i] - closes[i - 1]) / closes[i - 1] } else { f64::NAN };

        // MACD histogram
        let macd_hist = if !macd_line[i].is_nan() && !macd_signal[i].is_nan() {
            macd_line[i] - macd_signal[i]
        } else { f64::NAN };

        // Trend signals
        let trend_5_20 = if !sma5[i].is_nan() && !sma20[i].is_nan() { if sma5[i] > sma20[i] { 1.0 } else { 0.0 } } else { f64::NAN };
        let trend_20_50 = if !sma20[i].is_nan() && !sma50[i].is_nan() { if sma20[i] > sma50[i] { 1.0 } else { 0.0 } } else { f64::NAN };

        rows.push(FeatureRow {
            index: i as i32,
            return_1: ret1[i],
            return_5: r5,
            return_10: r10,
            return_20: r20,
            true_range: tr[i],
            atr_14: atr[i],
            volatility_20: vol20[i],
            range_pct: if close > 0.0 { hl_range / close } else { 0.0 },
            rsi_14: rsi[i],
            roc_10: roc10,
            momentum_10: mom10,
            sma_5: sma5[i],
            sma_20: sma20[i],
            sma_50: sma50[i],
            ema_12: ema12[i],
            ema_26: ema26[i],
            macd: macd_line[i],
            macd_signal: macd_signal[i],
            macd_histogram: macd_hist,
            bb_pct_b,
            bb_bandwidth: bb_bw,
            close_to_sma20: c_to_sma20,
            close_to_sma50: c_to_sma50,
            dist_from_high_20: d_high,
            dist_from_low_20: d_low,
            volume_change: vol_change,
            volume_ratio: vol_ratio,
            body_ratio,
            upper_shadow,
            lower_shadow,
            gap,
            trend_sma_5_20: trend_5_20,
            trend_sma_20_50: trend_20_50,
        });
    }

    Ok(rows)
}
