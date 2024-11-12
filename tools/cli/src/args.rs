use avs_toolkit_shared::deploy::DeployContractArgsRequestor;
use clap::Parser;
use clap::{Args, Subcommand, ValueEnum};
use cosmwasm_std::Decimal;
use lavs_apis::id::TaskId;
use lavs_apis::interfaces::task_hooks::TaskHookType;
use layer_climb_cli::command::{ContractCommand, WalletCommand};
use std::fmt;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    #[arg(long, value_enum, default_value_t = TargetEnvironment::Testnet)]
    // #[arg(long, value_enum, default_value_t = TargetEnvironment::Local)]
    pub target: TargetEnvironment,

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
    /// Upload subcommands
    Upload(UploadArgs),
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

    /// Commands for working with wasmatic
    Wasmatic(WasmaticArgs),
}

#[derive(Clone, Args)]
pub struct UploadArgs {
    #[command(subcommand)]
    pub command: UploadCommand,
}

#[derive(Clone, Subcommand)]
pub enum UploadCommand {
    /// Upload, but do not instantiate, all the core contracts
    Contracts {
        /// Artifacts path
        #[clap(short, long, default_value = "../../artifacts")]
        artifacts_path: PathBuf,
    },
}

#[derive(Clone, Args)]
pub struct DeployArgs {
    #[clap(short, long)]
    pub mode: DeployMode,

    #[command(subcommand)]
    pub command: DeployCommand,
}

#[derive(Clone, PartialEq, ValueEnum)]
pub enum DeployMode {
    VerifierSimple,
    OracleVerifier,
}

#[derive(Clone, Subcommand)]
pub enum DeployCommand {
    /// Deploy all the contracts needed for the system to work
    Contracts {
        /// Artifacts path
        #[clap(short, long, default_value = "../../artifacts")]
        artifacts_path: PathBuf,
        /// A list of operators.
        ///
        /// Voting power will be set with a ':' separator, otherwise it's `1`
        ///
        /// At least one operator must be set
        ///
        /// Tip: "wasmatic" is a special operator that will be set with the wasmatic address
        #[clap(long, num_args(1..))]
        operators: Vec<String>,
        /// The task queue owner.
        ///
        /// Responsible for managing task hooks on the task queue
        ///
        /// Defaults to sender
        #[clap(long)]
        owner: Option<String>,
        /// The default task timeout, in seconds
        #[clap(short, long, default_value_t = 300)]
        timeout: u64,
        /// The required voting percentage for a task to be approved
        #[clap(short, long, default_value_t = 70)]
        percentage: u32,
        /// What percentage of the operators must submit their vote (optional for certain contracts)
        #[clap(long)]
        threshold_percentage: Option<Decimal>,
        /// Maximum allowed difference between the votes of operators (optional for certain contracts)
        #[clap(long)]
        allowed_spread: Option<Decimal>,
        /// Difference bigger than `slashable_spread` would slash the operators (optional for certain contracts)
        #[clap(long)]
        slashable_spread: Option<Decimal>,
        /// The rules for allowed task requestors
        ///
        /// Examples:
        ///
        /// "payment(100)" - will require a payment of 100 gas tokens
        ///
        /// "payment(100, uslay)" - will require a payment of 100 uslay (same as above)
        ///
        /// "fixed(slayaddresshere)" - will require the caller be this specific address
        ///
        /// "deployer" - will require the caller be the same as the deployer
        #[clap(short, long, default_value_t = DeployContractArgsRequestor::default())]
        requestor: DeployContractArgsRequestor,
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
        /// Specify the completed task hook receivers
        #[clap(long, value_delimiter = ',')]
        with_completed_hooks: Option<Vec<String>>,
        /// Specify the timeout task hook receivers
        #[clap(long, value_delimiter = ',')]
        with_timeout_hooks: Option<Vec<String>>,
    },

    /// View the task queue
    ViewQueue {
        #[clap(short, long)]
        start_after: Option<TaskId>,
        #[clap(short, long)]
        limit: Option<u32>,
    },

    /// Adds hooks to the task queue
    AddHooks {
        #[clap(long, value_enum)]
        hook_type: CliHookType,
        #[clap(short, long, num_args(1..), value_delimiter = ',')]
        receivers: Vec<String>,
        #[clap(short, long)]
        task_id: Option<TaskId>,
    },

    /// Removes a task queue hook
    RemoveHook {
        #[clap(long, value_enum)]
        hook_type: CliHookType,
        #[clap(short, long)]
        receiver: String,
        #[clap(short, long)]
        task_id: Option<TaskId>,
    },

    /// Views the task hooks of a type
    ViewHooks {
        #[clap(short, long)]
        task_id: Option<TaskId>,
        #[clap(long, value_enum)]
        hook_type: CliHookType,
    },

    /// Updates the task-specific whitelist for hook management from task creators
    /// These users can create hooks for their task
    UpdateTaskSpecificWhitelist {
        #[clap(long, value_delimiter = ',')]
        to_add: Option<Vec<String>>,
        #[clap(long, value_delimiter = ',')]
        to_remove: Option<Vec<String>>,
    },

    /// View the task-specific hook whitelist
    ViewTaskSpecificWhitelist {
        #[clap(short, long)]
        start_after: Option<String>,
        #[clap(short, long)]
        limit: Option<u32>,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CliHookType {
    /// Hook triggered when a task is completed
    Completed,
    /// Hook triggered when a task times out
    Timeout,
    /// Hook triggered when a task is created
    Created,
}

impl fmt::Display for CliHookType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliHookType::Completed => write!(f, "Completed"),
            CliHookType::Timeout => write!(f, "Timeout"),
            CliHookType::Created => write!(f, "Created"),
        }
    }
}

impl From<CliHookType> for TaskHookType {
    fn from(cli_type: CliHookType) -> Self {
        match cli_type {
            CliHookType::Completed => TaskHookType::Completed,
            CliHookType::Timeout => TaskHookType::Timeout,
            CliHookType::Created => TaskHookType::Created,
        }
    }
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

#[derive(Clone, Args)]
pub struct WasmaticArgs {
    #[command(subcommand)]
    pub command: WasmaticCommand,
}

#[derive(Clone, Subcommand)]
pub enum WasmaticCommand {
    /// Deploy a Wasm application
    Deploy {
        /// Name of the application
        #[clap(short, long)]
        name: String,

        /// Digest of the wasm file (sha256)
        #[clap(short, long)]
        digest: Option<String>,

        /// Path to the Wasm file or a URL to the Wasm file
        #[clap(short, long)]
        wasm_source: String, // This can be a local path or a URL

        /// Cron schedule for the trigger (either this or task_trigger must be set)
        #[clap(long("cron"))]
        cron_trigger: Option<String>,

        /// Task queue to trigger the action
        #[clap(long("task"))]
        task_trigger: Option<String>,

        /// HD Index if using task trigger
        #[clap(long, default_value = "0")]
        hd_index: u32,

        /// Poll Interval if using task trigger
        #[clap(long, default_value = "3")]
        poll_interval: u32,

        /// Permissions, defaults to an empty array
        #[clap(short, long, default_value = "{}")]
        permissions: String,

        /// Environment variables, multiple can be provided in KEY=VALUE format
        #[clap(long)]
        envs: Vec<String>,

        /// Set to true to test the application (not for production)
        #[clap(long, default_value = "f")]
        testable: bool,
    },

    /// Remove a Wasm application
    Remove {
        /// The name of the application to remove
        #[clap(short, long)]
        name: String,
    },

    /// Run a Wasm application locally
    Run {
        /// Path to the Wasm file or a URL to the Wasm file
        #[clap(short, long)]
        wasm_source: String, // This can be a local path or a URL

        /// Cron trigger the action, otherwise task queue trigger
        #[clap(long("cron"))]
        cron_trigger: bool,

        /// Environment variables, multiple can be provided in KEY=VALUE format
        #[clap(long)]
        envs: Vec<String>,

        /// App cache directory to use, otherwise will default to temporary directory
        #[clap(long)]
        dir: Option<PathBuf>,

        /// Optional input for the test
        #[clap(short, long)]
        input: Option<String>,
    },

    /// Test a Wasm application
    Test {
        /// The name of the application to test
        #[clap(short, long)]
        name: String,

        /// Optional input for the test
        #[clap(short, long)]
        input: Option<String>,
    },

    /// Returns info about wasmatic operators
    Info {},

    /// Returns info deployed apps and sha256 digests
    App {
        #[clap(short, long)]
        endpoint: Option<String>,
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
