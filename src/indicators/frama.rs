use napi::bindgen_prelude::*;
use napi_derive::napi;
use crate::helpers::low_high_open_close_volume_date_to_array_helper::process_market_data;

#[napi(object)]
pub struct FramaResult {
    /// FRAMA values (adaptive moving average)
    pub frama: Vec<f64>,
    /// Fractal dimension at each bar (1.0 = trending, 2.0 = choppy)
    pub fractal_dimension: Vec<f64>,
    /// Alpha (smoothing factor) at each bar
    pub alpha: Vec<f64>,
    /// FRAMA slope (bar-to-bar change, for trend detection)
    pub slope: Vec<f64>,
}

/// Fractal Adaptive Moving Average (FRAMA) — John Ehlers
///
/// An EMA whose smoothing factor adapts based on the fractal dimension of prices.
/// In trending markets (fractal dim ~1): FRAMA is fast and responsive.
/// In choppy markets (fractal dim ~2): FRAMA is slow and smooth.
///
/// Fractal dimension is estimated by comparing the price range over N bars
/// to the ranges of two N/2 sub-periods (Ehlers' method).
///
/// Parameters:
/// - data: OHLCV market data
/// - period: lookback for fractal calculation (default: 20, must be even)
/// - fast_period: fast EMA equivalent period when trending (default: 4)
/// - slow_period: slow EMA equivalent period when choppy (default: 200)
#[napi]
pub fn frama(
    data: Vec<crate::MarketData>,
    period: Option<u32>,
    fast_period: Option<u32>,
    slow_period: Option<u32>,
) -> Result<FramaResult> {
    let market = process_market_data(data);
    let highs = &market.highs;
    let lows = &market.lows;
    let closes = &market.closes;
    let n = closes.len();

    let mut p = period.unwrap_or(20) as usize;
    if p % 2 != 0 {
        p += 1; // must be even
    }
    let half = p / 2;
    let fast_p = fast_period.unwrap_or(4) as f64;
    let slow_p = slow_period.unwrap_or(200) as f64;

    if p < 4 {
        return Err(Error::from_reason("Period must be at least 4"));
    }
    if n < p + 1 {
        return Err(Error::from_reason("Not enough data for the given period"));
    }

    let fast_alpha = 2.0 / (fast_p + 1.0);
    let slow_alpha = 2.0 / (slow_p + 1.0);

    let mut frama_vals = vec![f64::NAN; n];
    let mut fractal_dim = vec![f64::NAN; n];
    let mut alpha_vals = vec![f64::NAN; n];
    let mut slope_vals = vec![f64::NAN; n];

    // Initialize FRAMA with close at the first valid bar
    frama_vals[p - 1] = closes[p - 1];

    for i in p..n {
        // --- Compute fractal dimension (Ehlers method) ---
        // First half: [i-p+1 .. i-half]
        let h1_start = i - p + 1;
        let h1_end = i - half;
        let hh1 = max_in_range(highs, h1_start, h1_end);
        let ll1 = min_in_range(lows, h1_start, h1_end);
        let n1 = (hh1 - ll1) / half as f64;

        // Second half: [i-half+1 .. i]
        let h2_start = i - half + 1;
        let h2_end = i;
        let hh2 = max_in_range(highs, h2_start, h2_end);
        let ll2 = min_in_range(lows, h2_start, h2_end);
        let n2 = (hh2 - ll2) / half as f64;

        // Full period: [i-p+1 .. i]
        let hh_full = max_in_range(highs, i - p + 1, i);
        let ll_full = min_in_range(lows, i - p + 1, i);
        let n3 = (hh_full - ll_full) / p as f64;

        // Fractal dimension
        let fd = if n1 + n2 > 0.0 && n3 > 0.0 {
            let d = ((n1 + n2).ln() - n3.ln()) / 2.0_f64.ln();
            d.clamp(1.0, 2.0)
        } else {
            1.5 // default to mid-range
        };

        fractal_dim[i] = fd;

        // --- Convert fractal dimension to alpha ---
        // fd=1 → trending → fast alpha; fd=2 → choppy → slow alpha
        let alpha = fast_alpha + (slow_alpha - fast_alpha) * (fd - 1.0);
        let alpha = alpha.clamp(slow_alpha, fast_alpha);
        alpha_vals[i] = alpha;

        // --- Update FRAMA ---
        let prev = if frama_vals[i - 1].is_nan() { closes[i - 1] } else { frama_vals[i - 1] };
        frama_vals[i] = alpha * closes[i] + (1.0 - alpha) * prev;

        // --- Slope ---
        if !frama_vals[i - 1].is_nan() {
            slope_vals[i] = frama_vals[i] - frama_vals[i - 1];
        }
    }

    Ok(FramaResult {
        frama: frama_vals,
        fractal_dimension: fractal_dim,
        alpha: alpha_vals,
        slope: slope_vals,
    })
}

fn max_in_range(data: &[f64], start: usize, end: usize) -> f64 {
    data[start..=end]
        .iter()
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max)
}

fn min_in_range(data: &[f64], start: usize, end: usize) -> f64 {
    data[start..=end]
        .iter()
        .cloned()
        .fold(f64::INFINITY, f64::min)
}
