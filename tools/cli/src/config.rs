use anyhow::{Context, Result};
use layer_climb::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub local: Option<ChainInfo>,
    pub testnet: Option<ChainInfo>,
}

impl Config {
    // Load the config from the file
    // but in theory this could be from chain, http endpoint, avs, etc.
    // internally, it does additional loads as needed (e.g. from wasmatic endpoint)
    pub async fn load() -> Result<Self> {
        let config: Config = serde_json::from_str(include_str!("../config.json"))
            .context("Failed to parse config")?;

        // SANITY CHECK
        // make sure every chain has the same address kind

        match (&config.local, &config.testnet) {
            (Some(local), Some(testnet)) => {
                if local.chain.address_kind != testnet.chain.address_kind {
                    return Err(anyhow::anyhow!(
                        "Local and testnet chains must have the same address kind"
                    ));
                }
            }
            (None, None) => {
                return Err(anyhow::anyhow!("At least one chain must be configured"));
            }
            _ => {} // Either local or testnet is Some, which is valid
        }

        Ok(config)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChainInfo {
    pub chain: ChainConfig,
    pub faucet: FaucetConfig,
    pub wasmatic: WasmaticConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WasmaticConfig {
    pub endpoints: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FaucetConfig {
    pub mnemonic: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetInfo {
    pub operators: Vec<String>,
}

pub(crate) async fn load_wasmatic_addresses(endpoints: &[String]) -> Result<Vec<String>> {
    let client = reqwest::Client::new();

    futures::future::join_all(endpoints.iter().map(|endpoint| {
        let client = client.clone();
        async move {
            // Load from info endpoint
            let response = client
                .get(format!("{}/info", endpoint))
                .header("Content-Type", "application/json")
                .send()
                .await?;
            let info: GetInfo = response.json().await?;
            info.operators
                .first()
                .context("No operators found")
                .map(|v| v.to_string())
        }
    }))
    .await
    .into_iter()
    .collect::<Result<Vec<String>, _>>()
}
