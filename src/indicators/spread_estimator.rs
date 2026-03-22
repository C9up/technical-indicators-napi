use napi::bindgen_prelude::*;
use napi_derive::napi;
use crate::helpers::low_high_open_close_volume_date_to_array_helper::process_market_data;

#[napi(object)]
pub struct SpreadEstimatorResult {
    /// Rolling bid-ask spread estimates (0.01 = 1% spread)
    pub spreads: Vec<f64>,
    /// Rolling bid-ask spread with sign preserved
    pub signed_spreads: Vec<f64>,
}

/// Rolling Bid-Ask Spread Estimator (Ardia, Guidotti & Kroencke, 2024)
///
/// Estimates bid-ask spread from OHLC prices using moment conditions
/// and a rolling window. More accurate than Roll (1984), Corwin-Schultz (2012),
/// and Abdi-Ranaldo (2017) estimators, especially in low-liquidity markets.
///
/// A returned value of 0.01 means a 1% spread.
#[napi]
pub fn spread_estimator(
    data: Vec<crate::MarketData>,
    window: u32,
) -> Result<SpreadEstimatorResult> {
    let window = window as usize;

    if window < 2 {
        return Err(Error::from_reason("Window must be at least 2"));
    }

    let market = process_market_data(data);
    let opens = &market.opens;
    let highs = &market.highs;
    let lows = &market.lows;
    let closes = &market.closes;
    let n = opens.len();

    if n < window + 1 {
        return Err(Error::from_reason("Not enough data for the given window"));
    }

    // Log-transform prices
    let log_open: Vec<f64> = opens.iter().map(|x| x.ln()).collect();
    let log_high: Vec<f64> = highs.iter().map(|x| x.ln()).collect();
    let log_low: Vec<f64> = lows.iter().map(|x| x.ln()).collect();
    let log_close: Vec<f64> = closes.iter().map(|x| x.ln()).collect();
    let log_mid: Vec<f64> = log_high.iter().zip(log_low.iter()).map(|(h, l)| (h + l) / 2.0).collect();

    // Compute log-returns and indicators (starting from index 1 due to lagged values)
    let valid_n = n - 1;
    let mut r1 = Vec::with_capacity(valid_n); // mid - open
    let mut r2 = Vec::with_capacity(valid_n); // open - prev_mid
    let mut r3 = Vec::with_capacity(valid_n); // mid - prev_close
    let mut r4 = Vec::with_capacity(valid_n); // prev_close - prev_mid
    let mut r5 = Vec::with_capacity(valid_n); // open - prev_close
    let mut tau = Vec::with_capacity(valid_n); // non-flat indicator

    for i in 1..n {
        r1.push(log_mid[i] - log_open[i]);
        r2.push(log_open[i] - log_mid[i - 1]);
        r3.push(log_mid[i] - log_close[i - 1]);
        r4.push(log_close[i - 1] - log_mid[i - 1]);
        r5.push(log_open[i] - log_close[i - 1]);

        // tau: indicator for non-flat periods
        let is_non_flat = (log_high[i] - log_low[i]).abs() > 1e-15
            || (log_low[i] - log_close[i - 1]).abs() > 1e-15;
        tau.push(if is_non_flat { 1.0 } else { 0.0 });
    }

    // Compute moment products for GMM estimation
    // m1 = tau * r1 * r2 (related to spread)
    // m2 = tau * r3 * r4 (related to spread)
    // m3 = tau * r5^2    (related to variance, used for weighting)
    let mut m1 = Vec::with_capacity(valid_n);
    let mut m2 = Vec::with_capacity(valid_n);
    let mut m3 = Vec::with_capacity(valid_n);

    for i in 0..valid_n {
        m1.push(tau[i] * r1[i] * r2[i]);
        m2.push(tau[i] * r3[i] * r4[i]);
        m3.push(tau[i] * r5[i] * r5[i]);
    }

    // Rolling window estimation
    let mut spreads = vec![f64::NAN; window]; // NaN padding for warmup
    let mut signed_spreads = vec![f64::NAN; window];

    for end in window..=valid_n {
        let start = end - window;
        let win_m1 = &m1[start..end];
        let win_m2 = &m2[start..end];
        let win_m3 = &m3[start..end];
        let win_tau = &tau[start..end];

        // Count valid (non-flat) observations
        let tau_sum: f64 = win_tau.iter().sum();
        if tau_sum < 2.0 {
            spreads.push(0.0);
            signed_spreads.push(0.0);
            continue;
        }

        // Means of moment conditions
        let mean_m1: f64 = win_m1.iter().sum::<f64>() / tau_sum;
        let mean_m2: f64 = win_m2.iter().sum::<f64>() / tau_sum;
        let mean_m3: f64 = win_m3.iter().sum::<f64>() / tau_sum;

        // Variance of moment conditions (for GMM weighting)
        let var_m1: f64 = win_m1.iter()
            .zip(win_tau.iter())
            .map(|(m, t)| t * (m - mean_m1).powi(2))
            .sum::<f64>() / tau_sum;

        let var_m2: f64 = win_m2.iter()
            .zip(win_tau.iter())
            .map(|(m, t)| t * (m - mean_m2).powi(2))
            .sum::<f64>() / tau_sum;

        // GMM-weighted spread estimate: combine m1 and m2 with inverse-variance weights
        let (spread_signed, spread) = if var_m1 + var_m2 > 0.0 {
            let w1 = if var_m1 > 0.0 { 1.0 / var_m1 } else { 0.0 };
            let w2 = if var_m2 > 0.0 { 1.0 / var_m2 } else { 0.0 };
            let w_total = w1 + w2;

            if w_total > 0.0 {
                let weighted_mean = (w1 * mean_m1 + w2 * mean_m2) / w_total;
                // spread² ≈ -4 * weighted_mean (from the bid-ask bounce model)
                let s2 = -4.0 * weighted_mean;
                let signed = if s2 >= 0.0 { s2.sqrt() } else { -(-s2).sqrt() };
                let unsigned = s2.max(0.0).sqrt();
                (signed, unsigned)
            } else {
                // Fallback: simple average
                let avg = (mean_m1 + mean_m2) / 2.0;
                let s2 = -4.0 * avg;
                let signed = if s2 >= 0.0 { s2.sqrt() } else { -(-s2).sqrt() };
                (signed, s2.max(0.0).sqrt())
            }
        } else if mean_m3 > 0.0 {
            // Fallback using variance-based estimate
            let s2 = -4.0 * (mean_m1 + mean_m2) / 2.0;
            let signed = if s2 >= 0.0 { s2.sqrt() } else { -(-s2).sqrt() };
            (signed, s2.max(0.0).sqrt())
        } else {
            (0.0, 0.0)
        };

        spreads.push(spread);
        signed_spreads.push(spread_signed);
    }

    Ok(SpreadEstimatorResult {
        spreads,
        signed_spreads,
    })
}

/// Classic Roll (1984) spread estimator for comparison.
/// spread = 2 * sqrt(-Cov(ΔP_t, ΔP_{t-1})) if covariance is negative, else 0.
#[napi]
pub fn roll_spread_estimator(
    prices: Vec<f64>,
    window: u32,
) -> Result<Vec<f64>> {
    let window = window as usize;

    if window < 3 {
        return Err(Error::from_reason("Window must be at least 3"));
    }
    if prices.len() < window + 1 {
        return Err(Error::from_reason("Not enough data for the given window"));
    }

    // Price changes
    let changes: Vec<f64> = prices.windows(2).map(|w| w[1] - w[0]).collect();

    let mut spreads = vec![f64::NAN; window];

    for end in window..changes.len() {
        let start = end - window;
        let win = &changes[start..end];

        // Compute serial covariance: Cov(ΔP_t, ΔP_{t-1})
        let n = win.len() - 1;
        let mean: f64 = win.iter().sum::<f64>() / win.len() as f64;

        let mut cov = 0.0;
        for i in 1..win.len() {
            cov += (win[i] - mean) * (win[i - 1] - mean);
        }
        cov /= n as f64;

        // Roll spread = 2 * sqrt(-cov) if cov < 0
        let spread = if cov < 0.0 { 2.0 * (-cov).sqrt() } else { 0.0 };
        spreads.push(spread);
    }

    Ok(spreads)
}

/// Corwin-Schultz (2012) High-Low spread estimator.
/// Uses high and low prices over two consecutive periods.
#[napi]
pub fn corwin_schultz_spread_estimator(
    data: Vec<crate::MarketData>,
    window: u32,
) -> Result<Vec<f64>> {
    let window = window as usize;
    let market = process_market_data(data);
    let highs = &market.highs;
    let lows = &market.lows;
    let n = highs.len();

    if window < 2 {
        return Err(Error::from_reason("Window must be at least 2"));
    }
    if n < window + 1 {
        return Err(Error::from_reason("Not enough data for the given window"));
    }

    let sqrt2 = 2.0_f64.sqrt();

    // Compute beta and gamma for each pair of consecutive bars
    let mut spreads = vec![f64::NAN; window];

    for end in window..n {
        let start = end - window;
        let win_h = &highs[start..end];
        let win_l = &lows[start..end];

        let mut beta_sum = 0.0;
        let mut gamma_sum = 0.0;
        let mut count = 0;

        for i in 1..win_h.len() {
            if win_l[i] > 0.0 && win_l[i - 1] > 0.0 && win_h[i] > 0.0 && win_h[i - 1] > 0.0 {
                // Beta: sum of squared log(H/L) over two consecutive periods
                let beta = (win_h[i] / win_l[i]).ln().powi(2)
                    + (win_h[i - 1] / win_l[i - 1]).ln().powi(2);

                // Gamma: squared log of 2-period high/low ratio
                let h2 = win_h[i].max(win_h[i - 1]);
                let l2 = win_l[i].min(win_l[i - 1]);
                let gamma = (h2 / l2).ln().powi(2);

                beta_sum += beta;
                gamma_sum += gamma;
                count += 1;
            }
        }

        if count == 0 {
            spreads.push(0.0);
            continue;
        }

        let beta_avg = beta_sum / count as f64;
        let gamma_avg = gamma_sum / count as f64;

        // Alpha = (sqrt(2*beta) - sqrt(beta)) / (3 - 2*sqrt(2)) - sqrt(gamma / (3 - 2*sqrt(2)))
        let sqrt_2b = (2.0 * beta_avg).sqrt();
        let sqrt_b = beta_avg.sqrt();
        let denom = 3.0 - 2.0 * sqrt2;

        let alpha = if denom.abs() > 0.0 {
            (sqrt_2b - sqrt_b) / denom - (gamma_avg / denom).max(0.0).sqrt()
        } else {
            0.0
        };

        // Spread = 2 * (e^alpha - 1) / (1 + e^alpha)
        let spread = if alpha > 0.0 {
            let ea = alpha.exp();
            2.0 * (ea - 1.0) / (1.0 + ea)
        } else {
            0.0
        };

        spreads.push(spread);
    }

    Ok(spreads)
}
