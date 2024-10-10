#[allow(warnings)]
mod bindings;
use bindings::{Guest, Output, TaskQueueInput};

mod coin_gecko;
mod price_history;

use layer_wasi::{block_on, Reactor};

use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

struct Component;

impl Guest for Component {
    fn run_task(_input: TaskQueueInput) -> Output {
        block_on(get_avg_btc)
    }
}

/// Record the latest BTCUSD price and return the JSON serialized result to write to the chain.
async fn get_avg_btc(reactor: Reactor) -> Result<Vec<u8>, String> {
    let api_key = std::env::var("API_KEY").or(Err("missing env var `API_KEY`".to_string()))?;
    let price = coin_gecko::get_btc_usd_price(&reactor, &api_key)
        .await
        .map_err(|err| err.to_string())?
        .ok_or("invalid response from coin gecko API")?;

    // read previous price history
    let mut history = price_history::PriceHistory::read()?;

    // get current time in secs
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("failed to get current time")
        .as_secs();

    // record latest price
    history.record_latest_price(now, price)?;

    // calculate average price over the past hour
    let avg_last_hour = history.average(now - 3600);

    CalculatedPrices {
        price: avg_last_hour.price.to_string(),
    }
    .to_json()
}

/// The returned result.
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct CalculatedPrices {
    price: String,
}

impl CalculatedPrices {
    /// Serialize to JSON.
    fn to_json(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec(&self).map_err(|err| err.to_string())
    }
}

bindings::export!(Component with_types_in bindings);
