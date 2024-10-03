use anyhow::{Context, Result};
use layer_climb::prelude::*;
use serde::{Deserialize, Serialize};

// This is the on-disk config
// it's not the final config that gets filled in asynchrnously
#[derive(Debug, Deserialize, Serialize)]
pub struct OnDiskConfig {
    pub chains: ChainConfigs,
    pub faucet: FaucetConfig,
    pub wasmatic: OnDiskWasmaticConfig,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct OnDiskWasmaticConfig {
    pub endpoint: String,
}

// This is the actual, final config that gets used throughout the app
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub chains: ChainConfigs,
    pub faucet: FaucetConfig,
    pub wasmatic: WasmaticConfig,
}

impl Config {
    // Load the config from the file
    // but in theory this could be from chain, http endpoint, avs, etc.
    // internally, it does additional loads as needed (e.g. from wasmatic endpoint)
    pub async fn load() -> Result<Self> {
        let config: OnDiskConfig = serde_json::from_str(include_str!("../config.json"))
            .context("Failed to parse config")?;

        // SANITY CHECK
        // make sure every chain has the same address kind

        let address_kind = match (&config.chains.local, &config.chains.testnet) {
            (Some(local), Some(testnet)) => {
                if local.address_kind != testnet.address_kind {
                    return Err(anyhow::anyhow!(
                        "Local and testnet chains must have the same address kind"
                    ));
                }

                &local.address_kind
            }
            (Some(local), None) => &local.address_kind,
            (None, Some(testnet)) => &testnet.address_kind,
            (None, None) => return Err(anyhow::anyhow!("At least one chain must be configured")),
        };

        let wasmatic_address = load_wasmatic_address(&config.wasmatic.endpoint).await?;

        Ok(Config {
            wasmatic: WasmaticConfig {
                endpoint: config.wasmatic.endpoint,
                address: match &address_kind {
                    // TODO- this should be a method on AddrKind
                    AddrKind::Cosmos { prefix } => {
                        Address::new_cosmos_string(&wasmatic_address, Some(prefix))?
                    }
                    AddrKind::Eth => Address::new_eth_string(&wasmatic_address)?,
                },
            },
            chains: config.chains,
            faucet: config.faucet,
        })
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetInfo {
    pub operators: Vec<String>,
}

async fn load_wasmatic_address(endpoint: &str) -> Result<String> {
    let client = reqwest::Client::new();

    // Load from info endpoint
    let response = client
        .get(format!("{}/info", endpoint))
        .header("Content-Type", "application/json")
        .send()
        .await?;
    let info: GetInfo = response.json().await?;
    let op = info.operators.get(0).context("No operators found")?;

    Ok(op.to_string())
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WasmaticConfig {
    pub endpoint: String,
    pub address: Address,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChainConfigs {
    pub local: Option<ChainConfig>,
    pub testnet: Option<ChainConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FaucetConfig {
    pub mnemonic: String,
}
