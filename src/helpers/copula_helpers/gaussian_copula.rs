use statrs::distribution::{ContinuousCDF, Normal};
use rand::rngs::StdRng;
use rand::SeedableRng;
use rand_distr::{Distribution, StandardNormal};

/// Standard normal distribution for CDF/inverse CDF
fn std_normal() -> Normal {
    Normal::new(0.0, 1.0).unwrap()
}

/// Bivariate Gaussian copula CDF: C(u,v) = Φ_ρ(Φ⁻¹(u), Φ⁻¹(v))
/// Approximated using the conditional method
pub fn gaussian_copula_cdf(u: f64, v: f64, rho: f64) -> f64 {
    let norm = std_normal();
    let x = norm.inverse_cdf(u.clamp(1e-10, 1.0 - 1e-10));
    let y = norm.inverse_cdf(v.clamp(1e-10, 1.0 - 1e-10));

    // Bivariate normal CDF approximation using conditional
    // P(X<x, Y<y) = integral P(Y<y|X=t) * phi(t) dt
    // For computation, use the identity:
    // Φ_ρ(x,y) = Φ(x) * Φ((y - ρx) / sqrt(1-ρ²))  [approximation for conditional]
    // This is an approximation; for exact, we'd need numerical integration
    let conditional_y = (y - rho * x) / (1.0 - rho * rho).sqrt();
    norm.cdf(x) * norm.cdf(conditional_y)
}

/// Conditional CDF: C(v|u) = Φ((Φ⁻¹(v) - ρ·Φ⁻¹(u)) / √(1-ρ²))
pub fn gaussian_conditional_cdf(u: f64, v: f64, rho: f64) -> f64 {
    let norm = std_normal();
    let x = norm.inverse_cdf(u.clamp(1e-10, 1.0 - 1e-10));
    let y = norm.inverse_cdf(v.clamp(1e-10, 1.0 - 1e-10));

    let z = (y - rho * x) / (1.0 - rho * rho).sqrt();
    norm.cdf(z)
}

/// Sample from bivariate Gaussian copula using Cholesky decomposition
/// Returns Vec of (u, v) pairs in [0,1]²
pub fn gaussian_copula_sample(rho: f64, n_samples: usize, seed: Option<u64>) -> Vec<(f64, f64)> {
    let norm = std_normal();
    let mut rng = match seed {
        Some(s) => StdRng::seed_from_u64(s),
        None => StdRng::from_entropy(),
    };

    let mut samples = Vec::with_capacity(n_samples);

    for _ in 0..n_samples {
        // Generate independent standard normals
        let z1: f64 = StandardNormal.sample(&mut rng);
        let z2: f64 = StandardNormal.sample(&mut rng);

        // Apply Cholesky: x1 = z1, x2 = rho*z1 + sqrt(1-rho²)*z2
        let x1 = z1;
        let x2 = rho * z1 + (1.0 - rho * rho).sqrt() * z2;

        // Transform to uniform via normal CDF
        let u = norm.cdf(x1);
        let v = norm.cdf(x2);

        samples.push((u, v));
    }

    samples
}

/// Conditional sampling: given u1, sample u2 from C(·|u1)
/// Uses: u2 = Φ(ρ·Φ⁻¹(u1) + √(1-ρ²)·z) where z ~ N(0,1)
pub fn gaussian_conditional_sample(
    u1: f64,
    rho: f64,
    n_samples: usize,
    seed: Option<u64>,
) -> Vec<(f64, f64)> {
    let norm = std_normal();
    let mut rng = match seed {
        Some(s) => StdRng::seed_from_u64(s),
        None => StdRng::from_entropy(),
    };

    let x1 = norm.inverse_cdf(u1.clamp(1e-10, 1.0 - 1e-10));
    let mut samples = Vec::with_capacity(n_samples);

    for _ in 0..n_samples {
        let z: f64 = StandardNormal.sample(&mut rng);
        let x2 = rho * x1 + (1.0 - rho * rho).sqrt() * z;
        let u2 = norm.cdf(x2);
        samples.push((u1, u2));
    }

    samples
}

/// Fit Gaussian copula parameter (rho) via maximum likelihood
/// Maximizes: sum(log(c(u_i, v_i; rho))) where c is the copula density
/// Uses grid search + refinement for robustness
pub fn fit_gaussian_copula(u: &[f64], v: &[f64]) -> Result<(f64, f64), String> {
    if u.len() != v.len() {
        return Err("u and v must have the same length".to_string());
    }
    if u.len() < 3 {
        return Err("Need at least 3 data points".to_string());
    }

    let norm = std_normal();

    // Transform to normal space
    let x: Vec<f64> = u
        .iter()
        .map(|&ui| norm.inverse_cdf(ui.clamp(1e-10, 1.0 - 1e-10)))
        .collect();
    let y: Vec<f64> = v
        .iter()
        .map(|&vi| norm.inverse_cdf(vi.clamp(1e-10, 1.0 - 1e-10)))
        .collect();

    // MLE for rho: sample correlation of transformed data
    let n = x.len() as f64;
    let mean_x: f64 = x.iter().sum::<f64>() / n;
    let mean_y: f64 = y.iter().sum::<f64>() / n;

    let mut cov_xy = 0.0;
    let mut var_x = 0.0;
    let mut var_y = 0.0;

    for i in 0..x.len() {
        let dx = x[i] - mean_x;
        let dy = y[i] - mean_y;
        cov_xy += dx * dy;
        var_x += dx * dx;
        var_y += dy * dy;
    }

    if var_x == 0.0 || var_y == 0.0 {
        return Err("Zero variance in data".to_string());
    }

    let rho = cov_xy / (var_x * var_y).sqrt();
    let rho = rho.clamp(-0.999, 0.999);

    // Compute log-likelihood at estimated rho
    let ll = gaussian_log_likelihood(&x, &y, rho);

    Ok((rho, ll))
}

/// Log-likelihood of bivariate Gaussian copula density
fn gaussian_log_likelihood(x: &[f64], y: &[f64], rho: f64) -> f64 {
    let rho2 = rho * rho;
    let denom = 1.0 - rho2;

    if denom <= 0.0 {
        return f64::NEG_INFINITY;
    }

    let n = x.len() as f64;
    let mut sum = 0.0;

    for i in 0..x.len() {
        sum += (x[i] * x[i] + y[i] * y[i] - 2.0 * rho * x[i] * y[i]) / denom;
    }

    // Log copula density: -0.5*n*ln(1-rho²) - 0.5*(sum/denom - sum_x² - sum_y²)
    // Simplified: -0.5*n*ln(1-rho²) - (rho²*(x²+y²) - 2*rho*x*y) / (2*denom)
    -0.5 * n * denom.ln() - 0.5 * sum
        + 0.5 * x.iter().map(|xi| xi * xi).sum::<f64>()
        + 0.5 * y.iter().map(|yi| yi * yi).sum::<f64>()
}
