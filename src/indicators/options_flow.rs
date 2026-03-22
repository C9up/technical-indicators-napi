use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi(object)]
pub struct OptionContract {
    /// Strike price
    pub strike: f64,
    /// Open interest
    pub open_interest: f64,
    /// Daily volume
    pub volume: f64,
    /// Days to expiry
    pub dte: f64,
    /// "call" or "put"
    pub side: String,
    /// Implied volatility (optional, 0 if unknown)
    pub implied_volatility: f64,
}

#[napi(object)]
pub struct ScoredOption {
    /// Original contract index
    pub index: i32,
    pub strike: f64,
    pub open_interest: f64,
    pub volume: f64,
    pub dte: f64,
    pub side: String,
    pub implied_volatility: f64,
    /// OI/Volume ratio (capped)
    pub oi_volume_ratio: f64,
    /// Z-score of open interest within its expiry group
    pub oi_z_score: f64,
    /// Z-score of OI/Volume ratio within its expiry group
    pub ov_z_score: f64,
    /// Percentile rank of OI z-score (0-1)
    pub oi_percentile: f64,
    /// Percentile rank of OV z-score (0-1)
    pub ov_percentile: f64,
    /// OTM distance factor
    pub otm_factor: f64,
    /// DTE decay factor
    pub dte_factor: f64,
    /// Final composite score (higher = more institutional interest)
    pub score: f64,
}

/// Big Money Options Flow Scoring
///
/// Ranks option contracts by institutional interest using a composite score:
/// Score = w_oi * OI_percentile * dte_factor + w_ov * OV_percentile * dte_factor + w_otm * otm_factor
///
/// Parameters:
/// - contracts: array of option contracts with strike, OI, volume, DTE, side
/// - spot_price: current underlying price
/// - top_n: number of top contracts to return (default: 50)
/// - k_otm: OTM scaling factor (default: 2.0, higher = more penalty for far strikes)
/// - min_volume: minimum volume filter (default: 10)
/// - min_oi: minimum open interest filter (default: 100)
/// - cap_oi_vol: cap for OI/Volume ratio (default: 100)
/// - w_oi: weight on OI z-score percentile (default: 0.4)
/// - w_ov: weight on OI/Volume z-score percentile (default: 0.4)
/// - w_otm: weight on OTM distance (default: 0.2)
#[napi]
#[allow(clippy::too_many_arguments)]
pub fn options_flow_score(
    contracts: Vec<OptionContract>,
    spot_price: f64,
    top_n: Option<u32>,
    k_otm: Option<f64>,
    min_volume: Option<f64>,
    min_oi: Option<f64>,
    cap_oi_vol: Option<f64>,
    w_oi: Option<f64>,
    w_ov: Option<f64>,
    w_otm: Option<f64>,
) -> Result<Vec<ScoredOption>> {
    let top_n = top_n.unwrap_or(50) as usize;
    let k_otm = k_otm.unwrap_or(2.0);
    let min_volume = min_volume.unwrap_or(10.0);
    let min_oi = min_oi.unwrap_or(100.0);
    let cap_oi_vol = cap_oi_vol.unwrap_or(100.0);
    let w_oi = w_oi.unwrap_or(0.4);
    let w_ov = w_ov.unwrap_or(0.4);
    let w_otm = w_otm.unwrap_or(0.2);

    if contracts.is_empty() {
        return Err(Error::from_reason("Contracts array cannot be empty"));
    }
    if spot_price <= 0.0 {
        return Err(Error::from_reason("Spot price must be positive"));
    }

    // Filter by minimum volume and OI
    let filtered: Vec<(usize, &OptionContract)> = contracts
        .iter()
        .enumerate()
        .filter(|(_, c)| c.volume >= min_volume && c.open_interest >= min_oi && c.dte > 0.0)
        .collect();

    if filtered.is_empty() {
        return Ok(Vec::new());
    }

    // Compute OI/Volume ratio (capped)
    let oi_vol_ratios: Vec<f64> = filtered
        .iter()
        .map(|(_, c)| {
            if c.volume > 0.0 {
                (c.open_interest / c.volume).min(cap_oi_vol)
            } else {
                cap_oi_vol
            }
        })
        .collect();

    // Group by DTE (rounded to integer) for z-score computation
    let oi_values: Vec<f64> = filtered.iter().map(|(_, c)| c.open_interest).collect();
    let dte_keys: Vec<i32> = filtered.iter().map(|(_, c)| c.dte.round() as i32).collect();

    // Compute z-scores within each DTE group
    let oi_z = grouped_z_scores(&oi_values, &dte_keys);
    let ov_z = grouped_z_scores(&oi_vol_ratios, &dte_keys);

    // Compute percentile ranks within each DTE group
    let oi_pct = grouped_percentiles(&oi_z, &dte_keys);
    let ov_pct = grouped_percentiles(&ov_z, &dte_keys);

    // Build scored results
    let mut scored: Vec<ScoredOption> = Vec::with_capacity(filtered.len());

    for (idx, ((orig_idx, c), (((oi_z_val, ov_z_val), oi_p), ov_p))) in filtered
        .iter()
        .zip(
            oi_z.iter()
                .zip(ov_z.iter())
                .zip(oi_pct.iter())
                .zip(ov_pct.iter()),
        )
        .enumerate()
    {
        // OTM distance: exp(-k_otm * |strike - spot| / spot)
        let otm_distance = (c.strike - spot_price).abs() / spot_price;
        let otm_factor = (-k_otm * otm_distance).exp();

        // DTE decay: sqrt(DTE / 365)
        let dte_factor = (c.dte / 365.0).sqrt().min(1.0);

        // Composite score
        let score = w_oi * oi_p * dte_factor + w_ov * ov_p * dte_factor + w_otm * otm_factor;

        scored.push(ScoredOption {
            index: *orig_idx as i32,
            strike: c.strike,
            open_interest: c.open_interest,
            volume: c.volume,
            dte: c.dte,
            side: c.side.clone(),
            implied_volatility: c.implied_volatility,
            oi_volume_ratio: oi_vol_ratios[idx],
            oi_z_score: *oi_z_val,
            ov_z_score: *ov_z_val,
            oi_percentile: *oi_p,
            ov_percentile: *ov_p,
            otm_factor,
            dte_factor,
            score,
        });
    }

    // Sort by score descending
    scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    // Return top N
    scored.truncate(top_n);

    Ok(scored)
}

/// Compute z-scores within groups defined by keys
fn grouped_z_scores(values: &[f64], keys: &[i32]) -> Vec<f64> {
    let mut result = vec![0.0; values.len()];

    // Find unique keys
    let mut unique_keys: Vec<i32> = keys.to_vec();
    unique_keys.sort();
    unique_keys.dedup();

    for key in &unique_keys {
        let indices: Vec<usize> = keys
            .iter()
            .enumerate()
            .filter(|(_, k)| *k == key)
            .map(|(i, _)| i)
            .collect();

        if indices.len() < 2 {
            for &i in &indices {
                result[i] = 0.0;
            }
            continue;
        }

        let group_values: Vec<f64> = indices.iter().map(|&i| values[i]).collect();
        let n = group_values.len() as f64;
        let mean = group_values.iter().sum::<f64>() / n;
        let std = (group_values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n).sqrt();

        for &i in &indices {
            result[i] = if std > 1e-15 {
                (values[i] - mean) / std
            } else {
                0.0
            };
        }
    }

    result
}

/// Compute percentile ranks within groups (0 to 1)
fn grouped_percentiles(values: &[f64], keys: &[i32]) -> Vec<f64> {
    let mut result = vec![0.0; values.len()];

    let mut unique_keys: Vec<i32> = keys.to_vec();
    unique_keys.sort();
    unique_keys.dedup();

    for key in &unique_keys {
        let indices: Vec<usize> = keys
            .iter()
            .enumerate()
            .filter(|(_, k)| *k == key)
            .map(|(i, _)| i)
            .collect();

        if indices.len() <= 1 {
            for &i in &indices {
                result[i] = 0.5;
            }
            continue;
        }

        // Sort indices by value
        let mut sorted_indices = indices.clone();
        sorted_indices.sort_by(|&a, &b| {
            values[a]
                .partial_cmp(&values[b])
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let n = sorted_indices.len() as f64;
        for (rank, &i) in sorted_indices.iter().enumerate() {
            result[i] = rank as f64 / (n - 1.0).max(1.0);
        }
    }

    result
}
