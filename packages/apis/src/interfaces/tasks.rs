/// These are queries and messages can be sent to any task service implementation
/// by other contracts. This specifies the required APIs for on-chain interactions.
/// This must be a subset of any of the implementation.
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Decimal, DepsMut, Env, StdResult, Storage, Timestamp};
use cw_orch::{ExecuteFns, QueryFns};
use cw_storage_plus::Map;

use crate::{
    id::TaskId,
    tasks::Status,
    verifier_simple::{TaskMetadata, VerifierError},
};

use crate::interfaces::voting::QueryMsg as OperatorQueryMsg;

use super::voting::TotalPowerResponse;

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

    pub fn handle_metadata(
        deps: DepsMut,
        env: &Env,
        operator_addr: &Addr,
        task_queue: &Addr,
        task_id: TaskId,
        fraction_percent: u32,
    ) -> Result<TaskMetadata, VerifierError> {
        let tasks_storage = TasksStorage::new("tasks");
        let metadata = tasks_storage.get_tasks(deps.storage, (task_queue, task_id))?;

        match metadata {
            Some(meta) => {
                // Ensure this is not yet expired (or completed)
                match meta.status {
                    TaskStatus::Completed => Err(VerifierError::TaskAlreadyCompleted),
                    TaskStatus::Expired => Err(VerifierError::TaskExpired),
                    TaskStatus::Open if meta.is_expired(env) => Err(VerifierError::TaskExpired),
                    _ => Ok(meta.clone()),
                }
            }
            None => {
                // We need to query the info from the task queue
                let task_status: TaskStatusResponse = deps.querier.query_wasm_smart(
                    task_queue.to_string(),
                    &TaskQueryMsg::TaskStatus { id: task_id },
                )?;
                // Abort early if not still open
                match task_status.status {
                    TaskStatus::Completed => Err(VerifierError::TaskAlreadyCompleted),
                    TaskStatus::Expired => Err(VerifierError::TaskExpired),
                    TaskStatus::Open => {
                        // If we create this, we need to calculate total vote power needed
                        let total_power: TotalPowerResponse = deps.querier.query_wasm_smart(
                            operator_addr.to_string(),
                            &OperatorQueryMsg::TotalPowerAtHeight {
                                height: Some(task_status.created_height),
                            },
                        )?;
                        // need to round up!
                        // NOTE: for now we are using percentage, with oracle-verifier
                        // this would change
                        let fraction = Decimal::percent(fraction_percent as u64);
                        let power_required = total_power.power.mul_ceil(fraction);
                        let meta = TaskMetadata {
                            power_required,
                            status: TaskStatus::Open,
                            created_height: task_status.created_height,
                            expires_time: task_status.expires_time,
                        };
                        tasks_storage.save_tasks(
                            deps.storage,
                            (task_queue, task_id),
                            meta.clone(),
                        )?;
                        Ok(meta)
                    }
                }
            }
        }
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
    /// Time it was created
    pub created_time: Timestamp,
    /// Expiration of the task
    pub expires_time: Timestamp,
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
