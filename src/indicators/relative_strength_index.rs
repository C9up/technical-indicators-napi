use napi_derive::napi;
use crate::helpers::relative_strength_index_helper::calculate_gain_loss;

#[napi]
pub fn relative_strength_index(prices: Vec<f64>, period: u32) -> Vec<f64> {
    let period = period as usize;
    let mut rsi_values = Vec::new();

    for i in period..prices.len() {
        let window = &prices[i - period..i];
        let (avg_gain, avg_loss) = calculate_gain_loss(window, period);

        if avg_loss == 0.0 {
            rsi_values.push(100.0);
        } else {
            let rs = avg_gain / avg_loss;
            let rsi = 100.0 - (100.0 / (1.0 + rs));
            rsi_values.push(rsi);
        }
    }

    rsi_values
}