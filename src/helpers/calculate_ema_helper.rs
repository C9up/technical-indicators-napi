use napi::Error;

pub fn calculate_ema(data: &Vec<f64>, period: i32) -> Result<Vec<f64>, Error> {

    if period <= 0 {
        return Err(Error::from_reason("Period must be greater than 0"));
    }

    if data.is_empty() {
        return Err(Error::from_reason("Data array cannot be empty"));
    }

    if data.len() < period as usize {
        return Err(Error::from_reason("Period must be lower than data length"));
    }

    // Calculate the smoothing factor.
    let smoothing_factor = 2.0 / (period as f64 + 1.0);
    let mut ema = Vec::with_capacity(data.len());

    // Initialisation avec la première valeur
    ema.push(data[0]);

    // Calcul récursif sans SMA initiale
    for i in 1..data.len() {
        let val = smoothing_factor * data[i] + (1.0 - smoothing_factor) * ema[i - 1];
        ema.push(val);
    }

    Ok(ema)
}
