/// These are queries and messages can be sent to any task service implementation
/// by other contracts. This specifies the required APIs for on-chain interactions.
/// This must be a subset of any of the implementation.
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, StdResult, Storage};
use cw_orch::{ExecuteFns, QueryFns};
use cw_storage_plus::Map;

use crate::{id::TaskId, tasks::Status, verifier_simple::TaskMetadata};

// FIXME: hot to make these generic
pub type ResponseType = serde_json::Value;

pub struct TasksStorage<'a>(Map<(&'a Addr, TaskId), TaskMetadata>);

impl<'a> TasksStorage<'a> {
    pub const fn new(storage_key: &'static str) -> Self {
        TasksStorage(Map::new(storage_key))
    }
    /// key is Task queue address: &Addr and the taskid: TaskId
    pub fn get_tasks(
        &self,
        store: &mut dyn Storage,
        key: (&Addr, TaskId),
    ) -> StdResult<Option<TaskMetadata>> {
        match self.0.may_load(store, key) {
            Ok(meta) => Ok(meta),
            Err(e) => Err(e),
        }
    }

    pub fn save_tasks(
        &self,
        store: &mut dyn Storage,
        key: (&Addr, TaskId),
        metadata: TaskMetadata,
    ) -> StdResult<()> {
        self.0.save(store, key, &metadata)?;
        Ok(())
    }
}

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
        task_id: TaskId,
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
    TaskStatus { id: TaskId },
}

/// This is detailed information about a task, including the payload
#[cw_serde]
pub struct TaskStatusResponse {
    pub id: TaskId,
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
