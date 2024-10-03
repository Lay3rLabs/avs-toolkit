use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Env};
use cw_orch::{ExecuteFns, QueryFns};

use crate::id::TaskId;
pub use crate::interfaces::tasks::{
    TaskExecuteMsg, TaskExecuteMsgFns, TaskQueryMsg, TaskQueryMsgFns, TaskStatus,
    TaskStatusResponse,
};

// FIXME: make these generic
pub type RequestType = serde_json::Value;
pub type ResponseType = serde_json::Value;

#[cw_serde]
pub struct InstantiateMsg {
    /// Who can create new tasks
    pub requestor: Requestor,
    /// Timeout configuration for the tasks
    pub timeout: TimeoutInfo,
    /// Which contract can verify results
    pub verifier: String,
}

#[cw_serde]
pub enum Requestor {
    Fixed(String),
    OpenPayment(Coin),
}

#[cw_serde]
/// All timeouts are defined in seconds
/// If minimum and maximum are undefined, the default value is used
pub struct TimeoutInfo {
    pub default: u64,
    pub minimum: Option<u64>,
    pub maximum: Option<u64>,
}

impl TimeoutInfo {
    pub fn new(default: u64) -> Self {
        Self {
            default,
            minimum: None,
            maximum: None,
        }
    }
}

#[cw_serde]
#[serde(untagged)]
pub enum ExecuteMsg {
    /// Complete and any other public APIs in the interface
    Api(TaskExecuteMsg),
    /// The messages unique to this contract implementation
    Custom(CustomExecuteMsg),
}

#[cw_serde]
#[derive(ExecuteFns)]
#[cw_orch(disable_fields_sorting)]
pub enum CustomExecuteMsg {
    #[cw_orch(payable)]
    Create {
        /// Human-readable description of the task
        description: String,
        /// Specify a task timeout, or use the default
        timeout: Option<u64>,
        /// Machine-readable data for the AVS to use
        /// FIXME: use generic T to enforce a AVS-specific format
        payload: RequestType,
    },
    Timeout {
        /// The task ID to complete
        task_id: TaskId,
    },
}

impl From<CustomExecuteMsg> for ExecuteMsg {
    fn from(value: CustomExecuteMsg) -> Self {
        Self::Custom(value)
    }
}

impl From<TaskExecuteMsg> for ExecuteMsg {
    fn from(value: TaskExecuteMsg) -> Self {
        Self::Api(value)
    }
}

#[cw_serde]
#[derive(QueryResponses)]
#[query_responses(nested)]
#[serde(untagged)]
pub enum QueryMsg {
    /// Complete and any other public APIs in the interface
    Api(TaskQueryMsg),
    /// The messages unique to this implementation
    Custom(CustomQueryMsg),
}

#[cw_serde]
#[derive(QueryFns)]
#[cw_orch(disable_fields_sorting)]
#[derive(QueryResponses)]
pub enum CustomQueryMsg {
    /// Ordered by expiration time ascending
    #[returns(ListOpenResponse)]
    ListOpen {
        start_after: Option<TaskId>,
        limit: Option<u32>,
    },
    /// Ordered by completion time descending (last completed first)
    #[returns(ListCompletedResponse)]
    ListCompleted {
        start_after: Option<TaskId>,
        limit: Option<u32>,
    },
    /// Get specific task details
    #[returns(TaskResponse)]
    Task { id: TaskId },
    /// Get task configuration
    #[returns(ConfigResponse)]
    Config {},
}

impl From<TaskQueryMsg> for QueryMsg {
    fn from(value: TaskQueryMsg) -> Self {
        Self::Api(value)
    }
}

impl From<CustomQueryMsg> for QueryMsg {
    fn from(value: CustomQueryMsg) -> Self {
        Self::Custom(value)
    }
}

#[cw_serde]
pub struct ListOpenResponse {
    pub tasks: Vec<OpenTaskOverview>,
}

#[cw_serde]
pub struct ListCompletedResponse {
    pub tasks: Vec<CompletedTaskOverview>,
}

/// Minimal information about a task
#[cw_serde]
pub struct OpenTaskOverview {
    pub id: TaskId,
    pub expires: u64,
    pub payload: RequestType,
}

/// Minimal information about a task
#[cw_serde]
pub struct CompletedTaskOverview {
    pub id: TaskId,
    pub completed: u64,
    pub result: ResponseType,
}

#[cw_serde]
pub struct ConfigResponse {
    pub requestor: Requestor,
    pub timeout: TimeoutConfig,
    pub verifier: String,
}

/// All timeouts are defined in seconds
/// This is configured from `TimeoutInfo`, which is passed in the instantiate message
#[cw_serde]
pub struct TimeoutConfig {
    pub default: u64,
    pub minimum: u64,
    pub maximum: u64,
}

/// This is detailed information about a task, including the payload
#[cw_serde]
pub struct TaskResponse {
    pub id: TaskId,
    pub description: String,
    pub status: Status,
    pub payload: RequestType,
    pub result: Option<ResponseType>,
}

#[cw_serde]
pub enum Status {
    Open {},
    Completed { completed: u64 },
    Expired {},
}

impl Default for Status {
    fn default() -> Self {
        Self::new()
    }
}

impl Status {
    pub fn new() -> Self {
        Self::Open {}
    }

    pub fn completed(env: &Env) -> Self {
        Status::Completed {
            completed: env.block.time.seconds(),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Status::Open {} => "open",
            Status::Completed { .. } => "completed",
            Status::Expired {} => "expired",
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use deployment::Contract;

#[cfg(not(target_arch = "wasm32"))]
mod deployment {
    use super::*;
    use cosmwasm_std::Empty;
    use cw_orch::interface;

    use cw_orch::prelude::*;

    /// This is a minimal cw_orch bindings, useful for interacting with an instantiated contract
    /// in eg. deployment scripts.
    /// Please use the full implementation in the contract itself for multitest.

    pub const CONTRACT_ID: &str = "lavs_task_queue";

    #[interface(InstantiateMsg, ExecuteMsg, QueryMsg, Empty, id = CONTRACT_ID)]
    pub struct Contract;

    impl<Chain> Uploadable for Contract<Chain> {
        /// Return the path to the wasm file corresponding to the contract
        fn wasm(_chain: &ChainInfoOwned) -> WasmPath {
            artifacts_dir_from_workspace!()
                .find_wasm_path(CONTRACT_ID)
                .unwrap()
        }

        /// Will not be implemented, panics!
        fn wrapper() -> Box<dyn MockContract<Empty>> {
            panic!("This is a deployment stub, for multi-test use the real implementation in the contract itself");
        }
    }
}
