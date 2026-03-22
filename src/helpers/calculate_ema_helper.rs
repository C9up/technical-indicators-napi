use napi::Error;

pub fn calculate_ema(data: &[f64], period: i32) -> Result<Vec<f64>, Error> {
    if period <= 0 {
        return Err(Error::from_reason("Period must be greater than 0"));
    }

    if data.is_empty() {
        return Err(Error::from_reason("Data array cannot be empty"));
    }

    let period_usize = period as usize;

    if data.len() < period_usize {
        return Err(Error::from_reason("Period must be lower than data length"));
    }

    let smoothing_factor = 2.0 / (period as f64 + 1.0);
    let mut ema = Vec::with_capacity(data.len());

    // Seed EMA with SMA of first `period` values (standard initialization)
    let sma_seed: f64 = data[..period_usize].iter().sum::<f64>() / period as f64;
    ema.push(sma_seed);

    // Apply EMA formula starting from index `period`
    for i in period_usize..data.len() {
        let val = smoothing_factor * data[i] + (1.0 - smoothing_factor) * ema[i - period_usize];
        ema.push(val);
    }

    Ok(ema)
}
