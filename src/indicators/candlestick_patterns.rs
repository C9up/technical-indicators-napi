use napi::bindgen_prelude::*;
use napi_derive::napi;
use crate::helpers::low_high_open_close_volume_date_to_array_helper::process_market_data;

#[napi(object)]
pub struct CandlestickPatterns {
    /// Doji: +1 detected, 0 none
    pub doji: Vec<i32>,
    /// Bullish Engulfing: +1, Bearish Engulfing: -1
    pub engulfing: Vec<i32>,
    /// Hammer: +1 (bullish reversal signal)
    pub hammer: Vec<i32>,
    /// Hanging Man: -1 (bearish reversal signal)
    pub hanging_man: Vec<i32>,
    /// Bullish Harami: +1, Bearish Harami: -1
    pub harami: Vec<i32>,
    /// Morning Star: +1 (bullish three-bar reversal)
    pub morning_star: Vec<i32>,
    /// Evening Star: -1 (bearish three-bar reversal)
    pub evening_star: Vec<i32>,
    /// Three White Soldiers: +1 (strong bullish)
    pub three_white_soldiers: Vec<i32>,
    /// Three Black Crows: -1 (strong bearish)
    pub three_black_crows: Vec<i32>,
    /// Shooting Star: -1 (bearish reversal)
    pub shooting_star: Vec<i32>,
    /// Inverted Hammer: +1 (potential bullish reversal)
    pub inverted_hammer: Vec<i32>,
    /// Spinning Top: +1 (indecision)
    pub spinning_top: Vec<i32>,
    /// Marubozu: +1 bullish (no shadows), -1 bearish
    pub marubozu: Vec<i32>,
    /// Composite signal: sum of all pattern signals at each bar
    pub composite: Vec<i32>,
}

/// Detect common candlestick patterns from OHLC data.
///
/// Returns +1 for bullish patterns, -1 for bearish, 0 for none.
/// All 13 patterns are computed in a single pass for efficiency.
///
/// Parameters:
/// - data: OHLCV market data
/// - body_threshold: max body/range ratio for doji (default: 0.05 = 5%)
#[napi]
pub fn candlestick_patterns(
    data: Vec<crate::MarketData>,
    body_threshold: Option<f64>,
) -> Result<CandlestickPatterns> {
    let market = process_market_data(data);
    let opens = &market.opens;
    let highs = &market.highs;
    let lows = &market.lows;
    let closes = &market.closes;
    let n = closes.len();

    if n < 3 {
        return Err(Error::from_reason("Need at least 3 data points"));
    }

    let bt = body_threshold.unwrap_or(0.05);

    let mut doji = vec![0i32; n];
    let mut engulfing = vec![0i32; n];
    let mut hammer = vec![0i32; n];
    let mut hanging_man = vec![0i32; n];
    let mut harami = vec![0i32; n];
    let mut morning_star = vec![0i32; n];
    let mut evening_star = vec![0i32; n];
    let mut three_white = vec![0i32; n];
    let mut three_black = vec![0i32; n];
    let mut shooting_star = vec![0i32; n];
    let mut inverted_hammer = vec![0i32; n];
    let mut spinning_top = vec![0i32; n];
    let mut marubozu = vec![0i32; n];

    for i in 0..n {
        let o = opens[i];
        let h = highs[i];
        let l = lows[i];
        let c = closes[i];
        let body = (c - o).abs();
        let range = h - l;
        let upper_shadow = h - c.max(o);
        let lower_shadow = c.min(o) - l;
        let is_bullish = c > o;
        let is_bearish = o > c;

        if range < 1e-15 {
            continue;
        }

        let body_ratio = body / range;

        // --- Doji ---
        if body_ratio <= bt {
            doji[i] = 1;
        }

        // --- Spinning Top: small body, both shadows significant ---
        if body_ratio > bt && body_ratio < 0.3
            && upper_shadow > body * 0.5
            && lower_shadow > body * 0.5
        {
            spinning_top[i] = 1;
        }

        // --- Marubozu: body is almost the entire range ---
        if body_ratio > 0.95 {
            marubozu[i] = if is_bullish { 1 } else { -1 };
        }

        // --- Hammer: small body at top, long lower shadow ---
        if is_bullish
            && lower_shadow >= body * 2.0
            && upper_shadow < body * 0.3
            && body_ratio > bt
        {
            // Check if in potential downtrend (prev bar was bearish)
            if i > 0 && closes[i - 1] < opens[i - 1] {
                hammer[i] = 1;
            }
        }

        // --- Hanging Man: same shape as hammer but in uptrend ---
        if body_ratio > bt
            && lower_shadow >= body * 2.0
            && upper_shadow < body * 0.3
            && i > 0 && closes[i - 1] > opens[i - 1]
        {
            hanging_man[i] = -1;
        }

        // --- Shooting Star: small body at bottom, long upper shadow ---
        if body_ratio > bt
            && upper_shadow >= body * 2.0
            && lower_shadow < body * 0.3
            && i > 0 && closes[i - 1] > opens[i - 1]
        {
            shooting_star[i] = -1;
        }

        // --- Inverted Hammer: same shape as shooting star but after downtrend ---
        if body_ratio > bt
            && upper_shadow >= body * 2.0
            && lower_shadow < body * 0.3
            && i > 0 && closes[i - 1] < opens[i - 1]
        {
            inverted_hammer[i] = 1;
        }

        // --- Two-bar patterns (need i >= 1) ---
        if i >= 1 {
            let o1 = opens[i - 1];
            let c1 = closes[i - 1];
            let body1 = (c1 - o1).abs();

            // --- Bullish Engulfing ---
            if o1 > c1 // prev bearish
                && is_bullish
                && o < c1 // open below prev close
                && c > o1 // close above prev open
                && body > body1
            {
                engulfing[i] = 1;
            }

            // --- Bearish Engulfing ---
            if c1 > o1 // prev bullish
                && is_bearish
                && o > c1 // open above prev close
                && c < o1 // close below prev open
                && body > body1
            {
                engulfing[i] = -1;
            }

            // --- Bullish Harami ---
            if o1 > c1 // prev bearish
                && is_bullish
                && o > c1 && c < o1 // current inside prev body
                && body < body1 * 0.6
            {
                harami[i] = 1;
            }

            // --- Bearish Harami ---
            if c1 > o1 // prev bullish
                && is_bearish
                && o < c1 && c > o1 // current inside prev body
                && body < body1 * 0.6
            {
                harami[i] = -1;
            }
        }

        // --- Three-bar patterns (need i >= 2) ---
        if i >= 2 {
            let o2 = opens[i - 2];
            let c2 = closes[i - 2];
            let o1 = opens[i - 1];
            let c1 = closes[i - 1];
            let body2 = (c2 - o2).abs();
            let body1 = (c1 - o1).abs();
            let range1 = highs[i - 1] - lows[i - 1];

            // --- Morning Star ---
            if o2 > c2 // first bar bearish
                && body2 > range * 0.3 // first bar has significant body
                && body1 < body2 * 0.3 // middle bar small body (star)
                && range1 > 0.0
                && is_bullish // third bar bullish
                && c > (o2 + c2) / 2.0 // closes above midpoint of first bar
            {
                morning_star[i] = 1;
            }

            // --- Evening Star ---
            if c2 > o2 // first bar bullish
                && body2 > range * 0.3
                && body1 < body2 * 0.3 // middle bar small
                && is_bearish // third bar bearish
                && c < (o2 + c2) / 2.0 // closes below midpoint of first bar
            {
                evening_star[i] = -1;
            }

            // --- Three White Soldiers ---
            if c2 > o2 && c1 > o1 && is_bullish // all three bullish
                && c1 > c2 && c > c1 // each close higher
                && o1 > o2 && o > o1 // each open higher
                && body2 > range * 0.3 && body1 > range * 0.3 && body > range * 0.3
            {
                three_white[i] = 1;
            }

            // --- Three Black Crows ---
            if o2 > c2 && o1 > c1 && is_bearish // all three bearish
                && c1 < c2 && c < c1 // each close lower
                && o1 < o2 && o < o1 // each open lower
                && body2 > range * 0.3 && body1 > range * 0.3 && body > range * 0.3
            {
                three_black[i] = -1;
            }
        }
    }

    // Composite
    let mut composite = vec![0i32; n];
    for i in 0..n {
        composite[i] = doji[i] + engulfing[i] + hammer[i] + hanging_man[i]
            + harami[i] + morning_star[i] + evening_star[i]
            + three_white[i] + three_black[i]
            + shooting_star[i] + inverted_hammer[i]
            + marubozu[i];
    }

    Ok(CandlestickPatterns {
        doji,
        engulfing,
        hammer,
        hanging_man,
        harami,
        morning_star,
        evening_star,
        three_white_soldiers: three_white,
        three_black_crows: three_black,
        shooting_star,
        inverted_hammer,
        spinning_top,
        marubozu,
        composite,
    })
}
