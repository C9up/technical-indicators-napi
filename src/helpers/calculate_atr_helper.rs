pub fn calculate_atr(prices: &[f64], period: i32) -> Result<Vec<f64>, String> {
    let mut atr_values = Vec::new();

    if prices.len() < (period + 1) as usize {
        return Err("Insufficient data for ATR calculation".to_string());
    }

    let mut tr_values = Vec::new();

    // Calcul du True Range (TR)
    for i in 1..prices.len() {
        let current_price = prices[i];
        let prev_close = prices[i - 1];
        let high = current_price;
        let low = current_price;

        let tr = f64::max(
            f64::max(high - low, (high - prev_close).abs()),
            (low - prev_close).abs(),
        );
        tr_values.push(tr);
    }

    let period_usize = period as usize;

    if tr_values.len() < period_usize {
        return Err("Insufficient TR values for ATR calculation".to_string());
    }

    // Calcul de l'ATR avec une moyenne mobile simple (SMA) des TR
    for i in period_usize..tr_values.len() {
        let sum: f64 = tr_values[i - period_usize..i].iter().sum();
        let atr = sum / period as f64;
        atr_values.push(atr);
    }

    Ok(atr_values)
}