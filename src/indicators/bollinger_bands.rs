use napi_derive::napi;
use crate::compute_bollinger_bands::compute_bollinger_bands;

#[napi(object)]
pub struct BollingerBandsResult {
    pub upper: Vec<f64>,
    pub middle: Vec<f64>,
    pub lower: Vec<f64>,
}

#[napi]
pub fn bollinger_bands(
    data: Vec<f64>,
    period: Option<i32>,
    multiplier: Option<f64>,
) -> Result<BollingerBandsResult, napi::Error> {
    // Valeurs par défaut
    let period = period.unwrap_or(20);
    let multiplier = multiplier.unwrap_or(2.0);

    // Validation des entrées
    if period <= 0 {
        return Err(napi::Error::from_reason("Period must be greater than 0."));
    }

    if multiplier <= 0.0 {
        return Err(napi::Error::from_reason("Multiplier must be greater than 0."));
    }

    if data.is_empty() {
        return Err(napi::Error::from_reason("Prices vector must not be empty."));
    }

    // Calcul des bandes
    let result = compute_bollinger_bands(&data, period, multiplier);

    // Conversion vers le format NAPI
    Ok(BollingerBandsResult {
        upper: result.upper,
        middle: result.middle,
        lower: result.lower,
    })
}