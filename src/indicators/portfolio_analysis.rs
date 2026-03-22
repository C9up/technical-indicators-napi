use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi(object)]
pub struct PortfolioStats {
    /// Expected daily return of the portfolio
    pub expected_return_daily: f64,
    /// Expected annualized return
    pub expected_return_annual: f64,
    /// Daily portfolio volatility (std dev)
    pub volatility_daily: f64,
    /// Annualized portfolio volatility
    pub volatility_annual: f64,
    /// Daily portfolio variance
    pub variance_daily: f64,
    /// Sharpe ratio (annualized)
    pub sharpe_ratio: f64,
}

#[napi(object)]
pub struct CovarianceResult {
    /// Covariance matrix (flat, row-major, n_assets x n_assets)
    pub covariance: Vec<f64>,
    /// Correlation matrix (flat, row-major, n_assets x n_assets)
    pub correlation: Vec<f64>,
    /// Mean daily return per asset
    pub mean_returns: Vec<f64>,
    /// Annualized volatility per asset
    pub volatilities: Vec<f64>,
    /// Number of assets
    pub n_assets: i32,
}

#[napi(object)]
pub struct EfficientFrontierPoint {
    /// Target return (annualized)
    pub target_return: f64,
    /// Portfolio volatility at this point (annualized)
    pub volatility: f64,
    /// Optimal weights for this point
    pub weights: Vec<f64>,
    /// Sharpe ratio at this point
    pub sharpe_ratio: f64,
}

#[napi(object)]
pub struct EfficientFrontierResult {
    /// Points along the efficient frontier
    pub frontier: Vec<EfficientFrontierPoint>,
    /// Global Minimum Variance Portfolio
    pub gmvp: EfficientFrontierPoint,
    /// Maximum Sharpe Ratio (tangency) portfolio
    pub max_sharpe: EfficientFrontierPoint,
}

/// Compute covariance matrix, correlation matrix, and per-asset stats
/// from multiple return series.
///
/// Input: flat array of returns, row-major, with n_assets per row.
/// Each row = one time period, columns = assets.
/// Example: [ret_a_0, ret_b_0, ret_c_0, ret_a_1, ret_b_1, ret_c_1, ...]
#[napi]
pub fn covariance_matrix(
    returns_flat: Vec<f64>,
    n_assets: u32,
) -> Result<CovarianceResult> {
    let k = n_assets as usize;
    if k < 2 {
        return Err(Error::from_reason("Need at least 2 assets"));
    }
    if returns_flat.len() % k != 0 {
        return Err(Error::from_reason("returns length must be divisible by n_assets"));
    }

    let n = returns_flat.len() / k; // number of periods
    if n < 3 {
        return Err(Error::from_reason("Need at least 3 periods"));
    }

    // Compute means
    let mut means = vec![0.0; k];
    for t in 0..n {
        for j in 0..k {
            means[j] += returns_flat[t * k + j];
        }
    }
    for mean in means.iter_mut().take(k) {
        *mean /= n as f64;
    }

    // Covariance matrix (sample covariance, ddof=1)
    let mut cov = vec![0.0; k * k];
    for t in 0..n {
        for i in 0..k {
            let di = returns_flat[t * k + i] - means[i];
            for j in 0..k {
                let dj = returns_flat[t * k + j] - means[j];
                cov[i * k + j] += di * dj;
            }
        }
    }
    for v in &mut cov {
        *v /= (n - 1) as f64;
    }

    // Correlation matrix
    let mut stds = vec![0.0; k];
    for i in 0..k {
        stds[i] = cov[i * k + i].sqrt();
    }

    let mut corr = vec![0.0; k * k];
    for i in 0..k {
        for j in 0..k {
            corr[i * k + j] = if stds[i] > 1e-15 && stds[j] > 1e-15 {
                cov[i * k + j] / (stds[i] * stds[j])
            } else {
                0.0
            };
        }
    }

    // Annualized volatilities
    let vols: Vec<f64> = stds.iter().map(|s| s * 252.0_f64.sqrt()).collect();

    Ok(CovarianceResult {
        covariance: cov,
        correlation: corr,
        mean_returns: means,
        volatilities: vols,
        n_assets: k as i32,
    })
}

/// Compute portfolio return and risk for given weights.
///
/// - returns_flat: flat return series (row-major, n_assets per row)
/// - n_assets: number of assets
/// - weights: portfolio weights (must sum to ~1)
/// - risk_free_rate: annualized (default: 0.02)
#[napi]
pub fn portfolio_stats(
    returns_flat: Vec<f64>,
    n_assets: u32,
    weights: Vec<f64>,
    risk_free_rate: Option<f64>,
) -> Result<PortfolioStats> {
    let k = n_assets as usize;
    let rf = risk_free_rate.unwrap_or(0.02);

    if weights.len() != k {
        return Err(Error::from_reason("Weights length must equal n_assets"));
    }

    let cov_result = covariance_matrix(returns_flat, n_assets)?;
    let means = &cov_result.mean_returns;
    let cov = &cov_result.covariance;

    // Portfolio expected daily return: w' * r
    let exp_ret: f64 = weights.iter().zip(means.iter()).map(|(w, r)| w * r).sum();

    // Portfolio variance: w' * Σ * w
    let mut var = 0.0;
    for i in 0..k {
        for j in 0..k {
            var += weights[i] * weights[j] * cov[i * k + j];
        }
    }

    let vol_daily = var.max(0.0).sqrt();
    let vol_annual = vol_daily * 252.0_f64.sqrt();
    let ret_annual = exp_ret * 252.0;
    let sharpe = if vol_annual > 1e-15 { (ret_annual - rf) / vol_annual } else { 0.0 };

    Ok(PortfolioStats {
        expected_return_daily: exp_ret,
        expected_return_annual: ret_annual,
        volatility_daily: vol_daily,
        volatility_annual: vol_annual,
        variance_daily: var,
        sharpe_ratio: sharpe,
    })
}

/// Compute the efficient frontier using the analytical Markowitz solution.
///
/// - returns_flat: flat return series (row-major, n_assets per row)
/// - n_assets: number of assets
/// - n_points: number of points on the frontier (default: 50)
/// - risk_free_rate: annualized (default: 0.02)
#[napi]
pub fn efficient_frontier(
    returns_flat: Vec<f64>,
    n_assets: u32,
    n_points: Option<u32>,
    risk_free_rate: Option<f64>,
) -> Result<EfficientFrontierResult> {
    let k = n_assets as usize;
    let num_points = n_points.unwrap_or(50) as usize;
    let rf = risk_free_rate.unwrap_or(0.02);

    if k < 2 {
        return Err(Error::from_reason("Need at least 2 assets"));
    }

    let cov_result = covariance_matrix(returns_flat, n_assets)?;
    let r = &cov_result.mean_returns;
    let sigma = &cov_result.covariance;

    // Invert covariance matrix
    let sigma_inv = invert_matrix(sigma, k)?;

    let ones = vec![1.0; k];

    // A = 1' Σ⁻¹ r
    let sigma_inv_r = mat_vec_mult(&sigma_inv, r, k);
    let sigma_inv_ones = mat_vec_mult(&sigma_inv, &ones, k);

    let a: f64 = ones.iter().zip(sigma_inv_r.iter()).map(|(o, s)| o * s).sum();
    let b: f64 = r.iter().zip(sigma_inv_r.iter()).map(|(ri, s)| ri * s).sum();
    let c: f64 = ones.iter().zip(sigma_inv_ones.iter()).map(|(o, s)| o * s).sum();
    let delta = b * c - a * a;

    if delta.abs() < 1e-15 {
        return Err(Error::from_reason("Degenerate covariance matrix (delta = 0)"));
    }

    // GMVP: return = A/C, weights = Σ⁻¹ * 1 / C
    let mu_gmvp = a / c;
    let w_gmvp: Vec<f64> = sigma_inv_ones.iter().map(|s| s / c).collect();
    let var_gmvp = 1.0 / c;
    let vol_gmvp = var_gmvp.max(0.0).sqrt();

    let mu_gmvp_annual = mu_gmvp * 252.0;
    let vol_gmvp_annual = vol_gmvp * 252.0_f64.sqrt();
    let sharpe_gmvp = if vol_gmvp_annual > 1e-15 { (mu_gmvp_annual - rf) / vol_gmvp_annual } else { 0.0 };

    let gmvp = EfficientFrontierPoint {
        target_return: mu_gmvp_annual,
        volatility: vol_gmvp_annual,
        weights: w_gmvp,
        sharpe_ratio: sharpe_gmvp,
    };

    // Generate frontier points
    let max_ret = r.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let min_mu = mu_gmvp;
    let max_mu = max_ret * 1.5;

    let mut frontier = Vec::with_capacity(num_points);
    let mut best_sharpe = f64::NEG_INFINITY;
    let mut max_sharpe_point = gmvp.clone_point();

    for i in 0..num_points {
        let mu_p = min_mu + (max_mu - min_mu) * i as f64 / (num_points - 1).max(1) as f64;

        let lambda1 = (c * mu_p - a) / delta;
        let lambda2 = (b - a * mu_p) / delta;

        let weights: Vec<f64> = (0..k)
            .map(|j| lambda1 * sigma_inv_r[j] + lambda2 * sigma_inv_ones[j])
            .collect();

        let var_p = (c * mu_p * mu_p - 2.0 * a * mu_p + b) / delta;
        let vol_p = var_p.max(0.0).sqrt();

        let mu_annual = mu_p * 252.0;
        let vol_annual = vol_p * 252.0_f64.sqrt();
        let sharpe = if vol_annual > 1e-15 { (mu_annual - rf) / vol_annual } else { 0.0 };

        if sharpe > best_sharpe {
            best_sharpe = sharpe;
            max_sharpe_point = EfficientFrontierPoint {
                target_return: mu_annual,
                volatility: vol_annual,
                weights: weights.clone(),
                sharpe_ratio: sharpe,
            };
        }

        frontier.push(EfficientFrontierPoint {
            target_return: mu_annual,
            volatility: vol_annual,
            weights,
            sharpe_ratio: sharpe,
        });
    }

    Ok(EfficientFrontierResult {
        frontier,
        gmvp,
        max_sharpe: max_sharpe_point,
    })
}

/// Matrix-vector multiplication: result = M * v
fn mat_vec_mult(m: &[f64], v: &[f64], n: usize) -> Vec<f64> {
    let mut result = vec![0.0; n];
    for i in 0..n {
        for j in 0..n {
            result[i] += m[i * n + j] * v[j];
        }
    }
    result
}

/// Invert a square matrix using Gauss-Jordan elimination
fn invert_matrix(m: &[f64], n: usize) -> Result<Vec<f64>> {
    // Augmented matrix [M | I]
    let mut aug = vec![0.0; n * 2 * n];
    for i in 0..n {
        for j in 0..n {
            aug[i * 2 * n + j] = m[i * n + j];
        }
        aug[i * 2 * n + n + i] = 1.0; // identity
    }

    // Forward elimination with partial pivoting
    for col in 0..n {
        let mut max_row = col;
        let mut max_val = aug[col * 2 * n + col].abs();
        for row in (col + 1)..n {
            let val = aug[row * 2 * n + col].abs();
            if val > max_val {
                max_val = val;
                max_row = row;
            }
        }

        if max_val < 1e-12 {
            return Err(Error::from_reason("Singular covariance matrix, cannot invert"));
        }

        // Swap rows
        if max_row != col {
            for j in 0..(2 * n) {
                aug.swap(col * 2 * n + j, max_row * 2 * n + j);
            }
        }

        let pivot = aug[col * 2 * n + col];
        for j in 0..(2 * n) {
            aug[col * 2 * n + j] /= pivot;
        }

        for row in 0..n {
            if row == col {
                continue;
            }
            let factor = aug[row * 2 * n + col];
            for j in 0..(2 * n) {
                aug[row * 2 * n + j] -= factor * aug[col * 2 * n + j];
            }
        }
    }

    // Extract inverse
    let mut inv = vec![0.0; n * n];
    for i in 0..n {
        for j in 0..n {
            inv[i * n + j] = aug[i * 2 * n + n + j];
        }
    }

    Ok(inv)
}

// Helper trait to clone frontier points (no derive Clone on napi objects)
trait ClonePoint {
    fn clone_point(&self) -> Self;
}

impl ClonePoint for EfficientFrontierPoint {
    fn clone_point(&self) -> Self {
        EfficientFrontierPoint {
            target_return: self.target_return,
            volatility: self.volatility,
            weights: self.weights.clone(),
            sharpe_ratio: self.sharpe_ratio,
        }
    }
}
