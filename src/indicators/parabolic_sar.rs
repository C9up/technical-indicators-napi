use napi::bindgen_prelude::*;
use napi_derive::napi;
use crate::low_high_open_close_volume_date_to_array_helper::process_market_data;

fn compute_new_sar(sar_prev: f64, ep: f64, af: f64) -> f64 {
    sar_prev + af * (ep - sar_prev)
}

fn apply_boundaries(is_uptrend: bool, new_sar: f64, highs: &[f64], lows: &[f64], i: usize) -> f64 {
    if is_uptrend {
        let bound = if i >= 2 {
            lows[i - 1].min(lows[i - 2])
        } else {
            lows[i - 1]
        };
        new_sar.min(bound)
    } else {
        let bound = if i >= 2 {
            highs[i - 1].max(highs[i - 2])
        } else {
            highs[i - 1]
        };
        new_sar.max(bound)
    }
}

fn update_ep_and_af(is_uptrend: bool, current_ep: f64, current_af: f64, high: f64, low: f64, increment: f64, max_value: f64) -> (f64, f64) {
    if is_uptrend {
        if high > current_ep {
            let new_ep = high;
            let new_af = (current_af + increment).min(max_value);
            return (new_ep, new_af);
        }
    } else {
        if low < current_ep {
            let new_ep = low;
            let new_af = (current_af + increment).min(max_value);
            return (new_ep, new_af);
        }
    }
    (current_ep, current_af)
}

#[napi]
fn parabolic_sar(
    data: Vec<crate::MarketData>,
    start: Option<f64>,
    increment: Option<f64>,
    max_value: Option<f64>,
) -> Result<Vec<f64>> {
    let data = process_market_data(data);

    let highs = data.highs;
    let lows = data.lows;
    let closes = data.closes;

    let len = highs.len();
    if len < 2 {
        return Err(Error::from_reason("Not enough data."));
    }

    let start = start.unwrap_or(0.02);
    let increment = increment.unwrap_or(0.02);
    let max_value = max_value.unwrap_or(0.2);

    let mut sar = vec![0.0; len];
    let mut af = start;
    let mut is_uptrend = true;
    let mut ep = highs[0];
    sar[0] = closes[0];

    for i in 1..len {
        let provisional_sar = compute_new_sar(sar[i - 1], ep, af);
        let bounded_sar = apply_boundaries(is_uptrend, provisional_sar, &highs, &lows, i);

        if is_uptrend && lows[i] < bounded_sar {
            is_uptrend = false;
            sar[i] = ep;
            ep = lows[i];
            af = start;
        } else if !is_uptrend && highs[i] > bounded_sar {
            is_uptrend = true;
            sar[i] = ep;
            ep = highs[i];
            af = start;
        } else {
            sar[i] = bounded_sar;
            let (new_ep, new_af) = update_ep_and_af(
                is_uptrend,
                ep,
                af,
                highs[i],
                lows[i],
                increment,
                max_value,
            );
            ep = new_ep;
            af = new_af;
        }
    }

    Ok(sar)
}