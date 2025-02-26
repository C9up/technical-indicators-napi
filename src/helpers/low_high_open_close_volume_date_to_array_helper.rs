use napi_derive::napi;
use crate::{MarketData, MarketDataResult};

pub fn process_market_data(market_data: Vec<MarketData>) -> MarketDataResult {
    let mut lows = Vec::new();
    let mut highs = Vec::new();
    let mut opens = Vec::new();
    let mut closes = Vec::new();
    let mut volumes = Vec::new();
    let mut dates = Vec::new();

    for item in market_data {
        lows.push(item.low);
        highs.push(item.high);
        opens.push(item.open);
        closes.push(item.close);
        volumes.push(item.volume);
        dates.push(item.date);
    }

    MarketDataResult {
        lows,
        highs,
        opens,
        closes,
        volumes,
        dates,
    }
}

#[napi(js_name = "lowHighOpenCloseVolumeDateToArray")]
pub fn low_high_open_close_volume_date_to_array(data: Vec<MarketData>) -> MarketDataResult {
    process_market_data(data)
}