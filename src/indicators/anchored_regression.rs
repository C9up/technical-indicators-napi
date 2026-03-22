use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi(object)]
pub struct RegressionSegment {
    /// Start index in the original data
    pub start_index: i32,
    /// End index in the original data
    pub end_index: i32,
    /// Slope (trend direction and strength)
    pub slope: f64,
    /// Intercept
    pub intercept: f64,
    /// Residual standard deviation
    pub std_dev: f64,
    /// Fitted values for this segment
    pub fitted: Vec<f64>,
    /// Upper band (fitted + mult * std_dev)
    pub upper_band: Vec<f64>,
    /// Lower band (fitted - mult * std_dev)
    pub lower_band: Vec<f64>,
}

#[napi(object)]
pub struct AnchoredRegressionResult {
    /// All regression segments
    pub segments: Vec<RegressionSegment>,
    /// Full-length fitted line (NaN where no regression)
    pub fitted: Vec<f64>,
    /// Full-length upper band
    pub upper_band: Vec<f64>,
    /// Full-length lower band
    pub lower_band: Vec<f64>,
    /// Full-length slope values (slope of the segment at each bar)
    pub slopes: Vec<f64>,
}

/// Compute simple linear regression: y = slope * x + intercept
/// Returns (slope, intercept, residual_std_dev)
fn linear_regression(prices: &[f64]) -> (f64, f64, f64) {
    let n = prices.len() as f64;
    if n < 2.0 {
        return (0.0, prices.first().copied().unwrap_or(0.0), 0.0);
    }

    let mut sum_x = 0.0;
    let mut sum_y = 0.0;
    let mut sum_xy = 0.0;
    let mut sum_xx = 0.0;

    for (i, &y) in prices.iter().enumerate() {
        let x = i as f64;
        sum_x += x;
        sum_y += y;
        sum_xy += x * y;
        sum_xx += x * x;
    }

    let denom = n * sum_xx - sum_x * sum_x;
    let (slope, intercept) = if denom.abs() > 1e-15 {
        let m = (n * sum_xy - sum_x * sum_y) / denom;
        let b = (sum_y - m * sum_x) / n;
        (m, b)
    } else {
        (0.0, sum_y / n)
    };

    // Residual standard deviation
    let mut ss_res = 0.0;
    for (i, &y) in prices.iter().enumerate() {
        let fitted = slope * i as f64 + intercept;
        ss_res += (y - fitted).powi(2);
    }
    let std_dev = if n > 2.0 {
        (ss_res / (n - 2.0)).sqrt()
    } else {
        0.0
    };

    (slope, intercept, std_dev)
}

/// Static Anchored Regression
///
/// Divides the price series into fixed segments based on `anchor_period` bars.
/// Each segment gets its own independent linear regression.
///
/// Parameters:
/// - prices: closing prices
/// - anchor_period: number of bars per segment (e.g. 5 for weekly on daily data)
/// - band_mult: multiplier for std dev bands (default: 1.0)
#[napi]
pub fn anchored_regression_static(
    prices: Vec<f64>,
    anchor_period: u32,
    band_mult: Option<f64>,
) -> Result<AnchoredRegressionResult> {
    let period = anchor_period as usize;
    let mult = band_mult.unwrap_or(1.0);

    if prices.is_empty() {
        return Err(Error::from_reason("Prices array cannot be empty"));
    }
    if period < 2 {
        return Err(Error::from_reason("Anchor period must be at least 2"));
    }

    let n = prices.len();
    let mut segments = Vec::new();
    let mut full_fitted = vec![f64::NAN; n];
    let mut full_upper = vec![f64::NAN; n];
    let mut full_lower = vec![f64::NAN; n];
    let mut full_slopes = vec![f64::NAN; n];

    let mut start = 0;
    while start < n {
        let end = (start + period).min(n);
        let segment_prices = &prices[start..end];

        if segment_prices.len() < 2 {
            break;
        }

        let (slope, intercept, std_dev) = linear_regression(segment_prices);

        let mut fitted = Vec::with_capacity(end - start);
        let mut upper = Vec::with_capacity(end - start);
        let mut lower = Vec::with_capacity(end - start);

        for i in 0..segment_prices.len() {
            let f = slope * i as f64 + intercept;
            fitted.push(f);
            upper.push(f + mult * std_dev);
            lower.push(f - mult * std_dev);
            full_fitted[start + i] = f;
            full_upper[start + i] = f + mult * std_dev;
            full_lower[start + i] = f - mult * std_dev;
            full_slopes[start + i] = slope;
        }

        segments.push(RegressionSegment {
            start_index: start as i32,
            end_index: (end - 1) as i32,
            slope,
            intercept,
            std_dev,
            fitted,
            upper_band: upper,
            lower_band: lower,
        });

        start = end;
    }

    Ok(AnchoredRegressionResult {
        segments,
        fitted: full_fitted,
        upper_band: full_upper,
        lower_band: full_lower,
        slopes: full_slopes,
    })
}

/// Rolling Anchored Regression
///
/// Regression updates bar-by-bar from each anchor reset point.
/// The anchor resets every `anchor_period` bars.
/// At each bar, the regression is computed from the last anchor point to the current bar.
///
/// Parameters:
/// - prices: closing prices
/// - anchor_period: bars between anchor resets (e.g. 5 for weekly on daily data)
/// - band_mult: multiplier for std dev bands (default: 1.0)
#[napi]
pub fn anchored_regression_rolling(
    prices: Vec<f64>,
    anchor_period: u32,
    band_mult: Option<f64>,
) -> Result<AnchoredRegressionResult> {
    let period = anchor_period as usize;
    let mult = band_mult.unwrap_or(1.0);

    if prices.is_empty() {
        return Err(Error::from_reason("Prices array cannot be empty"));
    }
    if period < 2 {
        return Err(Error::from_reason("Anchor period must be at least 2"));
    }

    let n = prices.len();
    let mut full_fitted = vec![f64::NAN; n];
    let mut full_upper = vec![f64::NAN; n];
    let mut full_lower = vec![f64::NAN; n];
    let mut full_slopes = vec![f64::NAN; n];
    let mut segments = Vec::new();

    let mut anchor_start = 0;

    for i in 0..n {
        // Check if we need to start a new anchor
        if i > 0 && (i - anchor_start) >= period {
            // Finalize the previous segment
            let seg_prices = &prices[anchor_start..i];
            let (slope, intercept, std_dev) = linear_regression(seg_prices);

            let mut fitted = Vec::with_capacity(i - anchor_start);
            let mut upper = Vec::with_capacity(i - anchor_start);
            let mut lower = Vec::with_capacity(i - anchor_start);

            for j in 0..seg_prices.len() {
                let f = slope * j as f64 + intercept;
                fitted.push(f);
                upper.push(f + mult * std_dev);
                lower.push(f - mult * std_dev);
            }

            segments.push(RegressionSegment {
                start_index: anchor_start as i32,
                end_index: (i - 1) as i32,
                slope,
                intercept,
                std_dev,
                fitted,
                upper_band: upper,
                lower_band: lower,
            });

            anchor_start = i;
        }

        // Compute rolling regression from anchor_start to current bar
        let window = &prices[anchor_start..=i];
        if window.len() >= 2 {
            let (slope, intercept, std_dev) = linear_regression(window);
            let local_idx = i - anchor_start;
            let f = slope * local_idx as f64 + intercept;

            full_fitted[i] = f;
            full_upper[i] = f + mult * std_dev;
            full_lower[i] = f - mult * std_dev;
            full_slopes[i] = slope;
        } else {
            full_fitted[i] = prices[i];
            full_upper[i] = prices[i];
            full_lower[i] = prices[i];
            full_slopes[i] = 0.0;
        }
    }

    // Finalize last segment
    if anchor_start < n {
        let seg_prices = &prices[anchor_start..n];
        if seg_prices.len() >= 2 {
            let (slope, intercept, std_dev) = linear_regression(seg_prices);

            let mut fitted = Vec::with_capacity(n - anchor_start);
            let mut upper = Vec::with_capacity(n - anchor_start);
            let mut lower = Vec::with_capacity(n - anchor_start);

            for j in 0..seg_prices.len() {
                let f = slope * j as f64 + intercept;
                fitted.push(f);
                upper.push(f + mult * std_dev);
                lower.push(f - mult * std_dev);
            }

            segments.push(RegressionSegment {
                start_index: anchor_start as i32,
                end_index: (n - 1) as i32,
                slope,
                intercept,
                std_dev,
                fitted,
                upper_band: upper,
                lower_band: lower,
            });
        }
    }

    Ok(AnchoredRegressionResult {
        segments,
        fitted: full_fitted,
        upper_band: full_upper,
        lower_band: full_lower,
        slopes: full_slopes,
    })
}
