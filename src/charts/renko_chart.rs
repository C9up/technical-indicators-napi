use serde::Serialize;
use napi_derive::napi;
use napi::{Error, Result};

#[napi(object)]
#[derive(Serialize)]
pub struct RenkoBrick {
    pub price: f64,
    pub direction: String,
}

#[napi]
pub fn renko_chart(
    prices: Vec<f64>,
    #[napi(ts_arg_type = "number", default = 10)] brick_size: Option<f64>,
) -> Result<Vec<RenkoBrick>> {

    let brick_size = brick_size.unwrap_or(10.0).max(1.0);

    if brick_size <= 0.0 {
        return Err(Error::from_reason("brick_size amount must be greater than 0."));
    }

    if prices.is_empty() {
        return Err(Error::from_reason("Prices vector must not be empty."));
    }

    if brick_size >= prices.len() as f64 {
        return Err(Error::from_reason("brick_size must be lower than prices length."));
    }

    let mut bricks = Vec::new();
    let mut current_price = prices[0];

    bricks.push(RenkoBrick {
        price: current_price,
        direction: "up".to_string(),
    });

    for price in prices.into_iter().skip(1) {
        let diff = price - current_price;
        let abs_diff = diff.abs();

        if abs_diff >= brick_size {
            let num_bricks = (abs_diff / brick_size).floor() as i32;
            let direction = if diff > 0.0 { "up" } else { "down" };

            for _ in 0..num_bricks {
                current_price += brick_size * diff.signum();

                bricks.push(RenkoBrick {
                    price: current_price,
                    direction: direction.to_string(),
                });
            }
        }
    }

    Ok(bricks)
}