use std::sync::Arc;

use anyhow::{Context, Result};
use deadpool::managed::Pool;
use layer_climb::{pool::SigningClientPoolManager, prelude::*};
use rand::{rngs::StdRng, SeedableRng};

use crate::{
    args::{CliArgs, TargetEnvironment},
    config::{ChainInfo, Config, FaucetConfig},
};

// The context is relatively cheap to clone, so we can pass it around
#[derive(Clone)]
pub struct AppContext {
    pub args: Arc<CliArgs>,
    pub config: Arc<Config>,
    // this is held across an await point, so use async mutex to be safe
    pub rng: Arc<tokio::sync::Mutex<StdRng>>,
}

impl AppContext {
    // Getting a context requires parsing the args first
    pub async fn new(args: CliArgs) -> Result<Self> {
        Ok(Self {
            args: Arc::new(args),
            config: Arc::new(Config::load().await?),
            rng: Arc::new(tokio::sync::Mutex::new(StdRng::from_entropy())),
        })
    }

    pub fn chain_config(&self) -> Result<&ChainConfig> {
        self.chain_info().map(|ci| &ci.chain)
    }

    pub fn chain_info(&self) -> Result<&ChainInfo> {
        match self.args.target {
            TargetEnvironment::Local => &self.config.local,
            TargetEnvironment::Testnet => &self.config.testnet,
        }
        .as_ref()
        .context(format!(
            "Chain config for environment {:?} not found",
            self.args.target
        ))
    }

    pub fn client_mnemonic(&self) -> Result<String> {
        let mnemonic_var = match self.args.target {
            TargetEnvironment::Local => "LOCAL_MNEMONIC",
            TargetEnvironment::Testnet => "TEST_MNEMONIC",
        };

        std::env::var(mnemonic_var)
            .ok()
            .and_then(|m| if m.is_empty() { None } else { Some(m) })
            .context(format!("Mnemonic not found at {mnemonic_var}"))
    }

    // if we have a valid mnemonic, then get a signing client
    // otherwise, get a query client
    pub async fn any_client(&self) -> Result<AnyClient> {
        match self.client_mnemonic() {
            Ok(mnemonic) => {
                let signer = KeySigner::new_mnemonic_str(&mnemonic, None)?;
                Ok(AnyClient::Signing(
                    SigningClient::new(self.chain_config()?.clone(), signer).await?,
                ))
            }
            Err(_) => Ok(AnyClient::Query(self.query_client().await?)),
        }
    }

    pub async fn signing_client(&self) -> Result<SigningClient> {
        self.any_client().await?.try_into()
    }

    pub async fn query_client(&self) -> Result<QueryClient> {
        QueryClient::new(self.chain_config()?.clone()).await
    }

    pub async fn faucet_client(&self) -> Result<Option<SigningClient>> {
        match &self.chain_info()?.faucet {
            Some(faucet) => {
                let signer = KeySigner::new_mnemonic_str(&faucet.mnemonic, None)?;
                Ok(Some(
                    SigningClient::new(self.chain_config()?.clone(), signer).await?,
                ))
            }
            None => Ok(None),
        }
    }

    pub async fn create_client_pool(&self) -> Result<Pool<SigningClientPoolManager>> {
        let start_index = match &self.chain_info()?.faucet {
            Some(FaucetConfig { mnemonic })
                if self.args.concurrent_minimum_balance_from_faucet
                    && &self.client_mnemonic()? == mnemonic =>
            {
                Some(1)
            }
            _ => None,
        };

        let mut client_pool_manager = SigningClientPoolManager::new_mnemonic(
            self.client_mnemonic()?,
            self.chain_config()?.clone(),
            start_index,
        );

        // set the pool minimum balance, if greater than 0
        if self.args.concurrent_minimum_balance_threshhold > 0 {
            match self.args.concurrent_minimum_balance_from_faucet {
                true => {
                    client_pool_manager = client_pool_manager
                        .with_minimum_balance(
                            self.args.concurrent_minimum_balance_threshhold,
                            self.args.concurrent_minimum_balance_amount,
                            self.faucet_client().await?,
                            None,
                        )
                        .await?;
                }
                false => {
                    client_pool_manager = client_pool_manager
                        .with_minimum_balance(
                            self.args.concurrent_minimum_balance_threshhold,
                            self.args.concurrent_minimum_balance_amount,
                            None,
                            None,
                        )
                        .await?;
                }
            }
        }

        let client_pool: Pool<SigningClientPoolManager> = Pool::builder(client_pool_manager)
            .max_size(self.args.max_concurrent_accounts.try_into()?)
            .build()
            .context("Failed to create client pool")?;

        Ok(client_pool)
    }
}
