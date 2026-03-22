use napi::bindgen_prelude::*;
use napi_derive::napi;
use crate::helpers::low_high_open_close_volume_date_to_array_helper::process_market_data;

#[napi(object)]
pub struct VolatilityBucket {
    /// "low", "medium", or "high"
    pub regime: String,
    /// ATR multiplier for this regime
    pub atr_multiplier: f64,
    /// Current ATR value
    pub atr: f64,
    /// Current volatility (rolling std of returns)
    pub volatility: f64,
    /// Stop-loss distance = ATR * multiplier
    pub stop_distance: f64,
    /// Low volatility threshold (percentile)
    pub low_threshold: f64,
    /// High volatility threshold (percentile)
    pub high_threshold: f64,
}

#[napi(object)]
pub struct VolatilityEngineResult {
    /// ATR values (full length, NaN for warmup)
    pub atr: Vec<f64>,
    /// Rolling volatility (std dev of returns, full length, NaN for warmup)
    pub volatility: Vec<f64>,
    /// Volatility regime at each bar: 0=low, 1=medium, 2=high, -1=warmup
    pub regimes: Vec<i32>,
    /// ATR multiplier selected at each bar
    pub atr_multipliers: Vec<f64>,
    /// Stop-loss distance at each bar (ATR * multiplier)
    pub stop_distances: Vec<f64>,
    /// Percentile-based low threshold at each bar
    pub low_thresholds: Vec<f64>,
    /// Percentile-based high threshold at each bar
    pub high_thresholds: Vec<f64>,
}

/// Volatility-Adaptive Engine
///
/// Computes ATR + rolling volatility (std dev of returns), then classifies
/// each bar into a volatility regime (low/medium/high) using rolling percentiles.
/// Each regime maps to a different ATR multiplier for dynamic stop-loss sizing.
///
/// Parameters:
/// - data: OHLCV market data
/// - atr_period: ATR lookback (default: 14)
/// - vol_period: rolling std dev period for returns (default: 20)
/// - vol_history_len: number of bars for percentile calculation (default: 200)
/// - vol_warmup: minimum history before assigning regimes (default: 50)
/// - percentile_low: low vol threshold percentile (default: 20)
/// - percentile_high: high vol threshold percentile (default: 80)
/// - low_vol_mult: ATR multiplier for low volatility (default: 1.5)
/// - med_vol_mult: ATR multiplier for medium volatility (default: 2.5)
/// - high_vol_mult: ATR multiplier for high volatility (default: 4.0)
#[napi]
#[allow(clippy::too_many_arguments)]
pub fn volatility_engine(
    data: Vec<crate::MarketData>,
    atr_period: Option<u32>,
    vol_period: Option<u32>,
    vol_history_len: Option<u32>,
    vol_warmup: Option<u32>,
    percentile_low: Option<f64>,
    percentile_high: Option<f64>,
    low_vol_mult: Option<f64>,
    med_vol_mult: Option<f64>,
    high_vol_mult: Option<f64>,
) -> Result<VolatilityEngineResult> {
    let market = process_market_data(data);
    let highs = &market.highs;
    let lows = &market.lows;
    let closes = &market.closes;
    let n = closes.len();

    let atr_p = atr_period.unwrap_or(14) as usize;
    let vol_p = vol_period.unwrap_or(20) as usize;
    let hist_len = vol_history_len.unwrap_or(200) as usize;
    let warmup = vol_warmup.unwrap_or(50) as usize;
    let pct_low = percentile_low.unwrap_or(20.0);
    let pct_high = percentile_high.unwrap_or(80.0);
    let mult_low = low_vol_mult.unwrap_or(1.5);
    let mult_med = med_vol_mult.unwrap_or(2.5);
    let mult_high = high_vol_mult.unwrap_or(4.0);

    if n < atr_p + 1 || n < vol_p + 1 {
        return Err(Error::from_reason("Not enough data for the given periods"));
    }

    // --- Compute ATR (Wilder's smoothing) ---
    let mut tr = vec![0.0; n];
    for i in 1..n {
        tr[i] = (highs[i] - lows[i])
            .max((highs[i] - closes[i - 1]).abs())
            .max((lows[i] - closes[i - 1]).abs());
    }

    let mut atr_values = vec![f64::NAN; n];
    // First ATR = SMA of first atr_p TR values
    let first_atr: f64 = tr[1..=atr_p].iter().sum::<f64>() / atr_p as f64;
    atr_values[atr_p] = first_atr;

    for i in (atr_p + 1)..n {
        let prev = atr_values[i - 1];
        atr_values[i] = (prev * (atr_p as f64 - 1.0) + tr[i]) / atr_p as f64;
    }

    // --- Compute rolling volatility (std dev of pct returns) ---
    let mut returns = vec![0.0; n];
    for i in 1..n {
        returns[i] = if closes[i - 1] != 0.0 {
            (closes[i] - closes[i - 1]) / closes[i - 1]
        } else {
            0.0
        };
    }

    let mut volatility = vec![f64::NAN; n];
    for i in vol_p..n {
        let window = &returns[(i - vol_p + 1)..=i];
        let mean = window.iter().sum::<f64>() / vol_p as f64;
        let var = window.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / vol_p as f64;
        volatility[i] = var.sqrt();
    }

    // --- Classify volatility regimes using rolling percentiles ---
    let mut regimes = vec![-1i32; n];
    let mut atr_multipliers = vec![f64::NAN; n];
    let mut stop_distances = vec![f64::NAN; n];
    let mut low_thresholds = vec![f64::NAN; n];
    let mut high_thresholds = vec![f64::NAN; n];

    // Rolling history buffer
    let mut vol_history: Vec<f64> = Vec::with_capacity(hist_len);

    for i in 0..n {
        if volatility[i].is_nan() || atr_values[i].is_nan() {
            continue;
        }

        // Update rolling history
        vol_history.push(volatility[i]);
        if vol_history.len() > hist_len {
            vol_history.remove(0);
        }

        // Need enough history for percentile calculation
        if vol_history.len() < warmup {
            continue;
        }

        // Compute percentile thresholds
        let mut sorted = vol_history.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let low_thresh = percentile_value(&sorted, pct_low);
        let high_thresh = percentile_value(&sorted, pct_high);

        low_thresholds[i] = low_thresh;
        high_thresholds[i] = high_thresh;

        // Classify regime
        let (regime, mult) = if volatility[i] < low_thresh {
            (0, mult_low)
        } else if volatility[i] > high_thresh {
            (2, mult_high)
        } else {
            (1, mult_med)
        };

        regimes[i] = regime;
        atr_multipliers[i] = mult;
        stop_distances[i] = atr_values[i] * mult;
    }

    Ok(VolatilityEngineResult {
        atr: atr_values,
        volatility,
        regimes,
        atr_multipliers,
        stop_distances,
        low_thresholds,
        high_thresholds,
    })
}

/// Get a single volatility bucket classification for the current bar
#[napi]
#[allow(clippy::too_many_arguments)]
pub fn volatility_bucket(
    current_atr: f64,
    current_volatility: f64,
    volatility_history: Vec<f64>,
    percentile_low: Option<f64>,
    percentile_high: Option<f64>,
    low_vol_mult: Option<f64>,
    med_vol_mult: Option<f64>,
    high_vol_mult: Option<f64>,
) -> Result<VolatilityBucket> {
    let pct_low = percentile_low.unwrap_or(20.0);
    let pct_high = percentile_high.unwrap_or(80.0);
    let mult_low = low_vol_mult.unwrap_or(1.5);
    let mult_med = med_vol_mult.unwrap_or(2.5);
    let mult_high = high_vol_mult.unwrap_or(4.0);

    if volatility_history.is_empty() {
        return Err(Error::from_reason("Volatility history cannot be empty"));
    }

    let mut sorted = volatility_history;
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let low_threshold = percentile_value(&sorted, pct_low);
    let high_threshold = percentile_value(&sorted, pct_high);

    let (regime, mult) = if current_volatility < low_threshold {
        ("low", mult_low)
    } else if current_volatility > high_threshold {
        ("high", mult_high)
    } else {
        ("medium", mult_med)
    };

    Ok(VolatilityBucket {
        regime: regime.to_string(),
        atr_multiplier: mult,
        atr: current_atr,
        volatility: current_volatility,
        stop_distance: current_atr * mult,
        low_threshold,
        high_threshold,
    })
}

fn percentile_value(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let n = sorted.len();
    let idx = p / 100.0 * (n as f64 - 1.0);
    let lo = idx.floor() as usize;
    let hi = (lo + 1).min(n - 1);
    let frac = idx - lo as f64;
    sorted[lo] * (1.0 - frac) + sorted[hi] * frac
}
