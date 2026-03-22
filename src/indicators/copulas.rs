use napi::bindgen_prelude::*;
use napi_derive::napi;
use crate::helpers::copula_helpers::quantile_transform;
use crate::helpers::copula_helpers::gaussian_copula;
use crate::helpers::copula_helpers::clayton_copula;
use crate::helpers::copula_helpers::gumbel_copula;
use crate::helpers::copula_helpers::frank_copula;

#[napi(object)]
pub struct CopulaSample {
    pub u: Vec<f64>,
    pub v: Vec<f64>,
}

#[napi(object)]
pub struct CopulaFitResult {
    pub copula_type: String,
    pub parameter: f64,
    pub log_likelihood: f64,
}

#[napi(object)]
pub struct ScenarioResult {
    pub ticker: String,
    pub mean_return: f64,
    pub worst_case: f64,
    pub best_case: f64,
    pub simulated_returns: Vec<f64>,
}

// --- Quantile Transform ---

#[napi]
pub fn quantile_transform(data: Vec<f64>) -> Result<Vec<f64>> {
    if data.is_empty() {
        return Err(Error::from_reason("Data array cannot be empty"));
    }
    Ok(quantile_transform::ranks_to_uniform(&data))
}

// --- Gaussian Copula ---

#[napi]
pub fn gaussian_copula_sample(
    rho: f64,
    n_samples: u32,
    seed: Option<u32>,
) -> Result<CopulaSample> {
    if !(-1.0..=1.0).contains(&rho) {
        return Err(Error::from_reason("rho must be between -1 and 1"));
    }
    if n_samples == 0 {
        return Err(Error::from_reason("n_samples must be greater than 0"));
    }

    let samples = gaussian_copula::gaussian_copula_sample(
        rho,
        n_samples as usize,
        seed.map(|s| s as u64),
    );

    let (u, v): (Vec<f64>, Vec<f64>) = samples.into_iter().unzip();
    Ok(CopulaSample { u, v })
}

#[napi]
pub fn gaussian_conditional_sample(
    u1: f64,
    rho: f64,
    n_samples: u32,
    seed: Option<u32>,
) -> Result<CopulaSample> {
    if !(0.0..=1.0).contains(&u1) {
        return Err(Error::from_reason("u1 must be between 0 and 1"));
    }
    if !(-1.0..=1.0).contains(&rho) {
        return Err(Error::from_reason("rho must be between -1 and 1"));
    }
    if n_samples == 0 {
        return Err(Error::from_reason("n_samples must be greater than 0"));
    }

    let samples = gaussian_copula::gaussian_conditional_sample(
        u1,
        rho,
        n_samples as usize,
        seed.map(|s| s as u64),
    );

    let (u, v): (Vec<f64>, Vec<f64>) = samples.into_iter().unzip();
    Ok(CopulaSample { u, v })
}

// --- Clayton Copula ---

#[napi]
pub fn clayton_copula_sample(
    theta: f64,
    n_samples: u32,
    seed: Option<u32>,
) -> Result<CopulaSample> {
    if theta <= 0.0 {
        return Err(Error::from_reason("theta must be greater than 0 for Clayton copula"));
    }
    if n_samples == 0 {
        return Err(Error::from_reason("n_samples must be greater than 0"));
    }

    let samples = clayton_copula::clayton_copula_sample(
        theta,
        n_samples as usize,
        seed.map(|s| s as u64),
    );

    let (u, v): (Vec<f64>, Vec<f64>) = samples.into_iter().unzip();
    Ok(CopulaSample { u, v })
}

// --- Gumbel Copula ---

#[napi]
pub fn gumbel_copula_sample(
    theta: f64,
    n_samples: u32,
    seed: Option<u32>,
) -> Result<CopulaSample> {
    if theta < 1.0 {
        return Err(Error::from_reason("theta must be >= 1 for Gumbel copula"));
    }
    if n_samples == 0 {
        return Err(Error::from_reason("n_samples must be greater than 0"));
    }

    let samples = gumbel_copula::gumbel_copula_sample(
        theta,
        n_samples as usize,
        seed.map(|s| s as u64),
    );

    let (u, v): (Vec<f64>, Vec<f64>) = samples.into_iter().unzip();
    Ok(CopulaSample { u, v })
}

// --- Frank Copula ---

#[napi]
pub fn frank_copula_sample(
    theta: f64,
    n_samples: u32,
    seed: Option<u32>,
) -> Result<CopulaSample> {
    if n_samples == 0 {
        return Err(Error::from_reason("n_samples must be greater than 0"));
    }

    let samples = frank_copula::frank_copula_sample(
        theta,
        n_samples as usize,
        seed.map(|s| s as u64),
    );

    let (u, v): (Vec<f64>, Vec<f64>) = samples.into_iter().unzip();
    Ok(CopulaSample { u, v })
}

// --- Fit Copula ---

#[napi]
pub fn fit_copula(
    u: Vec<f64>,
    v: Vec<f64>,
    copula_type: String,
) -> Result<CopulaFitResult> {
    if u.len() != v.len() {
        return Err(Error::from_reason("u and v must have the same length"));
    }
    if u.len() < 3 {
        return Err(Error::from_reason("Need at least 3 data points"));
    }

    match copula_type.to_lowercase().as_str() {
        "gaussian" => {
            let (param, ll) = gaussian_copula::fit_gaussian_copula(&u, &v)
                .map_err(Error::from_reason)?;
            Ok(CopulaFitResult {
                copula_type: "gaussian".to_string(),
                parameter: param,
                log_likelihood: ll,
            })
        }
        "clayton" => {
            let (param, ll) = clayton_copula::fit_clayton_copula(&u, &v)
                .map_err(Error::from_reason)?;
            Ok(CopulaFitResult {
                copula_type: "clayton".to_string(),
                parameter: param,
                log_likelihood: ll,
            })
        }
        "gumbel" => {
            let (param, ll) = gumbel_copula::fit_gumbel_copula(&u, &v)
                .map_err(Error::from_reason)?;
            Ok(CopulaFitResult {
                copula_type: "gumbel".to_string(),
                parameter: param,
                log_likelihood: ll,
            })
        }
        "frank" => {
            let (param, ll) = frank_copula::fit_frank_copula(&u, &v)
                .map_err(Error::from_reason)?;
            Ok(CopulaFitResult {
                copula_type: "frank".to_string(),
                parameter: param,
                log_likelihood: ll,
            })
        }
        _ => Err(Error::from_reason(format!(
            "Unknown copula type: '{}'. Supported: gaussian, clayton, gumbel, frank",
            copula_type
        ))),
    }
}

// --- Portfolio Scenario ---

#[napi]
pub fn portfolio_scenario(
    returns_data: Vec<Vec<f64>>,
    market_drop: f64,
    copula_type: Option<String>,
    n_simulations: Option<u32>,
) -> Result<Vec<ScenarioResult>> {
    if returns_data.len() < 2 {
        return Err(Error::from_reason("Need at least 2 return series (market + 1 asset)"));
    }

    let market_returns = &returns_data[0];
    let n_sims = n_simulations.unwrap_or(1000) as usize;
    let cop_type = copula_type.unwrap_or_else(|| "gaussian".to_string());

    // Transform market returns to uniform
    let market_uniform = quantile_transform::ranks_to_uniform(market_returns);

    // Find the uniform quantile for the market drop
    let mut sorted_market: Vec<f64> = market_returns.to_vec();
    sorted_market.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let market_drop_quantile = empirical_cdf(&sorted_market, market_drop);

    let mut results = Vec::new();

    for (idx, asset_returns) in returns_data.iter().enumerate().skip(1) {
        if asset_returns.len() != market_returns.len() {
            return Err(Error::from_reason(format!(
                "Asset {} has different length than market returns",
                idx
            )));
        }

        let asset_uniform = quantile_transform::ranks_to_uniform(asset_returns);

        // Fit copula between market and asset
        let param = match cop_type.as_str() {
            "gaussian" => {
                gaussian_copula::fit_gaussian_copula(&market_uniform, &asset_uniform)
                    .map(|(p, _)| p)
                    .unwrap_or(0.5)
            }
            "clayton" => {
                clayton_copula::fit_clayton_copula(&market_uniform, &asset_uniform)
                    .map(|(p, _)| p)
                    .unwrap_or(2.0)
            }
            "gumbel" => {
                gumbel_copula::fit_gumbel_copula(&market_uniform, &asset_uniform)
                    .map(|(p, _)| p)
                    .unwrap_or(1.5)
            }
            "frank" => {
                frank_copula::fit_frank_copula(&market_uniform, &asset_uniform)
                    .map(|(p, _)| p)
                    .unwrap_or(5.0)
            }
            _ => return Err(Error::from_reason(format!("Unknown copula type: '{}'", cop_type))),
        };

        // Conditional sampling: given market drop, simulate asset returns
        let conditional_samples = match cop_type.as_str() {
            "gaussian" => gaussian_copula::gaussian_conditional_sample(
                market_drop_quantile, param, n_sims, None,
            ),
            "clayton" => clayton_copula::clayton_conditional_sample(
                market_drop_quantile, param, n_sims, None,
            ),
            "gumbel" => gumbel_copula::gumbel_conditional_sample(
                market_drop_quantile, param, n_sims, None,
            ),
            "frank" => frank_copula::frank_conditional_sample(
                market_drop_quantile, param, n_sims, None,
            ),
            _ => unreachable!(),
        };

        // Transform simulated uniform values back to original return distribution
        let mut sorted_asset: Vec<f64> = asset_returns.to_vec();
        sorted_asset.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let simulated_u2: Vec<f64> = conditional_samples.iter().map(|(_, u2)| *u2).collect();
        let simulated_returns =
            quantile_transform::uniform_to_original(&simulated_u2, &sorted_asset);

        // Compute statistics
        let mut sorted_sim = simulated_returns.clone();
        sorted_sim.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let mean = sorted_sim.iter().sum::<f64>() / sorted_sim.len() as f64;
        let worst = percentile(&sorted_sim, 5.0);
        let best = percentile(&sorted_sim, 95.0);

        results.push(ScenarioResult {
            ticker: format!("asset_{}", idx),
            mean_return: mean,
            worst_case: worst,
            best_case: best,
            simulated_returns,
        });
    }

    Ok(results)
}

/// Empirical CDF: fraction of sorted_data <= value
fn empirical_cdf(sorted_data: &[f64], value: f64) -> f64 {
    let count = sorted_data.iter().filter(|&&x| x <= value).count();
    (count as f64 / (sorted_data.len() as f64 + 1.0)).clamp(1e-10, 1.0 - 1e-10)
}

/// Percentile from sorted data using linear interpolation
fn percentile(sorted_data: &[f64], p: f64) -> f64 {
    if sorted_data.is_empty() {
        return 0.0;
    }
    let n = sorted_data.len();
    let idx = p / 100.0 * (n as f64 - 1.0);
    let lo = idx.floor() as usize;
    let hi = (lo + 1).min(n - 1);
    let frac = idx - lo as f64;
    sorted_data[lo] * (1.0 - frac) + sorted_data[hi] * frac
}
