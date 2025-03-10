use napi::{bindgen_prelude::*, Error};
use napi_derive::napi;
use serde::Serialize;

#[napi(object)]
#[derive(Serialize)]
struct KagiPoint {
    pub price: f64,
    pub direction: String,
}

#[napi]
fn kagi_chart(
    prices: Vec<f64>,
    #[napi(ts_arg_type = "number", default = 20)] reversal_amount: Option<f64>,
) -> Result<Vec<KagiPoint>> {

    let reversal_amount = reversal_amount.unwrap_or(20.0).max(1.0);
    if reversal_amount <= 0.0 {
        return Err(Error::from_reason("Reversal amount must be greater than 0."));
    }

    if prices.is_empty() {
        return Err(Error::from_reason("Prices vector must not be empty."));
    }

    let mut result_points = Vec::new();
    let mut current_direction = true;
    let mut current_price = prices[0];

    for &price in &prices[1..] {
        if current_direction {
            if price >= current_price {
                current_price = price;
            } else if current_price - price >= reversal_amount {
                result_points.push(KagiPoint {
                    price: current_price,
                    direction: "Yang".to_string(),
                });
                current_direction = false;
                current_price = price;
            }
        } else {
            if price <= current_price {
                current_price = price;
            } else if price - current_price >= reversal_amount {
                result_points.push(KagiPoint {
                    price: current_price,
                    direction: "Yin".to_string(),
                });
                current_direction = true;
                current_price = price;
            }
        }
    }

    result_points.push(KagiPoint {
        price: current_price,
        direction: if current_direction {
            "Yang".to_string()
        } else {
            "Yin".to_string()
        },
    });

    Ok(result_points)
}