use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use deadpool::managed::{Object, Pool};
use layer_climb::{pool::SigningClientPoolManager, prelude::*};

use crate::{
    args::{CliArgs, TargetEnvironment},
    config::Config,
};

// The context is relatively cheap to clone, so we can pass it around
#[derive(Clone)]
pub struct AppContext {
    pub args: Arc<CliArgs>,
    pub chain_config: Arc<ChainConfig>,
    // pool of additional clients for concurrent operations
    pub client_pool: Pool<SigningClientPoolManager>,
}

impl AppContext {
    // Getting a context requires parsing the args first
    pub async fn new(args: CliArgs) -> Result<Self> {
        let mnemonic_var = match args.target_env {
            TargetEnvironment::Local => "LOCAL_MNEMONIC",
            TargetEnvironment::Testnet => "TEST_MNEMONIC",
        };

        let mnemonic =
            std::env::var(mnemonic_var).context(format!("Mnemonic not found at {mnemonic_var}"))?;

        let configs: Config = serde_json::from_str(include_str!("../config.json"))
            .context("Failed to parse config")?;

        let chain_config = match args.target_env {
            TargetEnvironment::Local => configs.chains.local,
            TargetEnvironment::Testnet => configs.chains.testnet,
        }
        .context(format!(
            "Chain config for environment {:?} not found",
            args.target_env
        ))?;

        let mut client_pool_manager =
            SigningClientPoolManager::new_mnemonic(mnemonic, chain_config.clone(), None);

        // set the pool minimum balance, if greater than 0
        if args.concurrent_minimum_balance_threshhold > 0 {
            match args.concurrent_minimum_balance_from_faucet {
                true => {
                    let faucet_signer =
                        KeySigner::new_mnemonic_str(&configs.faucet.mnemonic, None)?;
                    let faucet = SigningClient::new(chain_config.clone(), faucet_signer).await?;
                    client_pool_manager = client_pool_manager
                        .with_minimum_balance(
                            args.concurrent_minimum_balance_threshhold,
                            args.concurrent_minimum_balance_amount,
                            Some(faucet),
                            None,
                        )
                        .await?;
                }
                false => {
                    client_pool_manager = client_pool_manager
                        .with_minimum_balance(
                            args.concurrent_minimum_balance_threshhold,
                            args.concurrent_minimum_balance_amount,
                            None,
                            None,
                        )
                        .await?;
                }
            }
        }

        let client_pool: Pool<SigningClientPoolManager> = Pool::builder(client_pool_manager)
            .max_size(args.max_concurrent_accounts.try_into()?)
            .build()
            .context("Failed to create client pool")?;

        Ok(Self { args: Arc::new(args), chain_config: Arc::new(chain_config), client_pool })
    }

    // small helper to make error handling nicer
    pub async fn get_client(&self) -> Result<Object<SigningClientPoolManager>> {
        self.client_pool.get().await.map_err(|e| anyhow!("{e:?}"))
    }

}
