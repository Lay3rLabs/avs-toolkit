use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;
use cw_orch::{ExecuteFns, QueryFns};

pub use crate::interfaces::tasks::TaskStatus;

// FIXME: make these generic
pub type RequestType = serde_json::Value;
pub type ResponseType = serde_json::Value;

#[cw_serde]
pub struct InstantiateMsg {
    /// The contract storing the operator weights
    pub operator_contract: String,
    /// The percentage of voting power needed to agree in order to complete a task
    pub required_percentage: u32,
}

#[cw_serde]
#[derive(ExecuteFns)]
#[cw_orch(disable_fields_sorting)]
pub enum ExecuteMsg {
    ExecutedTask {
        /// Task queue contract for which we completed the task
        task_queue_contract: String,
        /// The ID of the task that was completed
        task_id: u64,
        /// The result of the task, (JSON) serialized as a string
        /// It is serialized to allow for easy comparison and to avoid field sorting issues when verifying signatures
        result: String,
    },
}

#[cw_serde]
#[derive(QueryFns)]
#[cw_orch(disable_fields_sorting)]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// The contract configuration
    #[returns(ConfigResponse)]
    Config {},
    /// Ordered by completion time descending (last completed first)
    #[returns(Option<TaskInfoResponse>)]
    TaskInfo {
        /// The task contract we are interested in
        task_contract: String,
        /// The ID of the task we are interested in
        task_id: u64,
    },
    /// Ordered by completion time descending (last completed first)
    #[returns(Option<OperatorVoteInfoResponse>)]
    OperatorVote {
        /// The task contract we are interested in
        task_contract: String,
        /// The ID of the task we are interested in
        task_id: u64,
        /// The operator whose vote we are interested in
        operator: String,
    },
}

#[cw_serde]
pub struct ConfigResponse {
    /// The contract storing the operator weights
    pub operator_contract: String,
    /// The percentage of voting power needed to agree in order to complete a task
    pub required_percentage: u32,
}

#[cw_serde]
pub struct TaskInfoResponse {
    // TODO: update based on state we store
    /// The current state of the task
    pub status: TaskStatus,
    /// Total voting power needed to complete the task
    pub power_needed: Uint128,
    /// The various outstanding votes
    pub tallies: Vec<TaskTally>,
}

#[cw_serde]
pub struct TaskTally {
    /// The result that was voted on
    pub result: String,
    /// The total voting power for this result
    pub power: Uint128,
}

#[cw_serde]
pub struct OperatorVoteInfoResponse {
    /// The voting power of the operator for this task
    pub power: Uint128,
    /// The result this operator voted for
    pub result: String,
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

    pub const CONTRACT_ID: &str = "lavs_verifier_simple";

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