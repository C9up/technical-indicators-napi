use napi::Error;

pub fn calculate_sma(data: &[f64], period: i32) -> Result<Vec<f64>, Error> {
    if period <= 0 {
        return Err(Error::from_reason("[SMA] Period must be greater than 0"));
    }

    if data.is_empty() {
        return Err(Error::from_reason("[SMA] Data array cannot be empty"));
    }

    let period_usize = period as usize;

    if data.len() < period_usize {
        return Err(Error::from_reason(format!(
            "[SMA] Data array length ({}) is less than period ({})",
            data.len(),
            period
        )));
    }

    let mut sma = Vec::with_capacity(data.len());
    let mut sum = 0.0;

    // Calcul de la première moyenne
    for i in 0..period_usize {
        sum += data[i];
    }
    sma.push(sum / period as f64);

    // Calcul des moyennes suivantes avec mise à jour glissante
    for i in period_usize..data.len() {
        sum += data[i] - data[i - period_usize];
        sma.push(sum / period as f64);
    }

    // Ajout des NaN pour les périodes incomplètes
    let mut result = vec![f64::NAN; period_usize - 1];
    result.extend(sma);

    Ok(result)
}