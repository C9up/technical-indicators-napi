use napi_derive::napi;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[napi(object)]
pub struct BollingerBandsResult {
    pub middle: Vec<f64>,
    pub upper: Vec<f64>,
    pub lower: Vec<f64>,
}

pub fn compute_bollinger_bands(prices: &[f64], period: i32, multiplier: f64) -> BollingerBandsResult {
    let period_usize = period as usize;
    let n = period_usize as f64;

    // Gérer le cas où period est plus grand que le nombre de prix
    let out_size = prices.len().saturating_sub(period_usize).saturating_add(1);
    let mut middle = Vec::with_capacity(out_size);
    let mut upper = Vec::with_capacity(out_size);
    let mut lower = Vec::with_capacity(out_size);

    if prices.len() < period_usize {
        return BollingerBandsResult { middle, upper, lower };
    }

    // Calcul initial pour la première fenêtre
    let (mut sum, mut sum_sq) = prices[..period_usize].iter().fold((0.0, 0.0), |(s, s_sq), &p| {
        (s + p, s_sq + p * p)
    });

    let ma = sum / n;
    let stdev = calculate_std(sum, sum_sq, n);
    middle.push(ma);
    upper.push(ma + multiplier * stdev);
    lower.push(ma - multiplier * stdev);

    // Parcourir les éléments restants
    for i in period_usize..prices.len() {
        let old = prices[i - period_usize];
        let new = prices[i];

        sum += new - old;
        sum_sq += new * new - old * old;

        let ma = sum / n;
        let stdev = calculate_std(sum, sum_sq, n);
        middle.push(ma);
        upper.push(ma + multiplier * stdev);
        lower.push(ma - multiplier * stdev);
    }

    BollingerBandsResult { middle, upper, lower }
}

#[inline(always)]
fn calculate_std(sum: f64, sum_sq: f64, n: f64) -> f64 {
    let variance = (sum_sq - (sum * sum) / n) / n;
    variance.sqrt()
}