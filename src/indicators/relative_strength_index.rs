use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi]
pub fn relative_strength_index(prices: Vec<f64>, period: u32) -> Result<Vec<f64>> {
    let period = period as usize;

    if period == 0 {
        return Err(Error::from_reason("Period must be greater than 0"));
    }

    if prices.len() < period + 1 {
        return Err(Error::from_reason(format!(
            "Not enough data. Need at least {} prices for period {}",
            period + 1,
            period
        )));
    }

    // Calculate price changes
    let changes: Vec<f64> = prices.windows(2).map(|w| w[1] - w[0]).collect();

    // Initial average gain and loss over first `period` changes
    let mut avg_gain = 0.0;
    let mut avg_loss = 0.0;
    for change in &changes[..period] {
        if *change > 0.0 {
            avg_gain += change;
        } else {
            avg_loss -= change;
        }
    }
    avg_gain /= period as f64;
    avg_loss /= period as f64;

    let mut rsi_values = Vec::with_capacity(changes.len() - period + 1);

    // First RSI value
    if avg_loss == 0.0 {
        rsi_values.push(100.0);
    } else {
        let rs = avg_gain / avg_loss;
        rsi_values.push(100.0 - (100.0 / (1.0 + rs)));
    }

    // Subsequent RSI values using Wilder's smoothing
    for change in &changes[period..] {
        let gain = if *change > 0.0 { *change } else { 0.0 };
        let loss = if *change < 0.0 { -change } else { 0.0 };

        avg_gain = (avg_gain * (period as f64 - 1.0) + gain) / period as f64;
        avg_loss = (avg_loss * (period as f64 - 1.0) + loss) / period as f64;

        if avg_loss == 0.0 {
            rsi_values.push(100.0);
        } else {
            let rs = avg_gain / avg_loss;
            rsi_values.push(100.0 - (100.0 / (1.0 + rs)));
        }
    }

    Ok(rsi_values)
}
