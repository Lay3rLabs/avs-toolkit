use anyhow::Result;
use clap::{Parser, Subcommand};

// The full args used in main. parsed from CliArgs which comes from clap
pub struct Args {
    pub command: Command,
    pub chain_kind: ChainKind,
}

impl Args {
    pub async fn new(args: CliArgs) -> Result<Self> {
        Ok(Self {
            command: args.command,
            chain_kind: args.chain_kind,
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

    /// Set the chain kind
    #[arg(long, value_enum, default_value_t = ChainKind::Local)]
    pub chain_kind: ChainKind,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Copy, Clone, Debug, clap::ValueEnum)]
pub enum ChainKind {
    Local,
    Testnet,
    Mainnet,
    Unspecified,
}

impl From<ChainKind> for cw_orch::environment::ChainKind {
    fn from(chain_kind: ChainKind) -> Self {
        match chain_kind {
            ChainKind::Local => cw_orch::environment::ChainKind::Local,
            ChainKind::Testnet => cw_orch::environment::ChainKind::Testnet,
            ChainKind::Mainnet => cw_orch::environment::ChainKind::Mainnet,
            ChainKind::Unspecified => cw_orch::environment::ChainKind::Unspecified,
        }
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
    /// Deploy the AVS Contracts
    DeployAvsContracts {},
}
