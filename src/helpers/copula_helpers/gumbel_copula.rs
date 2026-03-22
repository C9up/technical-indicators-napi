use rand::rngs::StdRng;
use rand::SeedableRng;
use rand_distr::{Distribution, Uniform};

/// Gumbel copula CDF: C(u,v) = exp(−[(−ln u)ᶿ + (−ln v)ᶿ]¹/ᶿ)
/// theta >= 1 (theta = 1 is independence)
pub fn gumbel_copula_cdf(u: f64, v: f64, theta: f64) -> f64 {
    let u = u.clamp(1e-10, 1.0 - 1e-10);
    let v = v.clamp(1e-10, 1.0 - 1e-10);

    let lu = (-u.ln()).powf(theta);
    let lv = (-v.ln()).powf(theta);
    let sum = (lu + lv).powf(1.0 / theta);

    (-sum).exp()
}

/// Conditional CDF: ∂C(u,v)/∂u
/// = C(u,v) / u * (-ln u)^(theta-1) * [(−ln u)^theta + (−ln v)^theta]^(1/theta - 1)
pub fn gumbel_conditional_cdf(u: f64, v: f64, theta: f64) -> f64 {
    let u = u.clamp(1e-10, 1.0 - 1e-10);
    let v = v.clamp(1e-10, 1.0 - 1e-10);

    let lu = (-u.ln()).powf(theta);
    let lv = (-v.ln()).powf(theta);
    let a = lu + lv;
    let c = (-a.powf(1.0 / theta)).exp();

    c / u * (-u.ln()).powf(theta - 1.0) * a.powf(1.0 / theta - 1.0)
}

/// Inverse conditional CDF via bisection
/// Find v such that C(v|u) = t
fn gumbel_conditional_inverse(u: f64, t: f64, theta: f64) -> f64 {
    let t = t.clamp(1e-10, 1.0 - 1e-10);

    let mut lo = 1e-10_f64;
    let mut hi = 1.0 - 1e-10;

    // Bisection method
    for _ in 0..64 {
        let mid = (lo + hi) / 2.0;
        let cdf_val = gumbel_conditional_cdf(u, mid, theta);

        if cdf_val < t {
            lo = mid;
        } else {
            hi = mid;
        }
    }

    (lo + hi) / 2.0
}

/// Sample from bivariate Gumbel copula using conditional inverse method
pub fn gumbel_copula_sample(theta: f64, n_samples: usize, seed: Option<u64>) -> Vec<(f64, f64)> {
    let mut rng = match seed {
        Some(s) => StdRng::seed_from_u64(s),
        None => StdRng::from_entropy(),
    };
    let uniform = Uniform::new(1e-10, 1.0 - 1e-10);

    let mut samples = Vec::with_capacity(n_samples);

    for _ in 0..n_samples {
        let u1 = uniform.sample(&mut rng);
        let t = uniform.sample(&mut rng);
        let u2 = gumbel_conditional_inverse(u1, t, theta);
        samples.push((u1, u2));
    }

    samples
}

/// Conditional sampling: given u1, sample u2 from C(·|u1)
pub fn gumbel_conditional_sample(
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
        let u2 = gumbel_conditional_inverse(u1, t, theta);
        samples.push((u1, u2));
    }

    samples
}

/// Fit Gumbel copula parameter via Kendall's tau
/// theta = 1 / (1 - tau)
pub fn fit_gumbel_copula(u: &[f64], v: &[f64]) -> Result<(f64, f64), String> {
    if u.len() != v.len() {
        return Err("u and v must have the same length".to_string());
    }
    if u.len() < 3 {
        return Err("Need at least 3 data points".to_string());
    }

    let tau = super::clayton_copula::kendalls_tau(u, v);

    if tau < 0.0 {
        return Err("Gumbel copula requires non-negative dependence (tau >= 0)".to_string());
    }
    if tau >= 1.0 {
        return Err("Perfect dependence not supported".to_string());
    }

    let theta = (1.0 / (1.0 - tau)).max(1.0);

    let ll = gumbel_log_likelihood(u, v, theta);

    Ok((theta, ll))
}

fn gumbel_log_likelihood(u: &[f64], v: &[f64], theta: f64) -> f64 {
    let mut ll = 0.0;

    for i in 0..u.len() {
        let ui = u[i].clamp(1e-10, 1.0 - 1e-10);
        let vi = v[i].clamp(1e-10, 1.0 - 1e-10);

        let lu = (-ui.ln()).powf(theta);
        let lv = (-vi.ln()).powf(theta);
        let a = lu + lv;
        let a_inv_theta = a.powf(1.0 / theta);

        // Gumbel copula density (simplified log form)
        let log_c = -a_inv_theta
            + (theta - 1.0) * ((-ui.ln()).ln() + (-vi.ln()).ln())
            + (1.0 / theta - 2.0) * a.ln()
            + ((theta - 1.0) * a_inv_theta + 1.0).ln()
            - ui.ln()
            - vi.ln();

        ll += log_c;
    }

    ll
}
