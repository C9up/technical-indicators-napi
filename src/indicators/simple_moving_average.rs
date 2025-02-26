use napi::{bindgen_prelude::*, Error, Status};
use napi_derive::napi;
use crate::calculate_sma_helper::calculate_sma;

#[napi]
pub fn simple_moving_average(data: Vec<f64>, period: i32) -> Result<Vec<f64>> {
    if period == 0 {
        return Err(Error::new(
            Status::InvalidArg,
            "Period must be greater than 0".to_string(),
        ));
    }

    if data.is_empty() {
        return Err(Error::new(
            Status::InvalidArg,
            "Data array cannot be empty".to_string(),
        ));
    }

    let data_len = data.len();
    if data_len < period as usize {
        return Err(Error::from_reason(format!(
            "Data array length ({}) is less than period ({})",
            data_len,
            period
        )));
    }

    calculate_sma(&data, period)
        .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
}