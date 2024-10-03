use anyhow::{Context, Result};
use layer_climb::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub chains: ChainConfigs,
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

        match (&config.chains.local, &config.chains.testnet) {
            (Some(local), Some(testnet)) => {
                if local.address_kind != testnet.address_kind {
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

        Ok(Config {
            chains: config.chains,
        })
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChainConfigs {
    pub local: Option<ChainConfig>,
    pub testnet: Option<ChainConfig>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetInfo {
    pub operators: Vec<String>,
}

pub(crate) async fn load_wasmatic_address(endpoint: &str) -> Result<String> {
    let client = reqwest::Client::new();

    // Load from info endpoint
    let response = client
        .get(format!("{}/info", endpoint))
        .header("Content-Type", "application/json")
        .send()
        .await?;
    let info: GetInfo = response.json().await?;
    let op = info.operators.first().context("No operators found")?;

    Ok(op.to_string())
}
