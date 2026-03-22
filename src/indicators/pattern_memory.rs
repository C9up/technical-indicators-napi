use napi::bindgen_prelude::*;
use napi_derive::napi;
use crate::helpers::low_high_open_close_volume_date_to_array_helper::process_market_data;

#[napi(object)]
pub struct PatternMemoryResult {
    /// Directional signal at each bar: sum of labels from k-nearest neighbors.
    /// Positive = historically bullish, negative = historically bearish.
    pub signal: Vec<f64>,
    /// Normalized signal: signal / k (range -1 to +1)
    pub normalized_signal: Vec<f64>,
    /// Number of bullish neighbors at each bar
    pub bullish_count: Vec<i32>,
    /// Number of bearish neighbors at each bar
    pub bearish_count: Vec<i32>,
    /// Average Lorentzian distance to the k-nearest neighbors
    pub avg_distance: Vec<f64>,
}

/// Pattern Memory (Lorentzian Classification)
///
/// Non-parametric, memory-based directional signal. For each bar:
/// 1. Encode market state as a feature vector (5 indicators x window bars)
/// 2. Compare to all past states within a lookback using Lorentzian distance
/// 3. Find k-nearest neighbors and check what followed (+1 up, -1 down)
/// 4. Sum the labels as a directional signal
///
/// Features computed internally:
/// - RSI(14), WaveTrend(10,11), CCI(20), ADX(14), RSI(9)
///
/// Parameters:
/// - data: OHLCV market data
/// - k_neighbors: number of nearest neighbors (default: 100)
/// - lookback: how many past bars to search (default: 200)
/// - window: number of consecutive bars per feature vector (default: 5)
/// - forward_bars: bars ahead to determine label (default: 4)
#[napi]
pub fn pattern_memory(
    data: Vec<crate::MarketData>,
    k_neighbors: Option<u32>,
    lookback: Option<u32>,
    window: Option<u32>,
    forward_bars: Option<u32>,
) -> Result<PatternMemoryResult> {
    let market = process_market_data(data);
    let highs = &market.highs;
    let lows = &market.lows;
    let closes = &market.closes;
    let n = closes.len();

    let k = k_neighbors.unwrap_or(100) as usize;
    let lb = lookback.unwrap_or(200) as usize;
    let win = window.unwrap_or(5) as usize;
    let fwd = forward_bars.unwrap_or(4) as usize;

    let min_needed = 60 + win; // warmup for indicators + window
    if n < min_needed {
        return Err(Error::from_reason(format!("Need at least {} data points", min_needed)));
    }

    // --- Compute 5 features ---
    let hlc3: Vec<f64> = (0..n).map(|i| (highs[i] + lows[i] + closes[i]) / 3.0).collect();

    let feat_rsi14 = compute_rsi(closes, 14);
    let feat_wt = compute_wave_trend(&hlc3, 10, 11);
    let feat_cci = compute_cci(&hlc3, 20);
    let feat_adx = compute_adx(highs, lows, closes, 14);
    let feat_rsi9 = compute_rsi(closes, 9);

    let num_features = 5;
    let vec_len = num_features * win;

    // --- Build feature matrix (flattened windows) ---
    // feature_vectors[i] is defined for i >= win-1 + warmup
    let warmup = 34; // enough for all indicators to stabilize
    let start = warmup + win - 1;

    let mut feature_vectors: Vec<Option<Vec<f64>>> = vec![None; n];

    #[allow(clippy::needless_range_loop)]
    for i in start..n {
        let mut vec = Vec::with_capacity(vec_len);
        let mut valid = true;

        for j in 0..win {
            let idx = i - (win - 1) + j;
            if feat_rsi14[idx].is_nan() || feat_wt[idx].is_nan() || feat_cci[idx].is_nan()
                || feat_adx[idx].is_nan() || feat_rsi9[idx].is_nan()
            {
                valid = false;
                break;
            }
            vec.push(feat_rsi14[idx]);
            vec.push(feat_wt[idx]);
            vec.push(feat_cci[idx]);
            vec.push(feat_adx[idx]);
            vec.push(feat_rsi9[idx]);
        }

        if valid {
            feature_vectors[i] = Some(vec);
        }
    }

    // --- Labels: +1 if price rose in next `fwd` bars, -1 if fell, 0 if flat ---
    let mut labels = vec![0i8; n];
    for i in 0..(n.saturating_sub(fwd)) {
        let future_ret = closes[i + fwd] - closes[i];
        labels[i] = if future_ret > 0.0 { 1 } else if future_ret < 0.0 { -1 } else { 0 };
    }

    // --- For each bar, find k-nearest past neighbors and sum labels ---
    let mut signal = vec![f64::NAN; n];
    let mut normalized = vec![f64::NAN; n];
    let mut bullish = vec![0i32; n];
    let mut bearish = vec![0i32; n];
    let mut avg_dist = vec![f64::NAN; n];

    #[allow(clippy::needless_range_loop)]
    for i in start..n {
        let current_vec = match &feature_vectors[i] {
            Some(v) => v,
            None => continue,
        };

        // Search window: [max(start, i-lb) .. i-fwd] (don't include recent bars without labels)
        let search_start = if i > lb { i - lb } else { start };
        let search_end = if i > fwd { i - fwd } else { continue };

        if search_end <= search_start {
            continue;
        }

        // Compute distances to all candidates
        let mut distances: Vec<(f64, i8)> = Vec::new();

        for j in search_start..search_end {
            if let Some(past_vec) = &feature_vectors[j] {
                let dist = lorentzian_distance(current_vec, past_vec);
                distances.push((dist, labels[j]));
            }
        }

        if distances.is_empty() {
            continue;
        }

        // Sort by distance (ascending)
        distances.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        // Take k nearest
        let take = k.min(distances.len());
        let nearest = &distances[..take];

        let mut sum_labels = 0.0;
        let mut sum_dist = 0.0;
        let mut bull = 0i32;
        let mut bear = 0i32;

        for &(d, label) in nearest {
            sum_labels += label as f64;
            sum_dist += d;
            if label > 0 { bull += 1; }
            if label < 0 { bear += 1; }
        }

        signal[i] = sum_labels;
        normalized[i] = sum_labels / take as f64;
        bullish[i] = bull;
        bearish[i] = bear;
        avg_dist[i] = sum_dist / take as f64;
    }

    Ok(PatternMemoryResult {
        signal,
        normalized_signal: normalized,
        bullish_count: bullish,
        bearish_count: bearish,
        avg_distance: avg_dist,
    })
}

/// Lorentzian distance: sum(ln(1 + |a_i - b_i|))
fn lorentzian_distance(a: &[f64], b: &[f64]) -> f64 {
    a.iter()
        .zip(b.iter())
        .map(|(ai, bi)| (1.0 + (ai - bi).abs()).ln())
        .sum()
}

// --- Internal indicator functions ---

fn compute_rsi(closes: &[f64], period: usize) -> Vec<f64> {
    let n = closes.len();
    let mut result = vec![f64::NAN; n];
    if n <= period { return result; }

    let changes: Vec<f64> = closes.windows(2).map(|w| w[1] - w[0]).collect();
    let mut avg_gain = 0.0;
    let mut avg_loss = 0.0;

    for c in &changes[..period] {
        if *c > 0.0 { avg_gain += c; } else { avg_loss -= c; }
    }
    avg_gain /= period as f64;
    avg_loss /= period as f64;

    result[period] = if avg_loss == 0.0 { 100.0 } else { 100.0 - 100.0 / (1.0 + avg_gain / avg_loss) };

    for i in period..changes.len() {
        let g = if changes[i] > 0.0 { changes[i] } else { 0.0 };
        let l = if changes[i] < 0.0 { -changes[i] } else { 0.0 };
        avg_gain = (avg_gain * (period as f64 - 1.0) + g) / period as f64;
        avg_loss = (avg_loss * (period as f64 - 1.0) + l) / period as f64;
        result[i + 1] = if avg_loss == 0.0 { 100.0 } else { 100.0 - 100.0 / (1.0 + avg_gain / avg_loss) };
    }

    result
}

fn compute_wave_trend(hlc3: &[f64], n1: usize, n2: usize) -> Vec<f64> {
    let n = hlc3.len();
    let mut result = vec![f64::NAN; n];
    if n < n1 + n2 { return result; }

    // ESA = EMA(hlc3, n1)
    let k1 = 2.0 / (n1 as f64 + 1.0);
    let mut esa = vec![0.0; n];
    esa[0] = hlc3[0];
    for i in 1..n {
        esa[i] = k1 * hlc3[i] + (1.0 - k1) * esa[i - 1];
    }

    // D = EMA(|hlc3 - ESA|, n1)
    let mut d = vec![0.0; n];
    d[0] = (hlc3[0] - esa[0]).abs();
    for i in 1..n {
        d[i] = k1 * (hlc3[i] - esa[i]).abs() + (1.0 - k1) * d[i - 1];
    }

    // CI = (hlc3 - ESA) / (0.015 * D)
    let mut ci = vec![0.0; n];
    for i in 0..n {
        ci[i] = if d[i].abs() > 1e-15 { (hlc3[i] - esa[i]) / (0.015 * d[i]) } else { 0.0 };
    }

    // WT = EMA(CI, n2)
    let k2 = 2.0 / (n2 as f64 + 1.0);
    result[0] = ci[0];
    for i in 1..n {
        let prev = if result[i - 1].is_nan() { ci[i - 1] } else { result[i - 1] };
        result[i] = k2 * ci[i] + (1.0 - k2) * prev;
    }

    // Set NaN for warmup
    for val in result.iter_mut().take(n1 + n2) {
        *val = f64::NAN;
    }

    result
}

fn compute_cci(series: &[f64], period: usize) -> Vec<f64> {
    let n = series.len();
    let mut result = vec![f64::NAN; n];
    if n < period { return result; }

    for i in (period - 1)..n {
        let window = &series[(i - period + 1)..=i];
        let ma = window.iter().sum::<f64>() / period as f64;
        let md = window.iter().map(|x| (x - ma).abs()).sum::<f64>() / period as f64;

        result[i] = if md.abs() > 1e-15 {
            (series[i] - ma) / (0.015 * md)
        } else {
            0.0
        };
    }

    result
}

fn compute_adx(highs: &[f64], lows: &[f64], closes: &[f64], period: usize) -> Vec<f64> {
    let n = highs.len();
    let mut result = vec![f64::NAN; n];
    if n < period * 2 { return result; }

    let alpha = 1.0 / period as f64;

    // TR, +DM, -DM
    let mut tr = vec![0.0; n];
    let mut plus_dm = vec![0.0; n];
    let mut minus_dm = vec![0.0; n];

    for i in 1..n {
        tr[i] = (highs[i] - lows[i])
            .max((highs[i] - closes[i - 1]).abs())
            .max((lows[i] - closes[i - 1]).abs());

        let up = highs[i] - highs[i - 1];
        let down = lows[i - 1] - lows[i];

        plus_dm[i] = if up > down && up > 0.0 { up } else { 0.0 };
        minus_dm[i] = if down > up && down > 0.0 { down } else { 0.0 };
    }

    // EMA smoothing
    let mut atr = vec![0.0; n];
    let mut plus_di = vec![0.0; n];
    let mut minus_di = vec![0.0; n];

    atr[1] = tr[1];
    let mut sm_plus = plus_dm[1];
    let mut sm_minus = minus_dm[1];

    for i in 2..n {
        atr[i] = alpha * tr[i] + (1.0 - alpha) * atr[i - 1];
        sm_plus = alpha * plus_dm[i] + (1.0 - alpha) * sm_plus;
        sm_minus = alpha * minus_dm[i] + (1.0 - alpha) * sm_minus;

        if atr[i] > 0.0 {
            plus_di[i] = 100.0 * sm_plus / atr[i];
            minus_di[i] = 100.0 * sm_minus / atr[i];
        }
    }

    // DX and ADX
    let mut dx = vec![0.0; n];
    for i in 1..n {
        let di_sum = plus_di[i] + minus_di[i];
        dx[i] = if di_sum > 0.0 { 100.0 * (plus_di[i] - minus_di[i]).abs() / di_sum } else { 0.0 };
    }

    // ADX = EMA(DX, period)
    let mut adx_val = dx[period];
    for i in (period + 1)..n {
        adx_val = alpha * dx[i] + (1.0 - alpha) * adx_val;
        result[i] = adx_val;
    }

    result
}
