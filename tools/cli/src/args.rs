use clap::Parser;
use clap::{Args, Subcommand};
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
    /// Deploys the contracts
    DeployContracts {
        // set the default
        #[clap(short, long, default_value = "../../artifacts")]
        artifacts_path: PathBuf,
    },

    /// Commands for task queue
    TaskQueue(TaskQueueArgs),
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
