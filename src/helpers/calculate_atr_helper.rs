pub fn calculate_atr(highs: &[f64], lows: &[f64], closes: &[f64], period: usize) -> Result<Vec<f64>, String> {
    let len = highs.len();

    if len != lows.len() || len != closes.len() {
        return Err("Highs, lows and closes must have the same length".to_string());
    }

    if len < period + 1 {
        return Err("Insufficient data for ATR calculation".to_string());
    }

    // Calculate True Range
    let mut tr_values = Vec::with_capacity(len - 1);
    for i in 1..len {
        let tr = (highs[i] - lows[i])
            .max((highs[i] - closes[i - 1]).abs())
            .max((lows[i] - closes[i - 1]).abs());
        tr_values.push(tr);
    }

    // First ATR is SMA of first `period` TR values
    let first_atr: f64 = tr_values[..period].iter().sum::<f64>() / period as f64;
    let mut atr_values = Vec::with_capacity(tr_values.len() - period + 1);
    atr_values.push(first_atr);

    // Subsequent ATR values using Wilder's smoothing
    for tr in &tr_values[period..] {
        let prev = *atr_values.last().unwrap();
        let atr = (prev * (period as f64 - 1.0) + tr) / period as f64;
        atr_values.push(atr);
    }

    Ok(atr_values)
}
