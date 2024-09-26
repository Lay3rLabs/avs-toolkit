use std::{path::PathBuf, str::FromStr};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use cosmwasm_std::Coin;
use layer_climb::prelude::*;
use serde::{Deserialize, Serialize};

// Args is the thing main _really_ uses, but it depends on CliArgs being parsed first
pub struct Args {
    pub command: Command,
    pub client: SigningClient,
}

impl Args {
    pub async fn new(cli_args: CliArgs) -> Result<Self> {
        let env_var_key = match cli_args.target_env {
            TargetEnvironment::Local => "LOCAL_MNEMONIC",
            TargetEnvironment::Testnet => "TEST_MNEMONIC",
        };

        let mnemonic =
            std::env::var(env_var_key).context(format!("Mnemonic not found at {env_var_key}"))?;

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

        tracing::info!("Creating signing client...");

        let signer = KeySigner::new_mnemonic_str(&mnemonic, None)?;
        let client = SigningClient::new(chain_config, signer).await?;

        tracing::info!("Our address is {}", client.addr);

        Ok(Self {
            command: cli_args.command,
            client,
        })
    }
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

    #[command(subcommand)]
    pub command: Command,
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
