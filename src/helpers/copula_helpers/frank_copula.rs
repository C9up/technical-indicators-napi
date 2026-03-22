use rand::rngs::StdRng;
use rand::SeedableRng;
use rand_distr::{Distribution, Uniform};

/// Frank copula CDF: C(u,v) = -1/θ · ln(1 + (e⁻ᶿᵘ - 1)(e⁻ᶿᵛ - 1)/(e⁻ᶿ - 1))
pub fn frank_copula_cdf(u: f64, v: f64, theta: f64) -> f64 {
    let u = u.clamp(1e-10, 1.0 - 1e-10);
    let v = v.clamp(1e-10, 1.0 - 1e-10);

    if theta.abs() < 1e-10 {
        // Independence case
        return u * v;
    }

    let e_theta = (-theta).exp();
    let num = ((-theta * u).exp() - 1.0) * ((-theta * v).exp() - 1.0);
    let denom = e_theta - 1.0;

    (-1.0 / theta) * (1.0 + num / denom).ln()
}

/// Conditional CDF: ∂C(u,v)/∂u
/// = e⁻ᶿᵘ · (e⁻ᶿᵛ - 1) / ((e⁻ᶿ - 1) + (e⁻ᶿᵘ - 1)(e⁻ᶿᵛ - 1))
pub fn frank_conditional_cdf(u: f64, v: f64, theta: f64) -> f64 {
    let u = u.clamp(1e-10, 1.0 - 1e-10);
    let v = v.clamp(1e-10, 1.0 - 1e-10);

    if theta.abs() < 1e-10 {
        return v;
    }

    let eu = (-theta * u).exp();
    let ev = (-theta * v).exp();
    let e = (-theta).exp();

    let num = eu * (ev - 1.0);
    let denom = (e - 1.0) + (eu - 1.0) * (ev - 1.0);

    (num / denom).clamp(1e-10, 1.0 - 1e-10)
}

/// Inverse conditional CDF via bisection
/// Find v such that C(v|u) = t
fn frank_conditional_inverse(u: f64, t: f64, theta: f64) -> f64 {
    let t = t.clamp(1e-10, 1.0 - 1e-10);

    if theta.abs() < 1e-10 {
        return t;
    }

    // Analytic inverse:
    // t = eu * (ev - 1) / ((e - 1) + (eu - 1)(ev - 1))
    // Solving for v:
    let eu = (-theta * u.clamp(1e-10, 1.0 - 1e-10)).exp();
    let e = (-theta).exp();

    let denom = eu - t * eu + t * (1.0 - e);
    if denom.abs() < 1e-15 {
        return 0.5;
    }

    let inner = 1.0 + t * (e - 1.0) / denom;
    if inner <= 0.0 {
        return 1e-10;
    }

    let v = -inner.ln() / theta;
    v.clamp(1e-10, 1.0 - 1e-10)
}

/// Sample from bivariate Frank copula
pub fn frank_copula_sample(theta: f64, n_samples: usize, seed: Option<u64>) -> Vec<(f64, f64)> {
    let mut rng = match seed {
        Some(s) => StdRng::seed_from_u64(s),
        None => StdRng::from_entropy(),
    };
    let uniform = Uniform::new(1e-10, 1.0 - 1e-10);

    let mut samples = Vec::with_capacity(n_samples);

    for _ in 0..n_samples {
        let u1 = uniform.sample(&mut rng);
        let t = uniform.sample(&mut rng);
        let u2 = frank_conditional_inverse(u1, t, theta);
        samples.push((u1, u2));
    }

    samples
}

/// Conditional sampling: given u1, sample u2 from C(·|u1)
pub fn frank_conditional_sample(
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
        let u2 = frank_conditional_inverse(u1, t, theta);
        samples.push((u1, u2));
    }

    samples
}

/// Fit Frank copula parameter via Kendall's tau
/// Relationship: tau = 1 - 4/theta * (1 - D₁(theta))
/// where D₁ is the first Debye function
/// We invert numerically via bisection
pub fn fit_frank_copula(u: &[f64], v: &[f64]) -> Result<(f64, f64), String> {
    if u.len() != v.len() {
        return Err("u and v must have the same length".to_string());
    }
    if u.len() < 3 {
        return Err("Need at least 3 data points".to_string());
    }

    let tau = super::clayton_copula::kendalls_tau(u, v);

    // Find theta such that tau_frank(theta) = tau
    // Use bisection on a wide range
    let theta = find_frank_theta(tau)?;

    let ll = frank_log_likelihood(u, v, theta);

    Ok((theta, ll))
}

/// First Debye function: D₁(x) = (1/x) * integral(t/(e^t - 1), 0, x)
/// Approximated via numerical integration (trapezoidal rule)
fn debye1(x: f64) -> f64 {
    if x.abs() < 1e-10 {
        return 1.0;
    }

    let n_steps = 100;
    let h = x / n_steps as f64;
    let mut sum = 0.0;

    for i in 1..n_steps {
        let t = i as f64 * h;
        let et = t.exp();
        if et > 1.0 {
            sum += t / (et - 1.0);
        }
    }
    // Trapezoidal: add half of endpoints
    // At t=0: limit is 1, at t=x: x/(e^x - 1)
    let ex = x.exp();
    let endpoint = if ex > 1.0 { x / (ex - 1.0) } else { 1.0 };
    sum = (sum + 0.5 * 1.0 + 0.5 * endpoint) * h;

    sum / x
}

/// Kendall's tau as function of Frank theta
fn frank_tau(theta: f64) -> f64 {
    if theta.abs() < 1e-10 {
        return 0.0;
    }
    1.0 - 4.0 / theta * (1.0 - debye1(theta))
}

/// Find theta given tau using bisection
fn find_frank_theta(tau: f64) -> Result<f64, String> {
    if tau.abs() < 1e-10 {
        return Ok(0.0);
    }

    // Search range: theta ∈ [-50, 50] (covers practical range)
    let (mut lo, mut hi) = if tau > 0.0 {
        (1e-10, 50.0)
    } else {
        (-50.0, -1e-10)
    };

    for _ in 0..100 {
        let mid = (lo + hi) / 2.0;
        let tau_mid = frank_tau(mid);

        if (tau_mid - tau).abs() < 1e-8 {
            return Ok(mid);
        }

        if tau_mid < tau {
            lo = mid;
        } else {
            hi = mid;
        }
    }

    Ok((lo + hi) / 2.0)
}

fn frank_log_likelihood(u: &[f64], v: &[f64], theta: f64) -> f64 {
    if theta.abs() < 1e-10 {
        return 0.0; // independence
    }

    let mut ll = 0.0;
    let e = (-theta).exp();

    for i in 0..u.len() {
        let ui = u[i].clamp(1e-10, 1.0 - 1e-10);
        let vi = v[i].clamp(1e-10, 1.0 - 1e-10);

        let eu = (-theta * ui).exp();
        let ev = (-theta * vi).exp();

        // Frank copula density:
        // c(u,v) = -theta*(e^(-theta) - 1) * e^(-theta*(u+v)) /
        //          ((e^(-theta) - 1) + (e^(-theta*u) - 1)(e^(-theta*v) - 1))²
        let num = -theta * (e - 1.0) * (-theta * (ui + vi)).exp();
        let denom_inner = (e - 1.0) + (eu - 1.0) * (ev - 1.0);
        let denom = denom_inner * denom_inner;

        if num.abs() > 0.0 && denom > 0.0 {
            ll += (num / denom).abs().ln();
        }
    }

    ll
}
