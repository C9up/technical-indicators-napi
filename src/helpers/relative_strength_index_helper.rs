pub fn calculate_gain_loss(values: &[f64], period: usize) -> (f64, f64) {
    let mut gain = 0.0;
    let mut loss = 0.0;

    for i in 1..period {
        let diff = values[i] - values[i - 1];
        if diff > 0.0 {
            gain += diff;
        } else {
            loss -= diff;
        }
    }
    (gain / period as f64, loss / period as f64)
}