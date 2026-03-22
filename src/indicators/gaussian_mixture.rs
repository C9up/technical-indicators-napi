use napi::bindgen_prelude::*;
use napi_derive::napi;
use rand::rngs::StdRng;
use rand::SeedableRng;
use rand_distr::{Distribution, Uniform};

#[napi(object)]
pub struct GmmCluster {
    /// Cluster index
    pub id: i32,
    /// Mean of each feature dimension
    pub mean: Vec<f64>,
    /// Variance of each feature dimension (diagonal covariance)
    pub variance: Vec<f64>,
    /// Mixing weight (proportion of data in this cluster)
    pub weight: f64,
    /// Number of points assigned to this cluster
    pub count: i32,
}

#[napi(object)]
pub struct GmmResult {
    /// Cluster assignment for each data point
    pub labels: Vec<i32>,
    /// Posterior probabilities: labels.len() * n_components, row-major
    /// probabilities[i * n_components + k] = P(cluster k | point i)
    pub probabilities: Vec<f64>,
    /// Cluster details
    pub clusters: Vec<GmmCluster>,
    /// BIC score (lower = better model fit vs complexity)
    pub bic: f64,
    /// Log-likelihood of the fitted model
    pub log_likelihood: f64,
    /// Number of EM iterations performed
    pub iterations: i32,
}

/// Gaussian Mixture Model (GMM) via Expectation-Maximization
///
/// Clusters multi-dimensional data into n_components Gaussian distributions.
/// Useful for market regime detection (e.g., calm/volatile/transition states).
///
/// Input: a flat array of features, row-major, with `n_features` per row.
/// Example: for returns + volume_change with 100 bars:
///   data = [ret_0, vol_0, ret_1, vol_1, ...] with n_features = 2
///
/// Parameters:
/// - data: flat array of feature values (row-major)
/// - n_features: number of features per observation
/// - n_components: number of Gaussian clusters (default: 3)
/// - max_iterations: max EM iterations (default: 100)
/// - tolerance: convergence threshold on log-likelihood change (default: 1e-6)
/// - normalize: if true, z-score normalize each feature before fitting (default: true)
/// - seed: optional random seed for reproducibility
#[napi]
pub fn gaussian_mixture(
    data: Vec<f64>,
    n_features: u32,
    n_components: Option<u32>,
    max_iterations: Option<u32>,
    tolerance: Option<f64>,
    normalize: Option<bool>,
    seed: Option<u32>,
) -> Result<GmmResult> {
    let n_feat = n_features as usize;
    let k = n_components.unwrap_or(3) as usize;
    let max_iter = max_iterations.unwrap_or(100) as usize;
    let tol = tolerance.unwrap_or(1e-6);
    let do_normalize = normalize.unwrap_or(true);

    if n_feat == 0 {
        return Err(Error::from_reason("n_features must be > 0"));
    }
    if data.len() % n_feat != 0 {
        return Err(Error::from_reason("data length must be divisible by n_features"));
    }

    let n = data.len() / n_feat;
    if n < k {
        return Err(Error::from_reason("Need more data points than components"));
    }

    // Reshape into rows
    let mut rows: Vec<Vec<f64>> = Vec::with_capacity(n);
    for i in 0..n {
        rows.push(data[i * n_feat..(i + 1) * n_feat].to_vec());
    }

    // Optional z-score normalization
    let mut means_norm = vec![0.0; n_feat];
    let mut stds_norm = vec![1.0; n_feat];

    if do_normalize {
        for f in 0..n_feat {
            let vals: Vec<f64> = rows.iter().map(|r| r[f]).collect();
            let mean = vals.iter().sum::<f64>() / n as f64;
            let std = (vals.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n as f64).sqrt();
            means_norm[f] = mean;
            stds_norm[f] = if std > 1e-15 { std } else { 1.0 };
        }

        for row in &mut rows {
            for f in 0..n_feat {
                row[f] = (row[f] - means_norm[f]) / stds_norm[f];
            }
        }
    }

    // --- Initialize GMM parameters ---
    let mut rng = match seed {
        Some(s) => StdRng::seed_from_u64(s as u64),
        None => StdRng::from_entropy(),
    };

    // K-means++ style initialization for means
    let mut cluster_means: Vec<Vec<f64>> = Vec::with_capacity(k);

    // Pick first center randomly
    let first_idx = Uniform::new(0, n).sample(&mut rng);
    cluster_means.push(rows[first_idx].clone());

    // Pick remaining centers with probability proportional to distance
    for _ in 1..k {
        let mut dists: Vec<f64> = rows
            .iter()
            .map(|row| {
                cluster_means
                    .iter()
                    .map(|c| sq_dist(row, c))
                    .fold(f64::INFINITY, f64::min)
            })
            .collect();

        let total: f64 = dists.iter().sum();
        if total > 0.0 {
            for d in &mut dists {
                *d /= total;
            }
        }

        // Weighted random selection
        let r: f64 = Uniform::new(0.0, 1.0).sample(&mut rng);
        let mut cum = 0.0;
        let mut chosen = 0;
        for (i, &d) in dists.iter().enumerate() {
            cum += d;
            if cum >= r {
                chosen = i;
                break;
            }
        }
        cluster_means.push(rows[chosen].clone());
    }

    // Initialize variances (global variance per feature)
    let mut cluster_vars: Vec<Vec<f64>> = vec![vec![1.0; n_feat]; k];
    for kk in 0..k {
        for f in 0..n_feat {
            let global_var: f64 = rows.iter().map(|r| (r[f] - cluster_means[kk][f]).powi(2)).sum::<f64>() / n as f64;
            cluster_vars[kk][f] = global_var.max(1e-6);
        }
    }

    // Initialize weights uniformly
    let mut weights = vec![1.0 / k as f64; k];

    // Responsibility matrix: n x k
    let mut resp = vec![vec![0.0; k]; n];

    // --- EM iterations ---
    let mut prev_ll = f64::NEG_INFINITY;
    let mut iterations = 0;

    for iter in 0..max_iter {
        iterations = iter + 1;

        // E-step: compute responsibilities
        for i in 0..n {
            let mut log_probs = vec![0.0; k];
            let mut max_log = f64::NEG_INFINITY;

            for kk in 0..k {
                log_probs[kk] = weights[kk].ln() + log_gaussian(&rows[i], &cluster_means[kk], &cluster_vars[kk]);
                if log_probs[kk] > max_log {
                    max_log = log_probs[kk];
                }
            }

            // Log-sum-exp for numerical stability
            let mut sum_exp = 0.0;
            for kk in 0..k {
                resp[i][kk] = (log_probs[kk] - max_log).exp();
                sum_exp += resp[i][kk];
            }

            if sum_exp > 0.0 {
                for val in resp[i].iter_mut().take(k) {
                    *val /= sum_exp;
                }
            }
        }

        // M-step: update parameters
        for kk in 0..k {
            let nk: f64 = resp.iter().map(|r| r[kk]).sum();

            if nk < 1e-10 {
                continue;
            }

            // Update weight
            weights[kk] = nk / n as f64;

            // Update mean
            for f in 0..n_feat {
                cluster_means[kk][f] = resp.iter().enumerate().map(|(i, r)| r[kk] * rows[i][f]).sum::<f64>() / nk;
            }

            // Update variance
            for f in 0..n_feat {
                let var: f64 = resp
                    .iter()
                    .enumerate()
                    .map(|(i, r)| r[kk] * (rows[i][f] - cluster_means[kk][f]).powi(2))
                    .sum::<f64>()
                    / nk;
                cluster_vars[kk][f] = var.max(1e-6); // floor to prevent degenerate clusters
            }
        }

        // Compute log-likelihood
        let ll = log_likelihood(&rows, &cluster_means, &cluster_vars, &weights);

        if (ll - prev_ll).abs() < tol {
            break;
        }
        prev_ll = ll;
    }

    // --- Build output ---
    let final_ll = log_likelihood(&rows, &cluster_means, &cluster_vars, &weights);

    // Labels = argmax of responsibilities
    let mut labels = Vec::with_capacity(n);
    let mut flat_probs = Vec::with_capacity(n * k);

    for row_resp in &resp {
        let mut best_k = 0;
        let mut best_p = row_resp[0];
        for (kk, &p) in row_resp.iter().enumerate().skip(1) {
            if p > best_p {
                best_p = p;
                best_k = kk;
            }
        }
        labels.push(best_k as i32);

        for &p in row_resp.iter().take(k) {
            flat_probs.push(p);
        }
    }

    // BIC = -2*LL + p*ln(n)
    // p = k*(n_feat + n_feat + 1) - 1 = k*(2*n_feat + 1) - 1
    let num_params = k * (2 * n_feat + 1) - 1;
    let bic = -2.0 * final_ll + num_params as f64 * (n as f64).ln();

    // Cluster details (denormalize means/variances if normalized)
    let mut clusters = Vec::with_capacity(k);
    for kk in 0..k {
        let count = labels.iter().filter(|&&l| l == kk as i32).count() as i32;

        let mut mean = cluster_means[kk].clone();
        let mut variance = cluster_vars[kk].clone();

        if do_normalize {
            for f in 0..n_feat {
                mean[f] = mean[f] * stds_norm[f] + means_norm[f];
                variance[f] *= stds_norm[f] * stds_norm[f];
            }
        }

        clusters.push(GmmCluster {
            id: kk as i32,
            mean,
            variance,
            weight: weights[kk],
            count,
        });
    }

    Ok(GmmResult {
        labels,
        probabilities: flat_probs,
        clusters,
        bic,
        log_likelihood: final_ll,
        iterations: iterations as i32,
    })
}

/// Log probability density of a diagonal Gaussian
fn log_gaussian(x: &[f64], mean: &[f64], var: &[f64]) -> f64 {
    let d = x.len() as f64;
    let mut log_p = -0.5 * d * (2.0 * std::f64::consts::PI).ln();

    for i in 0..x.len() {
        log_p -= 0.5 * var[i].ln();
        log_p -= 0.5 * (x[i] - mean[i]).powi(2) / var[i];
    }

    log_p
}

/// Total log-likelihood of the mixture model
fn log_likelihood(
    rows: &[Vec<f64>],
    means: &[Vec<f64>],
    vars: &[Vec<f64>],
    weights: &[f64],
) -> f64 {
    let k = means.len();
    let mut ll = 0.0;

    for row in rows {
        let mut log_probs = Vec::with_capacity(k);
        let mut max_log = f64::NEG_INFINITY;

        for kk in 0..k {
            let lp = weights[kk].ln() + log_gaussian(row, &means[kk], &vars[kk]);
            if lp > max_log {
                max_log = lp;
            }
            log_probs.push(lp);
        }

        let sum_exp: f64 = log_probs.iter().map(|lp| (lp - max_log).exp()).sum();
        ll += max_log + sum_exp.ln();
    }

    ll
}

/// Squared Euclidean distance
fn sq_dist(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter()).map(|(ai, bi)| (ai - bi).powi(2)).sum()
}
