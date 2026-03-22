use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi(object)]
pub struct PerformanceMetrics {
    /// Annualized Sharpe Ratio: (mean_return - risk_free) / std * sqrt(252)
    pub sharpe_ratio: f64,
    /// Annualized Sortino Ratio: (mean_return - risk_free) / downside_std * sqrt(252)
    pub sortino_ratio: f64,
    /// Calmar Ratio: annualized_return / max_drawdown
    pub calmar_ratio: f64,
    /// Maximum Drawdown (as positive fraction, e.g. 0.25 = 25%)
    pub max_drawdown: f64,
    /// Maximum Drawdown duration in bars
    pub max_drawdown_duration: i32,
    /// Total cumulative return (e.g. 0.50 = 50%)
    pub total_return: f64,
    /// Annualized return
    pub annualized_return: f64,
    /// Annualized volatility (std of returns * sqrt(252))
    pub annualized_volatility: f64,
    /// Win rate: fraction of positive returns
    pub win_rate: f64,
    /// Profit factor: sum of gains / sum of losses
    pub profit_factor: f64,
    /// Average win / average loss ratio
    pub payoff_ratio: f64,
    /// Number of trading periods
    pub num_periods: i32,
    /// Skewness of returns
    pub skewness: f64,
    /// Excess kurtosis of returns
    pub kurtosis: f64,
    /// Value at Risk (5th percentile of returns)
    pub var_95: f64,
    /// Conditional VaR / Expected Shortfall (mean of returns below VaR)
    pub cvar_95: f64,
}

/// Compute comprehensive performance metrics from a returns series.
///
/// Input: array of period returns (e.g. daily returns as decimals: 0.01 = 1%)
///
/// Parameters:
/// - returns: array of period returns
/// - risk_free_rate: annualized risk-free rate (default: 0.02 = 2%)
/// - periods_per_year: trading periods per year (default: 252 for daily)
#[napi]
pub fn performance_metrics(
    returns: Vec<f64>,
    risk_free_rate: Option<f64>,
    periods_per_year: Option<u32>,
) -> Result<PerformanceMetrics> {
    if returns.is_empty() {
        return Err(Error::from_reason("Returns array cannot be empty"));
    }

    let rf = risk_free_rate.unwrap_or(0.02);
    let ppy = periods_per_year.unwrap_or(252) as f64;
    let n = returns.len() as f64;
    let rf_per_period = rf / ppy;
    let sqrt_ppy = ppy.sqrt();

    // --- Basic stats ---
    let mean = returns.iter().sum::<f64>() / n;
    let variance = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / n;
    let std = variance.sqrt();

    // --- Sharpe Ratio ---
    let sharpe = if std > 1e-15 {
        (mean - rf_per_period) / std * sqrt_ppy
    } else {
        0.0
    };

    // --- Sortino Ratio (downside deviation) ---
    let downside_returns: Vec<f64> = returns.iter()
        .map(|r| (r - rf_per_period).min(0.0))
        .collect();
    let downside_var = downside_returns.iter().map(|r| r.powi(2)).sum::<f64>() / n;
    let downside_std = downside_var.sqrt();

    let sortino = if downside_std > 1e-15 {
        (mean - rf_per_period) / downside_std * sqrt_ppy
    } else {
        0.0
    };

    // --- Cumulative return and drawdown ---
    let mut cumulative = Vec::with_capacity(returns.len());
    let mut cum = 1.0;
    for &r in &returns {
        cum *= 1.0 + r;
        cumulative.push(cum);
    }

    let total_return = cum - 1.0;
    let years = n / ppy;
    let annualized_return = if years > 0.0 {
        cum.powf(1.0 / years) - 1.0
    } else {
        0.0
    };
    let annualized_vol = std * sqrt_ppy;

    // Max drawdown
    let mut peak = f64::NEG_INFINITY;
    let mut max_dd = 0.0_f64;
    let mut max_dd_duration = 0i32;
    let mut current_dd_start = 0usize;
    let mut in_drawdown = false;

    for (i, &c) in cumulative.iter().enumerate() {
        if c > peak {
            peak = c;
            if in_drawdown {
                let duration = i - current_dd_start;
                if duration as i32 > max_dd_duration {
                    max_dd_duration = duration as i32;
                }
                in_drawdown = false;
            }
        } else {
            let dd = (peak - c) / peak;
            if dd > max_dd {
                max_dd = dd;
            }
            if !in_drawdown {
                current_dd_start = i;
                in_drawdown = true;
            }
        }
    }
    // Check if still in drawdown at end
    if in_drawdown {
        let duration = (returns.len() - current_dd_start) as i32;
        if duration > max_dd_duration {
            max_dd_duration = duration;
        }
    }

    // --- Calmar Ratio ---
    let calmar = if max_dd > 1e-15 {
        annualized_return / max_dd
    } else {
        0.0
    };

    // --- Win rate, profit factor, payoff ratio ---
    let wins: Vec<f64> = returns.iter().filter(|&&r| r > 0.0).copied().collect();
    let losses: Vec<f64> = returns.iter().filter(|&&r| r < 0.0).copied().collect();

    let win_rate = wins.len() as f64 / n;

    let sum_wins: f64 = wins.iter().sum();
    let sum_losses: f64 = losses.iter().map(|l| l.abs()).sum();

    let profit_factor = if sum_losses > 1e-15 {
        sum_wins / sum_losses
    } else if sum_wins > 0.0 {
        f64::INFINITY
    } else {
        0.0
    };

    let avg_win = if !wins.is_empty() { sum_wins / wins.len() as f64 } else { 0.0 };
    let avg_loss = if !losses.is_empty() { sum_losses / losses.len() as f64 } else { 0.0 };
    let payoff_ratio = if avg_loss > 1e-15 { avg_win / avg_loss } else { 0.0 };

    // --- Skewness and Kurtosis ---
    let skewness = if std > 1e-15 && n > 2.0 {
        let m3 = returns.iter().map(|r| ((r - mean) / std).powi(3)).sum::<f64>();
        m3 / n
    } else {
        0.0
    };

    let kurtosis = if std > 1e-15 && n > 3.0 {
        let m4 = returns.iter().map(|r| ((r - mean) / std).powi(4)).sum::<f64>();
        m4 / n - 3.0 // excess kurtosis
    } else {
        0.0
    };

    // --- VaR and CVaR (95%) ---
    let mut sorted_returns = returns.clone();
    sorted_returns.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let var_idx = ((n * 0.05).floor() as usize).max(0);
    let var_95 = sorted_returns[var_idx.min(sorted_returns.len() - 1)];

    let tail: Vec<f64> = sorted_returns.iter().filter(|&&r| r <= var_95).copied().collect();
    let cvar_95 = if !tail.is_empty() {
        tail.iter().sum::<f64>() / tail.len() as f64
    } else {
        var_95
    };

    Ok(PerformanceMetrics {
        sharpe_ratio: sharpe,
        sortino_ratio: sortino,
        calmar_ratio: calmar,
        max_drawdown: max_dd,
        max_drawdown_duration: max_dd_duration,
        total_return,
        annualized_return,
        annualized_volatility: annualized_vol,
        win_rate,
        profit_factor,
        payoff_ratio,
        num_periods: returns.len() as i32,
        skewness,
        kurtosis,
        var_95,
        cvar_95,
    })
}

/// Quick Sharpe Ratio calculation from returns.
#[napi]
pub fn sharpe_ratio(
    returns: Vec<f64>,
    risk_free_rate: Option<f64>,
    periods_per_year: Option<u32>,
) -> Result<f64> {
    if returns.is_empty() {
        return Err(Error::from_reason("Returns array cannot be empty"));
    }

    let rf = risk_free_rate.unwrap_or(0.02);
    let ppy = periods_per_year.unwrap_or(252) as f64;
    let n = returns.len() as f64;
    let rf_per_period = rf / ppy;

    let mean = returns.iter().sum::<f64>() / n;
    let std = (returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / n).sqrt();

    Ok(if std > 1e-15 { (mean - rf_per_period) / std * ppy.sqrt() } else { 0.0 })
}

/// Quick Sortino Ratio calculation from returns.
#[napi]
pub fn sortino_ratio(
    returns: Vec<f64>,
    risk_free_rate: Option<f64>,
    periods_per_year: Option<u32>,
) -> Result<f64> {
    if returns.is_empty() {
        return Err(Error::from_reason("Returns array cannot be empty"));
    }

    let rf = risk_free_rate.unwrap_or(0.02);
    let ppy = periods_per_year.unwrap_or(252) as f64;
    let n = returns.len() as f64;
    let rf_per_period = rf / ppy;

    let mean = returns.iter().sum::<f64>() / n;
    let downside_var = returns.iter()
        .map(|r| (r - rf_per_period).min(0.0).powi(2))
        .sum::<f64>() / n;
    let downside_std = downside_var.sqrt();

    Ok(if downside_std > 1e-15 { (mean - rf_per_period) / downside_std * ppy.sqrt() } else { 0.0 })
}

/// Quick Max Drawdown calculation from returns.
#[napi]
pub fn max_drawdown(returns: Vec<f64>) -> Result<f64> {
    if returns.is_empty() {
        return Err(Error::from_reason("Returns array cannot be empty"));
    }

    let mut peak = 1.0_f64;
    let mut cum = 1.0;
    let mut max_dd = 0.0_f64;

    for &r in &returns {
        cum *= 1.0 + r;
        if cum > peak { peak = cum; }
        let dd = (peak - cum) / peak;
        if dd > max_dd { max_dd = dd; }
    }

    Ok(max_dd)
}
