use napi::bindgen_prelude::*;
use napi_derive::napi;
use crate::helpers::low_high_open_close_volume_date_to_array_helper::process_market_data;

#[napi(object)]
pub struct HarResult {
    /// Predicted volatility at each bar (annualized)
    pub predicted_vol: Vec<f64>,
    /// Daily volatility component (Yang-Zhang, 1-day)
    pub vol_daily: Vec<f64>,
    /// Weekly volatility component (5-day average of daily vol)
    pub vol_weekly: Vec<f64>,
    /// Monthly volatility component (22-day average of daily vol)
    pub vol_monthly: Vec<f64>,
    /// Volatility regime: 0=low, 1=medium, 2=high, -1=warmup
    pub regime: Vec<i32>,
    /// Suggested exposure: 2.0 (low vol), 1.0 (medium), 0.0 (high)
    pub exposure: Vec<f64>,
}

/// HAR-X Volatility Model (Heterogeneous Autoregressive with eXogenous variables)
///
/// Combines volatility from multiple timeframes:
/// - Daily (1-day Yang-Zhang volatility)
/// - Weekly (5-day average)
/// - Monthly (22-day average)
///
/// Predicts future volatility using: vol_pred = a + b1*vol_d + b2*vol_w + b3*vol_m + b4*vix
/// Coefficients estimated via rolling OLS regression.
///
/// Regime classification based on rolling percentiles:
/// - Low vol (< percentile_low): exposure = 2.0 (leveraged)
/// - Medium vol: exposure = 1.0 (normal)
/// - High vol (> percentile_high): exposure = 0.0 (cash)
///
/// Parameters:
/// - data: OHLCV market data
/// - yz_window: Yang-Zhang window (default: 10)
/// - har_lookback: OLS regression lookback (default: 252)
/// - percentile_low: low vol threshold (default: 25)
/// - percentile_high: high vol threshold (default: 75)
/// - vix_data: optional VIX values (same length as data) for HAR-X extension
#[napi]
#[allow(clippy::too_many_arguments)]
pub fn har_volatility(
    data: Vec<crate::MarketData>,
    yz_window: Option<u32>,
    har_lookback: Option<u32>,
    percentile_low: Option<f64>,
    percentile_high: Option<f64>,
    vix_data: Option<Vec<f64>>,
) -> Result<HarResult> {
    let market = process_market_data(data);
    let opens = &market.opens;
    let highs = &market.highs;
    let lows = &market.lows;
    let closes = &market.closes;
    let n = closes.len();

    let yz_w = yz_window.unwrap_or(10).max(2) as usize;
    let lookback = har_lookback.unwrap_or(252) as usize;
    let pct_low = percentile_low.unwrap_or(25.0);
    let pct_high = percentile_high.unwrap_or(75.0);

    if n < lookback + 22 {
        return Err(Error::from_reason("Not enough data (need at least lookback + 22 bars)"));
    }

    // Validate VIX data if provided
    let has_vix = vix_data.is_some();
    let vix = vix_data.unwrap_or_default();
    if has_vix && vix.len() != n {
        return Err(Error::from_reason("VIX data must have the same length as price data"));
    }

    // --- Step 1: Compute daily Yang-Zhang volatility ---
    let mut daily_vol = vec![f64::NAN; n];
    {
        let k = 0.34 / (1.34 + (yz_w as f64 + 1.0) / (yz_w as f64 - 1.0));
        let sqrt252 = 252.0_f64.sqrt();

        let mut log_co = vec![f64::NAN; n];
        let mut log_oc = vec![0.0; n];
        let mut rs_bar = vec![0.0; n];

        for i in 0..n {
            if opens[i] > 0.0 {
                log_oc[i] = (closes[i] / opens[i]).ln();
                let log_ho = (highs[i] / opens[i]).ln();
                let log_lo = (lows[i] / opens[i]).ln();
                let log_hc = if closes[i] > 0.0 { (highs[i] / closes[i]).ln() } else { 0.0 };
                let log_lc = if closes[i] > 0.0 { (lows[i] / closes[i]).ln() } else { 0.0 };
                rs_bar[i] = log_ho * log_hc + log_lo * log_lc;
            }
            if i > 0 && opens[i] > 0.0 && closes[i - 1] > 0.0 {
                log_co[i] = (opens[i] / closes[i - 1]).ln();
            }
        }

        #[allow(clippy::needless_range_loop)]
        for i in yz_w..n {
            let start = i - yz_w + 1;
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

            if count >= 2.0 {
                let yz_var = sum_co2 / count + k * sum_oc2 / count + (1.0 - k) * sum_rs / count;
                daily_vol[i] = if yz_var > 0.0 { yz_var.sqrt() * sqrt252 } else { 0.0 };
            }
        }
    }

    // --- Step 2: Weekly and Monthly volatility (rolling averages) ---
    let mut weekly_vol = vec![f64::NAN; n];
    let mut monthly_vol = vec![f64::NAN; n];

    for i in 4..n {
        let w = &daily_vol[(i - 4)..=i];
        let valid: Vec<f64> = w.iter().filter(|x| !x.is_nan()).copied().collect();
        if !valid.is_empty() {
            weekly_vol[i] = valid.iter().sum::<f64>() / valid.len() as f64;
        }
    }

    for i in 21..n {
        let w = &daily_vol[(i - 21)..=i];
        let valid: Vec<f64> = w.iter().filter(|x| !x.is_nan()).copied().collect();
        if !valid.is_empty() {
            monthly_vol[i] = valid.iter().sum::<f64>() / valid.len() as f64;
        }
    }

    // --- Step 3: HAR(-X) regression and prediction ---
    let mut predicted = vec![f64::NAN; n];
    let mut regime = vec![-1i32; n];
    let mut exposure = vec![f64::NAN; n];

    // Rolling history for percentile calculation
    let mut vol_history: Vec<f64> = Vec::with_capacity(lookback);

    let har_start = 22 + yz_w; // need enough data for monthly vol

    for i in har_start..n {
        // Skip if components not ready
        if daily_vol[i].is_nan() || weekly_vol[i].is_nan() || monthly_vol[i].is_nan() {
            continue;
        }

        // Rolling OLS: fit vol[t] = a + b1*vol_d[t-1] + b2*vol_w[t-1] + b3*vol_m[t-1] (+ b4*vix[t-1])
        let reg_start = if i > lookback { i - lookback } else { har_start };

        // Build X and y for regression
        let num_features = if has_vix { 4 } else { 3 };
        let mut x_rows: Vec<Vec<f64>> = Vec::new();
        let mut y_vals: Vec<f64> = Vec::new();

        for j in (reg_start + 1)..i {
            if daily_vol[j].is_nan() || daily_vol[j - 1].is_nan()
                || weekly_vol[j - 1].is_nan() || monthly_vol[j - 1].is_nan()
            {
                continue;
            }

            let mut row = vec![1.0, daily_vol[j - 1], weekly_vol[j - 1], monthly_vol[j - 1]];
            if has_vix {
                if j - 1 < vix.len() && !vix[j - 1].is_nan() {
                    row.push(vix[j - 1] / 100.0); // normalize VIX
                } else {
                    continue;
                }
            }

            x_rows.push(row);
            y_vals.push(daily_vol[j]);
        }

        // Need enough data for regression
        if x_rows.len() < (num_features + 2) * 2 {
            // Fallback: use simple average
            predicted[i] = (daily_vol[i] + weekly_vol[i] + monthly_vol[i]) / 3.0;
        } else {
            // Simple OLS: beta = (X'X)^-1 * X'y
            let coeffs = ols_regression(&x_rows, &y_vals, num_features + 1);

            // Predict
            let mut x_now = vec![1.0, daily_vol[i], weekly_vol[i], monthly_vol[i]];
            if has_vix && i < vix.len() && !vix[i].is_nan() {
                x_now.push(vix[i] / 100.0);
            } else if has_vix {
                x_now.push(0.0);
            }

            let pred: f64 = coeffs.iter().zip(x_now.iter()).map(|(b, x)| b * x).sum();
            predicted[i] = pred.max(0.0);
        }

        // Update vol history for percentile
        vol_history.push(predicted[i]);
        if vol_history.len() > lookback {
            vol_history.remove(0);
        }

        // Regime classification
        if vol_history.len() >= 50 {
            let mut sorted = vol_history.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

            let low_thresh = percentile_val(&sorted, pct_low);
            let high_thresh = percentile_val(&sorted, pct_high);

            if predicted[i] < low_thresh {
                regime[i] = 0;
                exposure[i] = 2.0;
            } else if predicted[i] > high_thresh {
                regime[i] = 2;
                exposure[i] = 0.0;
            } else {
                regime[i] = 1;
                exposure[i] = 1.0;
            }
        }
    }

    Ok(HarResult {
        predicted_vol: predicted,
        vol_daily: daily_vol,
        vol_weekly: weekly_vol,
        vol_monthly: monthly_vol,
        regime,
        exposure,
    })
}

/// Simple OLS regression: returns coefficient vector
/// X is a matrix of rows, y is the target vector
fn ols_regression(x: &[Vec<f64>], y: &[f64], num_cols: usize) -> Vec<f64> {
    // X'X
    let mut xtx = vec![vec![0.0; num_cols]; num_cols];
    for row in x {
        for i in 0..num_cols {
            for j in 0..num_cols {
                xtx[i][j] += row[i] * row[j];
            }
        }
    }

    // X'y
    let mut xty = vec![0.0; num_cols];
    for (idx, row) in x.iter().enumerate() {
        for i in 0..num_cols {
            xty[i] += row[i] * y[idx];
        }
    }

    // Add small ridge regularization for stability
    #[allow(clippy::needless_range_loop)]
    for i in 0..num_cols {
        xtx[i][i] += 1e-8;
    }

    // Solve via Gauss elimination
    gauss_solve(&mut xtx, &mut xty)
}

/// Gaussian elimination to solve Ax = b
#[allow(clippy::needless_range_loop)]
fn gauss_solve(a: &mut [Vec<f64>], b: &mut [f64]) -> Vec<f64> {
    let n = b.len();

    // Forward elimination with partial pivoting
    for col in 0..n {
        // Find pivot
        let mut max_row = col;
        let mut max_val = a[col][col].abs();
        for row in (col + 1)..n {
            if a[row][col].abs() > max_val {
                max_val = a[row][col].abs();
                max_row = row;
            }
        }

        // Swap rows
        if max_row != col {
            a.swap(col, max_row);
            b.swap(col, max_row);
        }

        let pivot = a[col][col];
        if pivot.abs() < 1e-12 {
            continue; // skip singular column
        }

        for row in (col + 1)..n {
            let factor = a[row][col] / pivot;
            for j in col..n {
                a[row][j] -= factor * a[col][j];
            }
            b[row] -= factor * b[col];
        }
    }

    // Back substitution
    let mut x = vec![0.0; n];
    for i in (0..n).rev() {
        if a[i][i].abs() < 1e-12 {
            x[i] = 0.0;
            continue;
        }
        let mut sum = b[i];
        for j in (i + 1)..n {
            sum -= a[i][j] * x[j];
        }
        x[i] = sum / a[i][i];
    }

    x
}

fn percentile_val(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() { return 0.0; }
    let n = sorted.len();
    let idx = p / 100.0 * (n as f64 - 1.0);
    let lo = idx.floor() as usize;
    let hi = (lo + 1).min(n - 1);
    let frac = idx - lo as f64;
    sorted[lo] * (1.0 - frac) + sorted[hi] * frac
}
