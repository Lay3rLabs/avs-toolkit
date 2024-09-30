use std::{
    ops::{Deref, DerefMut},
    path::PathBuf,
    str::FromStr,
};

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use cosmwasm_std::Coin;
use deadpool::managed::Pool;
use layer_climb::{pool::SigningClientPoolManager, prelude::*};
use serde::{Deserialize, Serialize};

// Args is the thing main _really_ uses, but it depends on CliArgs being parsed first
pub struct Args {
    pub command: Command,
    // pool of additional clients for concurrent operations
    pub client_pool: Pool<SigningClientPoolManager>,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    #[arg(long, value_enum, default_value_t = TargetEnvironment::Local)]
    pub target_env: TargetEnvironment,

    /// Set the logging level
    #[arg(long, value_enum, default_value_t = LogLevel::Info)]
    //#[arg(long, value_enum, default_value_t = LogLevel::Debug)]
    pub log_level: LogLevel,

    /// max concurrent accounts in the pool
    #[arg(long, default_value_t = 3)]
    pub max_concurrent_accounts: u32,

    /// minimum balance required for all the concurrent accounts in the pool
    /// set to 0 if you don't want any automatic minimum balance top-up
    #[arg(long, default_value_t = 200_000)]
    pub concurrent_minimum_balance_threshhold: u128,

    /// amount sent to top-up accounts that fall below the minimum balance threshhold
    #[arg(long, default_value_t = 2_000_000)]
    pub concurrent_minimum_balance_amount: u128,

    /// Will use the faucet account for the minimum balance top-up (if set)
    /// if this is false, then the first derivation will be used instead
    #[arg(long, default_value_t = true)]
    pub concurrent_minimum_balance_from_faucet: bool,

    #[command(subcommand)]
    pub command: Command,
}

impl Args {
    pub async fn new(cli_args: CliArgs) -> Result<Self> {
        let mnemonic_var = match cli_args.target_env {
            TargetEnvironment::Local => "LOCAL_MNEMONIC",
            TargetEnvironment::Testnet => "TEST_MNEMONIC",
        };

        let mnemonic =
            std::env::var(mnemonic_var).context(format!("Mnemonic not found at {mnemonic_var}"))?;

        let configs: Config = serde_json::from_str(include_str!("../config.json"))
            .context("Failed to parse config")?;

        let chain_config = match cli_args.target_env {
            TargetEnvironment::Local => configs.chains.local,
            TargetEnvironment::Testnet => configs.chains.testnet,
        }
        .context(format!(
            "Chain config for environment {:?} not found",
            cli_args.target_env
        ))?;

        let mut client_pool_manager =
            SigningClientPoolManager::new_mnemonic(mnemonic, chain_config.clone(), None);

        // set the pool minimum balance, if greater than 0
        if cli_args.concurrent_minimum_balance_threshhold > 0 {
            match cli_args.concurrent_minimum_balance_from_faucet {
                true => {
                    let faucet_signer =
                        KeySigner::new_mnemonic_str(&configs.faucet.mnemonic, None)?;
                    let faucet = SigningClient::new(chain_config, faucet_signer).await?;
                    client_pool_manager = client_pool_manager
                        .with_minimum_balance(
                            cli_args.concurrent_minimum_balance_threshhold,
                            cli_args.concurrent_minimum_balance_amount,
                            Some(faucet),
                            None,
                        )
                        .await?;
                }
                false => {
                    client_pool_manager = client_pool_manager
                        .with_minimum_balance(
                            cli_args.concurrent_minimum_balance_threshhold,
                            cli_args.concurrent_minimum_balance_amount,
                            None,
                            None,
                        )
                        .await?;
                }
            }
        }

        let client_pool: Pool<SigningClientPoolManager> = Pool::builder(client_pool_manager)
            .max_size(cli_args.max_concurrent_accounts.try_into()?)
            .build()
            .context("Failed to create client pool")?;

        Ok(Self {
            command: cli_args.command,
            client_pool,
        })
    }
}

#[derive(Copy, Clone, Debug, clap::ValueEnum)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl From<LogLevel> for tracing::Level {
    fn from(log_level: LogLevel) -> Self {
        match log_level {
            LogLevel::Trace => tracing::Level::TRACE,
            LogLevel::Debug => tracing::Level::DEBUG,
            LogLevel::Info => tracing::Level::INFO,
            LogLevel::Warn => tracing::Level::WARN,
            LogLevel::Error => tracing::Level::ERROR,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum TargetEnvironment {
    Local,
    Testnet,
}

#[derive(Clone, Subcommand)]
pub enum Command {
    /// Deploys the contracts
    DeployContracts {
        // set the default
        #[clap(short, long, default_value = "../../artifacts")]
        artifacts_path: PathBuf,
    },
}

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    pub chains: ChainConfigs,
    pub faucet: FaucetConfig,
}
#[derive(Debug, Deserialize, Serialize)]
struct ChainConfigs {
    pub local: Option<ChainConfig>,
    pub testnet: Option<ChainConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FaucetConfig {
    pub mnemonic: String,
}
