use anyhow::{anyhow, Result};
use clap::Parser;
use clap::{Args, Subcommand};
use lavs_apis::id::TaskId;
use layer_climb_cli::command::{ContractCommand, WalletCommand};
use std::path::PathBuf;
use std::str::FromStr;

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
        /// A list of operators.
        ///
        /// Voting power will be set with a ':' separator, otherwise it's `1`
        ///
        /// At least one operator must be set
        ///
        /// Tip: "wasmatic" is a special operator that will be set with the wasmatic address
        #[clap(short, long, num_args(1..))]
        operators: Vec<String>,
        /// The default task timeout, in seconds
        #[clap(short, long, default_value_t = 300)]
        timeout: u64,
        /// The required voting percentage for a task to be approved
        #[clap(short, long, default_value_t = 70)]
        percentage: u32,
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
        #[clap(short, long, default_value_t = DeployTaskRequestor::default())]
        requestor: DeployTaskRequestor,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum DeployTaskRequestor {
    Deployer,
    Fixed(String),
    Payment { amount: u128, denom: Option<String> },
}

impl Default for DeployTaskRequestor {
    fn default() -> Self {
        DeployTaskRequestor::Payment {
            amount: 5_000,
            // implementation fills out chain_config.gas_denom in case of None
            denom: None,
        }
    }
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
    ViewQueue {
        #[clap(short, long)]
        start_after: Option<TaskId>,
        #[clap(short, long)]
        limit: Option<u32>,
    },
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

/// Supporting impls needed for custom types
impl FromStr for DeployTaskRequestor {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s == "deployer" {
            Ok(DeployTaskRequestor::Deployer)
        } else if s.starts_with("payment(") && s.ends_with(')') {
            let inner = &s[8..s.len() - 1]; // Extract content inside parentheses
            let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();

            match parts.len() {
                1 => {
                    let amount = parts[0]
                        .parse::<u128>()
                        .map_err(|_| anyhow!("invalid amount"))?;
                    Ok(DeployTaskRequestor::Payment {
                        amount,
                        denom: None,
                    })
                }
                2 => {
                    let amount = parts[0]
                        .parse::<u128>()
                        .map_err(|_| anyhow!("invalid amount"))?;
                    let denom = Some(parts[1].to_string());
                    Ok(DeployTaskRequestor::Payment { amount, denom })
                }
                _ => Err(anyhow!("invalid format")),
            }
        } else if s.starts_with("fixed(") && s.ends_with(')') {
            let inner = &s[6..s.len() - 1]; // Extract content inside parentheses
            Ok(DeployTaskRequestor::Fixed(inner.trim().to_string()))
        } else {
            Err(anyhow!("unknown variant"))
        }
    }
}

impl std::fmt::Display for DeployTaskRequestor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeployTaskRequestor::Payment { amount, denom } => match denom {
                Some(denom) => write!(f, "payment({}, {})", amount, denom),
                None => write!(f, "payment({})", amount),
            },
            DeployTaskRequestor::Fixed(identifier) => {
                write!(f, "fixed({})", identifier)
            }
            DeployTaskRequestor::Deployer => {
                write!(f, "deployer")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_deployer() {
        let input = "deployer";
        let result = DeployTaskRequestor::from_str(input).unwrap();
        assert_eq!(result, DeployTaskRequestor::Deployer);
    }

    #[test]
    fn test_parse_payment_amount_only() {
        let input = "payment(200)";
        let result = DeployTaskRequestor::from_str(input).unwrap();
        assert_eq!(
            result,
            DeployTaskRequestor::Payment {
                amount: 200,
                denom: None,
            }
        );
    }

    #[test]
    fn test_parse_payment_amount_and_denom() {
        let input = "payment(300, USD)";
        let result = DeployTaskRequestor::from_str(input).unwrap();
        assert_eq!(
            result,
            DeployTaskRequestor::Payment {
                amount: 300,
                denom: Some("USD".to_string()),
            }
        );
    }

    #[test]
    fn test_parse_payment_with_whitespace() {
        let input = " payment( 400 ,  EUR ) ";
        let result = DeployTaskRequestor::from_str(input).unwrap();
        assert_eq!(
            result,
            DeployTaskRequestor::Payment {
                amount: 400,
                denom: Some("EUR".to_string()),
            }
        );
    }

    #[test]
    fn test_parse_fixed() {
        let input = "fixed(my_identifier)";
        let result = DeployTaskRequestor::from_str(input).unwrap();
        assert_eq!(
            result,
            DeployTaskRequestor::Fixed("my_identifier".to_string())
        );
    }

    #[test]
    fn test_parse_fixed_with_whitespace() {
        let input = " fixed( my_identifier ) ";
        let result = DeployTaskRequestor::from_str(input).unwrap();
        assert_eq!(
            result,
            DeployTaskRequestor::Fixed("my_identifier".to_string())
        );
    }

    #[test]
    fn test_parse_invalid_variant() {
        let input = "unknown(123)";
        let result = DeployTaskRequestor::from_str(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_amount() {
        let input = "payment(not_a_number)";
        let result = DeployTaskRequestor::from_str(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_format_extra_fields() {
        let input = "payment(100, USD, extra)";
        let result = DeployTaskRequestor::from_str(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_format_no_parentheses() {
        let input = "payment100, USD";
        let result = DeployTaskRequestor::from_str(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_display_payment_without_denom() {
        let requestor = DeployTaskRequestor::Payment {
            amount: 100,
            denom: None,
        };
        assert_eq!(format!("{}", requestor), "payment(100)");
    }

    #[test]
    fn test_display_payment_with_denom() {
        let requestor = DeployTaskRequestor::Payment {
            amount: 200,
            denom: Some("EUR".to_string()),
        };
        assert_eq!(format!("{}", requestor), "payment(200, EUR)");
    }

    #[test]
    fn test_display_fixed() {
        let requestor = DeployTaskRequestor::Fixed("identifier".to_string());
        assert_eq!(format!("{}", requestor), "fixed(identifier)");
    }

    #[test]
    fn test_display_deployer() {
        let requestor = DeployTaskRequestor::Deployer;
        assert_eq!(format!("{}", requestor), "deployer");
    }
}
