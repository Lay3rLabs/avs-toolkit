use clap::Parser;
use clap::{Args, Subcommand};
use layer_climb_cli::command::{ContractCommand, WalletCommand};
use std::path::PathBuf;

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

#[derive(Clone, Subcommand)]
pub enum Command {
    /// Deploy subcommands
    Deploy(DeployArgs),
    /// Task queue subcommands
    TaskQueue(TaskQueueArgs),
    /// Faucet subcommands
    Faucet(FaucetArgs),
    /// Wallet subcommands
    Wallet(WalletArgs),
    /// Generic utility contract subcommands
    Contract(ContractArgs),
}

#[derive(Clone, Args)]
pub struct DeployArgs {
    #[command(subcommand)]
    pub command: DeployCommand,
}

#[derive(Clone, Subcommand)]
pub enum DeployCommand {
    /// Deploy all the contracts needed for the system to work
    Contracts {
        /// Artifacts path
        #[clap(short, long, default_value = "../../artifacts")]
        artifacts_path: PathBuf,
    },
}

#[derive(Clone, Args)]
pub struct WalletArgs {
    #[command(subcommand)]
    pub command: WalletCommand,
}

#[derive(Clone, Args)]
pub struct ContractArgs {
    #[command(subcommand)]
    pub command: ContractCommand,
}

#[derive(Clone, Args)]
pub struct TaskQueueArgs {
    /// Task queue address. If not provided, then it will be read
    /// from the environment variable LOCAL_TASK_QUEUE_ADDRESS or TEST_TASK_QUEUE_ADDRESS
    /// depending on the target environment
    #[clap(long)]
    pub address: Option<String>,

    #[command(subcommand)]
    pub command: TaskQueueCommand,
}

#[derive(Clone, Subcommand)]
pub enum TaskQueueCommand {
    /// Commands for task queue
    AddTask {
        /// The body of the task, must be valid JSON
        #[clap(short, long)]
        body: String,
        /// Human-readable description of the task
        #[clap(short, long)]
        description: String,
        /// Specify a task timeout, or use the default
        #[clap(short, long)]
        timeout: Option<u64>,
    },

    /// View the task queue
    ViewQueue,
}

#[derive(Clone, Args)]
pub struct FaucetArgs {
    #[command(subcommand)]
    pub command: FaucetCommand,
}

#[derive(Clone, Subcommand)]
pub enum FaucetCommand {
    /// Tap the faucet to get some funds
    Tap {
        /// The address to send the funds to
        /// if not set, will be the default client
        #[arg(long)]
        to: Option<String>,
        /// The amount to send
        /// if not set, will be `Self::DEFAULT_TAP_AMOUNT`
        #[arg(long)]
        amount: Option<u128>,
        /// The denom of the funds to send, if not set will use the chain gas denom
        #[arg(long)]
        denom: Option<String>,
    },
}

impl FaucetCommand {
    pub const DEFAULT_TAP_AMOUNT: u128 = 1_000_000;
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
