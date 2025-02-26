use napi_derive::napi;
use crate::calculate_ema_helper::calculate_ema;

#[napi]
pub fn exponential_moving_average(data: Vec<f64>, period: i32) -> napi::Result<Vec<f64>>{

    if data.is_empty() {
        return Err(napi::Error::from_reason("Prices vector must not be empty."));
    }

    let result = calculate_ema(&data, period);
    result
}
