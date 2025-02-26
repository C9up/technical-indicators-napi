use napi_derive::napi;
use crate::helpers::low_high_open_close_volume_date_to_array_helper::process_market_data;
use crate::helpers::MarketData;

#[napi]
pub fn directional_movement_index(
    data: Vec<MarketData>,
    period: i32
) -> napi::Result<Vec<f64>> {
    if period <= 0 {
        return Err(napi::Error::from_reason("Period must be greater than 0."));
    }

    let data = process_market_data(data);
    let period = period as usize;

    let highs = data.highs;
    let lows = data.lows;
    let closes = data.closes;

    let len = highs.len();
    if len < period {
        return Err(napi::Error::from_reason("Not enough data points"));
    }

    let mut plus_di = vec![0.0; len];
    let mut minus_di = vec![0.0; len];
    let mut adx = vec![0.0; len];

    let mut tr_values = vec![0.0; len];
    let mut plus_dm = vec![0.0; len];
    let mut minus_dm = vec![0.0; len];

    // Calculate true range and directional movements
    for i in 1..len {
        tr_values[i] = true_range(&highs, &lows, &closes, i);
        let (p_dm, m_dm) = directional_movement(&highs, &lows, i);
        plus_dm[i] = p_dm;
        minus_dm[i] = m_dm;
    }

    // Calculate DI and ADX
    for i in period..len {
        let plus_dm_sum: f64 = plus_dm[i - period..i].iter().sum();
        let minus_dm_sum: f64 = minus_dm[i - period..i].iter().sum();
        let tr_sum: f64 = tr_values[i - period..i].iter().sum();

        if tr_sum != 0.0 {
            plus_di[i] = (plus_dm_sum / tr_sum) * 100.0;
            minus_di[i] = (minus_dm_sum / tr_sum) * 100.0;
        }

        let di_sum = plus_di[i] + minus_di[i];
        if di_sum != 0.0 {
            let di_diff = (plus_di[i] - minus_di[i]).abs();
            adx[i] = (di_diff / di_sum) * 100.0;
        }
    }

    Ok(adx)
}

// Ces fonctions restent identiques à la version originale
fn true_range(highs: &[f64], lows: &[f64], closes: &[f64], i: usize) -> f64 {
    let high_low = highs[i] - lows[i];
    let high_close = (highs[i] - closes[i - 1]).abs();
    let low_close = (lows[i] - closes[i - 1]).abs();
    high_low.max(high_close).max(low_close)
}

fn directional_movement(highs: &[f64], lows: &[f64], i: usize) -> (f64, f64) {
    let up_move = highs[i] - highs[i - 1];
    let down_move = lows[i - 1] - lows[i];

    if up_move > down_move && up_move > 0.0 {
        (up_move, 0.0)
    } else if down_move > up_move && down_move > 0.0 {
        (0.0, down_move)
    } else {
        (0.0, 0.0)
    }
}