use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin, Env, Timestamp};
use cw_orch::{ExecuteFns, QueryFns};
use cw_ownable::{cw_ownable_execute, cw_ownable_query, Ownership};

pub use crate::interfaces::tasks::{
    TaskExecuteMsg, TaskExecuteMsgFns, TaskQueryMsg, TaskQueryMsgFns, TaskStatus,
    TaskStatusResponse,
};
use crate::{id::TaskId, interfaces::task_hooks::TaskHookType, time::Duration};

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
    /// The address that owns and manages the task queue.
    ///
    /// ## Privileges
    /// - Can update task hooks
    /// - Can remove task hooks
    /// - Can update task specific whitelist
    /// - Can transfer ownership
    ///
    /// Defaults to the message sender during initialization.
    pub owner: Option<String>,
    /// Optionally populate the task-specific whitelist at instantiation
    pub task_specific_whitelist: Option<Vec<String>>,
}

#[cw_serde]
pub enum Requestor {
    Fixed(String),
    OpenPayment(Coin),
}

#[cw_serde]
/// If minimum and maximum are undefined, the default value is used
/// # Fields
/// * `default` - default timeout duration.
/// * `minimum` - the minimum allowed timeout duration.
/// * `maximum` - maximum allowed timeout duration.
pub struct TimeoutInfo {
    pub default: Duration,
    pub minimum: Option<Duration>,
    pub maximum: Option<Duration>,
}

impl TimeoutInfo {
    pub fn new(default: Duration) -> Self {
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

#[cw_ownable_execute]
#[cw_serde]
#[derive(ExecuteFns)]
#[cw_orch(disable_fields_sorting)]
pub enum CustomExecuteMsg {
    #[cw_orch(payable)]
    Create {
        /// Human-readable description of the task
        description: String,
        /// Specify a task timeout, or use the default
        timeout: Option<Duration>,
        /// Machine-readable data for the AVS to use
        /// FIXME: use generic T to enforce a AVS-specific format
        payload: RequestType,
        /// Optionally register timeout task hooks for a set of receivers
        /// Requires the sender to be in the task-specific whitelist
        with_timeout_hooks: Option<Vec<String>>,
        /// Optionally register completed task hooks for a set of receivers
        /// Requires the sender to be in the task-specific whitelist
        with_completed_hooks: Option<Vec<String>>,
    },
    Timeout {
        /// The task ID to complete
        task_id: TaskId,
    },
    /// Adds hooks to a set of receivers for the given task hook type
    AddHooks {
        /// Optional task id for task-specific hooks. If None, adds a global hook.
        task_id: Option<TaskId>,
        /// The type of hook to add
        hook_type: TaskHookType,
        /// The receiver addresses of the hook messages
        receivers: Vec<String>,
    },
    /// Remove a hook from a receiver
    RemoveHook {
        /// Optional task id to remove a task-specific hook. If None, removes a global hook.
        task_id: Option<TaskId>,
        /// The type of hook to remove (Created, Completed, or Timeout)
        hook_type: TaskHookType,
        /// The receiver address that will stop receiving hook messages
        receiver: String,
    },
    /// Update task-specific whitelist
    /// These users are allowed to add task hooks to their submissions
    UpdateTaskSpecificWhitelist {
        to_add: Option<Vec<String>>,
        to_remove: Option<Vec<String>>,
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

#[cw_ownable_query]
#[cw_serde]
#[derive(QueryFns)]
#[cw_orch(disable_fields_sorting)]
#[derive(QueryResponses)]
pub enum CustomQueryMsg {
    /// List all tasks, ordered descending by task ID
    #[returns(ListResponse)]
    List {
        start_after: Option<TaskId>,
        limit: Option<u32>,
    },
    /// List open tasks, ordered descending by task ID
    #[returns(ListOpenResponse)]
    ListOpen {
        start_after: Option<TaskId>,
        limit: Option<u32>,
    },
    /// List completed tasks, ordered descending by task ID
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
    /// Gets the task hooks for the given task hook type
    #[returns(crate::interfaces::task_hooks::HooksResponse)]
    TaskHooks {
        hook_type: TaskHookType,
        task_id: Option<TaskId>,
    },
    #[returns(TaskSpecificWhitelistResponse)]
    TaskSpecificWhitelist {
        start_after: Option<String>,
        limit: Option<u32>,
    },
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

/// This is detailed information about a task for listing, including the
/// payload, result, and timing information.
#[cw_serde]
pub struct TaskInfoResponse {
    pub id: TaskId,
    pub description: String,
    pub status: InfoStatus,
    pub payload: RequestType,
    pub result: Option<ResponseType>,
    pub created_at: Timestamp,
}

#[cw_serde]
pub enum InfoStatus {
    Open { expires: Timestamp },
    Completed { completed: Timestamp },
    Expired { expired: Timestamp },
}

#[cw_serde]
pub struct ListResponse {
    pub tasks: Vec<TaskInfoResponse>,
}

#[cw_serde]
pub struct ListOpenResponse {
    pub tasks: Vec<OpenTaskOverview>,
}

#[cw_serde]
pub struct TaskSpecificWhitelistResponse {
    pub addrs: Vec<Addr>,
}

#[cw_serde]
pub struct ListCompletedResponse {
    pub tasks: Vec<CompletedTaskOverview>,
}

/// Minimal information about a task
#[cw_serde]
pub struct OpenTaskOverview {
    pub id: TaskId,
    pub expires: Timestamp,
    pub payload: RequestType,
}

/// Minimal information about a task
#[cw_serde]
pub struct CompletedTaskOverview {
    pub id: TaskId,
    pub completed: Timestamp,
    pub result: ResponseType,
}

#[cw_serde]
pub struct ConfigResponse {
    pub ownership: Ownership<Addr>,
    pub requestor: Requestor,
    pub timeout: TimeoutConfig,
    pub verifier: String,
}

/// All timeouts are defined in seconds
/// This is configured from `TimeoutInfo`, which is passed in the instantiate message
#[cw_serde]
pub struct TimeoutConfig {
    pub default: Duration,
    pub minimum: Duration,
    pub maximum: Duration,
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
    Completed { completed: Timestamp },
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
            completed: env.block.time,
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
