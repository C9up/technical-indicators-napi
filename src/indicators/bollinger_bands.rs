use napi_derive::napi;
use crate::compute_bollinger_bands::{compute_bollinger_bands, BollingerBandsResult};

#[napi]
pub fn bollinger_bands(
    data: Vec<f64>,
    period: Option<i32>,
    multiplier: Option<f64>,
) -> Result<BollingerBandsResult, napi::Error> {
    let period = period.unwrap_or(20);
    let multiplier = multiplier.unwrap_or(2.0);

    if period <= 0 {
        return Err(napi::Error::from_reason("Period must be greater than 0."));
    }

    if multiplier <= 0.0 {
        return Err(napi::Error::from_reason("Multiplier must be greater than 0."));
    }

    if data.is_empty() {
        return Err(napi::Error::from_reason("Prices vector must not be empty."));
    }

    Ok(compute_bollinger_bands(&data, period, multiplier))
}
