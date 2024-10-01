/// These are queries and messages can be sent to any task service implementation
/// by other contracts. This specifies the required APIs for on-chain interactions.
/// This must be a subset of any of the implementation.
use cosmwasm_schema::{cw_serde, QueryResponses};
// FIXME: do we need to derive cw_orch? This is not used by the contracts calling,
// And the external callers likely use the full APIs.
// TODO: try embedding these (enum with serde(untagged)) in the main types of the implementing contracts
use cw_orch::{ExecuteFns, QueryFns};

use crate::tasks::Status;

// FIXME: hot to make these generic
pub type ResponseType = serde_json::Value;

// FIXME: we need the impl_into here to allow it to be embedded.
// Which means it must be aware of everywhere it is embedded.
// Can we add an impl_from on the other side?
#[cw_serde]
#[derive(ExecuteFns)]
#[cw_orch(disable_fields_sorting)]
pub enum TaskExecuteMsg {
    /// This can only be called by the verifier contract
    Complete {
        /// The task ID to complete
        task_id: u64,
        /// The result of the task
        response: ResponseType,
    },
}

// FIXME: same issue as above / impl_into vs impl_from
#[cw_serde]
#[derive(QueryFns)]
#[cw_orch(disable_fields_sorting)]
#[derive(QueryResponses)]
pub enum TaskQueryMsg {
    /// Get specific task details
    #[returns(TaskStatusResponse)]
    TaskStatus { id: u64 },
}

/// This is detailed information about a task, including the payload
#[cw_serde]
pub struct TaskStatusResponse {
    pub id: u64,
    pub status: TaskStatus,
    /// We need the height to query the voting power
    pub created_height: u64,
    /// Time it was created in UNIX seconds
    pub created_time: u64,
    /// Expiration of the task in UNIX seconds
    pub expires_time: u64,
}

#[derive(Copy)]
#[cw_serde]
pub enum TaskStatus {
    Open,
    Completed,
    Expired,
}

impl From<Status> for TaskStatus {
    fn from(status: Status) -> Self {
        match status {
            Status::Open { .. } => TaskStatus::Open,
            Status::Completed { .. } => TaskStatus::Completed,
            Status::Expired { .. } => TaskStatus::Expired,
        }
    }
}
