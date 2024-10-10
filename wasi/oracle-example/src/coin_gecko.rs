use layer_wasi::{Reactor, Request, WasiPollable};

use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct CoinInfo {
    pub value: f32,
}

#[derive(Deserialize, Debug)]
pub struct CoinGeckoResponse {
    pub rates: HashMap<String, CoinInfo>,
}

impl CoinGeckoResponse {
    fn btc_usd(&self) -> Option<f32> {
        self.rates.get("usd").map(|info| info.value)
    }
}

pub async fn get_btc_usd_price(reactor: &Reactor, api_key: &str) -> Result<Option<f32>, String> {
    let mut req = Request::get("https://api.coingecko.com/api/v3/exchange_rates")?;
    req.headers = vec![("x-cg-pro-api-key".to_string(), api_key.to_owned())];
    let res = reactor.send(req).await?;

    match res.status {
        200 => res.json::<CoinGeckoResponse>().map(|info| info.btc_usd()),
        429 => Err("rate limited, price unavailable".to_string()),
        status => Err(format!("unexpected status code: {status}")),
    }
}
