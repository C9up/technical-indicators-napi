use rand::rngs::StdRng;
use rand::SeedableRng;
use rand_distr::{Distribution, Uniform};

/// Clayton copula CDF: C(u,v) = (u⁻ᶿ + v⁻ᶿ - 1)⁻¹/ᶿ
/// theta > 0 for positive dependence
pub fn clayton_copula_cdf(u: f64, v: f64, theta: f64) -> f64 {
    let u = u.clamp(1e-10, 1.0 - 1e-10);
    let v = v.clamp(1e-10, 1.0 - 1e-10);

    let val = u.powf(-theta) + v.powf(-theta) - 1.0;
    if val <= 0.0 {
        return 0.0;
    }
    val.powf(-1.0 / theta)
}

/// Conditional CDF: C(v|u) = u⁻⁽ᶿ⁺¹⁾ · (u⁻ᶿ + v⁻ᶿ - 1)⁻⁽ᶿ⁺¹⁾/ᶿ
pub fn clayton_conditional_cdf(u: f64, v: f64, theta: f64) -> f64 {
    let u = u.clamp(1e-10, 1.0 - 1e-10);
    let v = v.clamp(1e-10, 1.0 - 1e-10);

    let val = u.powf(-theta) + v.powf(-theta) - 1.0;
    if val <= 0.0 {
        return 0.0;
    }
    u.powf(-(theta + 1.0)) * val.powf(-(theta + 1.0) / theta)
}

/// Inverse conditional CDF: given u and t ∈ [0,1], find v such that C(v|u) = t
/// Derived: v = ((t·u^(θ+1))^(-θ/(θ+1)) - u^(-θ) + 1)^(-1/θ)
fn clayton_conditional_inverse(u: f64, t: f64, theta: f64) -> f64 {
    let u = u.clamp(1e-10, 1.0 - 1e-10);
    let t = t.clamp(1e-10, 1.0 - 1e-10);

    let exp = -theta / (theta + 1.0);
    let inner = (t * u.powf(theta + 1.0)).powf(exp) - u.powf(-theta) + 1.0;

    if inner <= 0.0 {
        return 1e-10;
    }
    inner.powf(-1.0 / theta).clamp(1e-10, 1.0 - 1e-10)
}

/// Sample from bivariate Clayton copula
pub fn clayton_copula_sample(theta: f64, n_samples: usize, seed: Option<u64>) -> Vec<(f64, f64)> {
    let mut rng = match seed {
        Some(s) => StdRng::seed_from_u64(s),
        None => StdRng::from_entropy(),
    };
    let uniform = Uniform::new(1e-10, 1.0 - 1e-10);

    let mut samples = Vec::with_capacity(n_samples);

    for _ in 0..n_samples {
        let u1 = uniform.sample(&mut rng);
        let t = uniform.sample(&mut rng);
        let u2 = clayton_conditional_inverse(u1, t, theta);
        samples.push((u1, u2));
    }

    samples
}

/// Conditional sampling: given u1, sample u2 from C(·|u1)
pub fn clayton_conditional_sample(
    u1: f64,
    theta: f64,
    n_samples: usize,
    seed: Option<u64>,
) -> Vec<(f64, f64)> {
    let mut rng = match seed {
        Some(s) => StdRng::seed_from_u64(s),
        None => StdRng::from_entropy(),
    };
    let uniform = Uniform::new(1e-10, 1.0 - 1e-10);

    let mut samples = Vec::with_capacity(n_samples);

    for _ in 0..n_samples {
        let t = uniform.sample(&mut rng);
        let u2 = clayton_conditional_inverse(u1, t, theta);
        samples.push((u1, u2));
    }

    samples
}

/// Fit Clayton copula parameter via Kendall's tau
/// theta = 2 * tau / (1 - tau)
pub fn fit_clayton_copula(u: &[f64], v: &[f64]) -> Result<(f64, f64), String> {
    if u.len() != v.len() {
        return Err("u and v must have the same length".to_string());
    }
    let n = u.len();
    if n < 3 {
        return Err("Need at least 3 data points".to_string());
    }

    // Compute Kendall's tau
    let tau = kendalls_tau(u, v);

    if tau <= 0.0 {
        return Err("Clayton copula requires positive dependence (tau > 0)".to_string());
    }
    if tau >= 1.0 {
        return Err("Perfect dependence not supported".to_string());
    }

    let theta = 2.0 * tau / (1.0 - tau);
    let theta = theta.max(0.01); // ensure positive

    // Compute log-likelihood
    let ll = clayton_log_likelihood(u, v, theta);

    Ok((theta, ll))
}

/// Kendall's tau: concordance measure
pub fn kendalls_tau(u: &[f64], v: &[f64]) -> f64 {
    let n = u.len();
    let mut concordant = 0i64;
    let mut discordant = 0i64;

    for i in 0..n {
        for j in (i + 1)..n {
            let du = u[i] - u[j];
            let dv = v[i] - v[j];
            let product = du * dv;
            if product > 0.0 {
                concordant += 1;
            } else if product < 0.0 {
                discordant += 1;
            }
        }
    }

    let total = concordant + discordant;
    if total == 0 {
        return 0.0;
    }
    (concordant - discordant) as f64 / total as f64
}

fn clayton_log_likelihood(u: &[f64], v: &[f64], theta: f64) -> f64 {
    let mut ll = 0.0;
    let n = u.len();

    for i in 0..n {
        let ui = u[i].clamp(1e-10, 1.0 - 1e-10);
        let vi = v[i].clamp(1e-10, 1.0 - 1e-10);

        // Clayton copula density:
        // c(u,v) = (1+theta) * (u*v)^(-(theta+1)) * (u^(-theta) + v^(-theta) - 1)^(-(2*theta+1)/theta)
        let log_density = (1.0 + theta).ln()
            - (theta + 1.0) * (ui.ln() + vi.ln())
            + (-(2.0 * theta + 1.0) / theta)
                * (ui.powf(-theta) + vi.powf(-theta) - 1.0).ln();

        ll += log_density;
    }

    ll
}
